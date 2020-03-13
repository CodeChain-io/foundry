// Copyright 2020 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License

use crate::ipc::semaphore::WaitOnlySemaphore;
use crate::ipc::Ipc;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const SOCKET_SRC: &str = "./tmp/Foundry_Socket_Boxer";
const SOCKET_DST: &str = "./tmp/Foundry_Socket_Boxee";
const SEMAPHORE: &str = "./tmp/Foundry_Semaphore";

struct Executor {
    child: std::process::Child,
}

impl Executor {
    fn new(path: &str, args: &[&str]) -> Self {
        let mut command = Command::new(path);
        command.args(args);
        Executor {
            child: command.spawn().unwrap(),
        }
    }
}

impl Drop for Executor {
    fn drop(&mut self) {
        // This is synchronized with excutee's signal,
        // which is supposed to be sent in termination step.
        // Thus, in normal case, it won't take much time to be in a waitable status.
        // However a malicous excutee might send a signal arbitrarily before the termination.
        // For that, we have a timeout for the wait.
        for _ in 0..10 {
            std::thread::sleep(std::time::Duration::from_millis(50));
            if self.child.try_wait().unwrap().is_some() {
                return // successful
            }
        }
        panic!("Module hasn't terminated itself. Protocol Error")
    }
}

struct DirectoryReserver {
    path: String,
}

impl DirectoryReserver {
    fn new(path: String) -> Self {
        std::fs::create_dir(&path).unwrap();
        DirectoryReserver {
            path,
        }
    }
}

impl Drop for DirectoryReserver {
    fn drop(&mut self) {
        std::fs::remove_dir(&self.path).unwrap();
    }
}

/// declaration order of fields is important because of Drop dependencies
pub struct Context<T: Ipc> {
    pub ipc: T,
    pub semaphore: WaitOnlySemaphore,
    _child: Executor,
    _directory: DirectoryReserver,
}

// Note that field's drop is called later that of struct.
impl<T: Ipc> Drop for Context<T> {
    fn drop(&mut self) {
        self.semaphore.wait();
    }
}

/// id must be unique for each instance.
pub fn execute<T: Ipc>(path: &str, id: &str) -> Result<Context<T>, String> {
    let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let hash = ccrypto::blake256(format!("{:?}{}", time, id));
    let addr_src = format!("{}.{}", SOCKET_SRC, hash);
    let addr_dst = format!("{}.{}", SOCKET_DST, hash);
    let addr_sem = format!("{}.{}", SEMAPHORE, hash);

    let directory = DirectoryReserver::new("./tmp".to_owned());
    // src-dst order is reversed for the boxee
    let args: Vec<&str> = vec![&addr_dst, &addr_src, &addr_sem];

    // Here the order of 5 statements is very important
    let mut semaphore = WaitOnlySemaphore::new(addr_sem.clone() + "_Boxer", addr_sem.clone() + "_Boxee");
    let child = Executor::new(path, &args);
    semaphore.wait();
    let ipc = T::new(addr_src, addr_dst);
    ipc.send(b"#INIT");

    Ok(Context {
        ipc,
        semaphore,
        _child: child,
        _directory: directory,
    })
}
