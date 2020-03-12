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

pub mod domain_socket;
pub mod semaphore;

pub trait Ipc {
    /// We expect two sides of IPC to initialize themselve in a same routine.
    fn new(address_src: String, address_dst: String) -> Self;

    /// It might block until counterparty's recv(). Even if not, the order is still guaranteed.
    fn send(&self, data: &[u8]);
    /// Synchronous recv.
    fn recv(&mut self) -> Vec<u8>;
}
