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

use crate::context;
use crate::handle::{HandleInstance, MethodId};
use crate::PacketHeader;
use std::io::Cursor;

pub fn call<S: serde::Serialize, D: serde::de::DeserializeOwned>(
    handle: &HandleInstance,
    method: MethodId,
    args: &S,
) -> D {
    #[cfg(fml_statistics)]
    {
        crate::statistics::CALL_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    let mut buffer: Vec<u8> = Vec::new();
    buffer.resize(std::mem::size_of::<PacketHeader>(), 0 as u8);
    serde_cbor::to_writer(
        {
            let mut c = Cursor::new(&mut buffer);
            c.set_position(std::mem::size_of::<PacketHeader>() as u64);
            c
        },
        &args,
    )
    .unwrap();

    let context = context::global::get();
    let port_table = context.read().unwrap();
    let port = &port_table.map.get(&handle.port_id_importer).expect("PortTable corrupted").2;
    let result = port.call(handle.id, method, buffer);
    serde_cbor::from_reader(&result[std::mem::size_of::<PacketHeader>()..]).unwrap()
}

pub fn delete(handle: &HandleInstance) {
    #[cfg(fml_statistics)]
    {
        crate::statistics::DELETE_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    let context = context::global::get();
    let port_table = context.read().unwrap();
    if port_table.no_drop {
        return
    }
    let port = &port_table.map.get(&handle.port_id_importer).expect("PortTable corrupted").2;
    port.delete(handle.id);
}
