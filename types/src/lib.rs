// Copyright 2018-2020 Kodebox, Inc.
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

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate rlp_derive;

mod block_hash;
mod common_params;
mod deposit;
mod tracker;
mod tx_hash;
mod validator_set;

pub mod errors;
pub mod header;
pub mod transaction;
pub mod util;

pub type BlockNumber = u64;
pub type StorageId = u16;

pub use block_hash::BlockHash;
pub use common_params::CommonParams;
pub use deposit::Deposit;
pub use header::Header;
pub use tracker::Tracker;
pub use tx_hash::TxHash;
pub use validator_set::CompactValidatorEntry;
pub use validator_set::CompactValidatorSet;
