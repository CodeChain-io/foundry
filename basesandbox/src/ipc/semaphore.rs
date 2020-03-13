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

use crate::ipc::domain_socket::DomainSocket;
use crate::ipc::Ipc;
use nix::errno::Errno;
use nix::libc;

#[derive(PartialEq)]
pub enum Mode {
    CREATE,
    OPEN,
}

/// Inter-process semaphore using POSIX
pub struct Semaphore {
    raw: *mut libc::sem_t,
    mode: Mode,
    name: String,
}

impl Semaphore {
    pub fn new(name: String, mode: Mode) -> Self {
        unsafe {
            if Mode::CREATE == mode {
                libc::sem_unlink(name.as_ptr() as *const i8);
            }

            let semaphore = match mode {
                Mode::CREATE => libc::sem_open(
                    name.as_ptr() as *const i8,
                    nix::fcntl::OFlag::O_CREAT.bits(),
                    (libc::S_IRWXU | libc::S_IRWXG | libc::S_IRWXO) as libc::c_uint,
                    0,
                ),
                Mode::OPEN => libc::sem_open(name.as_ptr() as *const i8, 0),
            };
            assert_ne!(semaphore, libc::SEM_FAILED, "Failed to create semaphore: {}", Errno::last());
            Semaphore {
                raw: semaphore,
                mode,
                name,
            }
        }
    }

    pub fn wait(&mut self) {
        unsafe {
            libc::sem_wait(self.raw);
        }
    }

    pub fn signal(&mut self) {
        unsafe {
            libc::sem_post(self.raw);
        }
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        unsafe {
            if let Mode::CREATE = self.mode {
                libc::sem_close(self.raw);
                libc::sem_unlink(self.name.as_ptr() as *const i8);
            }
        }
    }
}

// One-way, One-cosumer, One-produce limited semaphore
// It is less platform-dependent and supports Send + Sync
pub struct WaitOnlySemaphore {
    socket: DomainSocket,
}

pub struct SignalOnlySemaphore {
    socket: DomainSocket,
}

impl WaitOnlySemaphore {
    pub fn new(name: String, counter_name: String) -> Self {
        WaitOnlySemaphore {
            socket: DomainSocket::new(name, counter_name),
        }
    }

    pub fn wait(&mut self) {
        let r = self.socket.recv();
        assert_eq!(r, []);
    }
}

impl SignalOnlySemaphore {
    pub fn new(name: String, counter_name: String) -> Self {
        SignalOnlySemaphore {
            socket: DomainSocket::new(name, counter_name),
        }
    }

    pub fn signal(&mut self) {
        self.socket.send(&[]);
    }
}
