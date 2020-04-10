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

use coordinator::validator::TxOrigin;
use ctypes::TxHash;
use std::collections::HashMap;

pub struct TxMetadata {
    pub mem_usage: usize,
    pub gas: usize,
    pub origin: TxOrigin,
    pub insertion_id: u64,
    pub custom_metadata: HashMap<&'static str, String>,
    pub tx_hash: TxHash,
}

#[derive(Debug)]
// Error which cannot be handled by orderer
pub enum OrdererError {
    MetadataFieldNotFound {
        field: &'static str,
        tx_hash: TxHash,
    },
}
