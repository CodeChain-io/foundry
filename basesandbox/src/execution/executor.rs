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
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::ipc::*;
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

pub trait Executor: Send {
    fn new(path: &str, args: &[&str]) -> Self;
    fn join(&mut self);
}

pub struct Executable {
    child: std::process::Child,
}

impl Executor for Executable {
    fn new(path: &str, args: &[&str]) -> Self {
        let mut command = Command::new(path);
        command.args(args);
        Executable {
            child: command.spawn().unwrap(),
        }
    }

    fn join(&mut self) {
        // This is synchronized with excutee's signal (#TERMINATE),
        // which is supposed to be sent in termination step.
        // Thus, in normal case, it won't take much time to be in a waitable status.
        // However a malicous excutee might send a signal arbitrarily before the termination.
        // For that, we have a timeout for the wait.
        for _ in 0..10 {
            std::thread::sleep(std::time::Duration::from_millis(100));
            if self.child.try_wait().unwrap().is_some() {
                return // successful
            }
        }
        panic!("Module hasn't terminated itself. Protocol Error")
    }
}

/// This is for the intra-process tasks. You can register a runnable function,
/// and it will be executed later as an instance of 'process' (which is actually not).
pub type ThreadAsProcesss = Arc<dyn Fn(Vec<String>) -> () + Send + Sync>;

static POOL: OnceCell<RwLock<HashMap<String, ThreadAsProcesss>>> = OnceCell::new();

fn get_function_pool(key: &str) -> ThreadAsProcesss {
    POOL.get_or_init(Default::default).read().get(key).unwrap().clone()
}

pub fn add_function_pool(key: String, f: ThreadAsProcesss) {
    assert!(POOL.get_or_init(Default::default).write().insert(key, f).is_none());
}

pub struct PlainThread {
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Executor for PlainThread {
    fn new(path: &str, args: &[&str]) -> Self {
        let path = path.to_owned();
        let mut args: Vec<String> = args.iter().map(|&x| x.to_string()).collect();
        args.insert(0, "Thread".to_owned()); // corresponding to program path
        let handle = std::thread::spawn(move || get_function_pool(&path)(args));

        PlainThread {
            handle: Some(handle),
        }
    }

    fn join(&mut self) {
        // PlainThread Executor is for test, so no worry for malicous unresponsiveness
        self.handle.take().unwrap().join().unwrap();
    }
}

/// Rust doesn't allow Drop for trait, so we need this. See E0120
struct ExecutorDropper<T: Executor> {
    executor: T,
}

impl<T: Executor> Drop for ExecutorDropper<T> {
    fn drop(&mut self) {
        self.executor.join();
    }
}

/// declaration order of fields is important because of Drop dependencies
pub struct Context<T: Ipc, E: Executor> {
    pub ipc: T,
    _child: ExecutorDropper<E>,
}

/// id must be unique for each instance.
pub fn execute<T: Ipc + 'static, E: Executor>(path: &str) -> Result<Context<T, E>, String> {
    let (config_server, config_client) = T::arguments_for_both_ends();
    let ipc = std::thread::spawn(move || T::new(config_server));
    let config_client = hex::encode(&config_client);
    let args: Vec<&str> = vec![&config_client];
    let child = ExecutorDropper {
        executor: Executor::new(path, &args),
    };
    let ipc = ipc.join().unwrap();
    let ping = ipc.recv(Some(Duration::from_millis(1000))).unwrap();
    assert_eq!(ping, b"#INIT\0");
    Ok(Context {
        ipc,
        _child: child,
    })
}

impl<T: Ipc, E: Executor> Context<T, E> {
    /// Call this when you're sure that the excutee is ready to teminate; i.e.
    /// it will call excutee::terminate() asap.
    pub fn terminate(self) {
        let signal = self.ipc.recv(Some(Duration::from_millis(1000))).unwrap();
        assert_eq!(signal, b"#TERMINATE\0");
        self.ipc.send(b"#TERMINATE\0");
    }
}
