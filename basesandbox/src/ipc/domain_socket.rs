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

use super::Ipc;
use std::os::unix::net::UnixDatagram;
use std::path::Path;

pub struct DomainSocket {
    address_src: String,
    address_dst: String,
    socket: UnixDatagram,
    buffer: Vec<u8>,
}

fn create(address_src: String, address_dst: String) -> DomainSocket {
    let socket = UnixDatagram::bind(&address_src).unwrap();
    DomainSocket {
        address_src,
        address_dst,
        socket,
        buffer: vec![0; 1024],
    }
}

/// No distinction between server / client for DomainSocket in Drop
impl Drop for DomainSocket {
    fn drop(&mut self) {
        self.socket.shutdown(std::net::Shutdown::Both).unwrap();
        std::fs::remove_file(&self.address_src).unwrap();
    }
}

impl Ipc for DomainSocket {
    fn new(address_src: String, address_dst: String) -> Self {
        create(address_src, address_dst)
    }

    fn send(&self, data: &[u8]) {
        assert_eq!(self.socket.send_to(data, &self.address_dst).unwrap(), data.len());
    }

    fn recv(&mut self) -> Vec<u8> {
        let (count, address) = self.socket.recv_from(&mut self.buffer).unwrap();
        assert!(count <= self.buffer.len(), "Unix datagram got data larger than the buffer.");
        assert_eq!(
            address.as_pathname().unwrap(),
            Path::new(&self.address_dst),
            "Unix datagram received packet from an unexpected sender."
        );
        self.buffer[0..count].to_vec()
    }
}
