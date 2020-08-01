// Copyright 2018-2020 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

mod block;
mod transaction;
mod unsigned_transaction;
mod work;

pub use self::block::Block;
pub use self::block::BlockNumberAndHash;
pub use self::transaction::{PendingTransactions, Transaction};
pub use self::unsigned_transaction::UnsignedTransaction;
pub use self::work::Work;

use ctypes::TxHash;
use primitives::H256;

#[derive(Debug, Serialize, Deserialize)]
pub struct FilterStatus {
    pub list: Vec<(::cidr::IpCidr, String)>,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendTransactionResult {
    pub hash: TxHash,
    pub seq: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TPSTestSetting {
    pub count: u64,
    pub seed: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorSet(Vec<ValidatorSetEntry>);

impl ValidatorSet {
    pub fn from_core(validator_set: ctypes::CompactValidatorSet) -> Self {
        let entries = Vec::<ctypes::CompactValidatorEntry>::from(validator_set)
            .into_iter()
            .map(ValidatorSetEntry::from_core)
            .collect();
        ValidatorSet(entries)
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidatorSetEntry {
    pub public_key: H256,
    pub delegation: u64,
}

impl ValidatorSetEntry {
    pub fn from_core(validator_set: ctypes::CompactValidatorEntry) -> Self {
        ValidatorSetEntry {
            public_key: H256::from_slice(validator_set.public_key.as_ref()),
            delegation: validator_set.delegation,
        }
    }
}
