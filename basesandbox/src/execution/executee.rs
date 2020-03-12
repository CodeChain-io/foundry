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

use crate::ipc::semaphore::{Mode, Semaphore};
use crate::ipc::Ipc;

// Interface for the sandboxee written in Rust
pub struct Context<T: Ipc> {
    pub ipc: T,
    pub semaphore: Semaphore,
}

impl<T: Ipc> Drop for Context<T> {
    fn drop(&mut self) {
        // Note: the child must exit as soon as possible after you call this.
        self.semaphore.signal();
    }
}

pub fn start<T: Ipc>() -> Context<T> {
    let args: Vec<String> = std::env::args().collect();

    let socket_src_name = &args[1];
    let socket_dst_name = &args[2];
    let semaphore_name = &args[3];

    let mut semaphore = Semaphore::new(semaphore_name.to_string(), Mode::OPEN);

    let mut ipc = T::new(socket_src_name.to_string(), socket_dst_name.to_string());
    semaphore.signal();
    assert_eq!(ipc.recv(), b"#INIT");

    Context {
        ipc,
        semaphore,
    }
}
