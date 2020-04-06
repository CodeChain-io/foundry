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

// Interface for the sandboxee written in Rust
pub struct Context<T: Ipc> {
    /// ipc will be given with Some, but module may take it
    /// However, the ipc must be return back here before the module terminates
    pub ipc: Option<T>,
}

pub fn start<T: Ipc>(mut args: Vec<String>) -> Context<T> {
    let ipc = T::new(hex::decode(args.remove(1)).unwrap());
    ipc.send(b"#INIT\0");
    Context {
        ipc: Some(ipc),
    }
}

impl<T: Ipc> Context<T> {
    /// Tell the executor that I will exit asap after this byebye handshake.
    pub fn terminate(self) {
        let ipc = self.ipc.unwrap();
        ipc.send(b"#TERMINATE\0");
        assert_eq!(ipc.recv(Some(std::time::Duration::from_millis(1000))).unwrap(), b"#TERMINATE\0");
    }
}
