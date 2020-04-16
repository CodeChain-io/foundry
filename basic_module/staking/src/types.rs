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

pub use fkey::{sign, verify, Ed25519Private as Private, Ed25519Public as Public, Signature};
pub use ftypes::Header;

pub type Bytes = Vec<u8>;
pub type StakeQuantity = u64;
pub type DepositQuantity = u64;

#[derive(Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize, Clone)]
pub struct Validator {
    // Indicates weights in a round-robin proposer scheduling
    pub weight: StakeQuantity,
    pub delegation: StakeQuantity,
    pub deposit: DepositQuantity,
    pub pubkey: Public,
}

impl Validator {
    pub fn new(delegation: StakeQuantity, deposit: DepositQuantity, pubkey: Public) -> Self {
        Self {
            weight: delegation,
            delegation,
            deposit,
            pubkey,
        }
    }

    pub fn reset(&mut self) {
        self.weight = self.delegation;
    }

    pub fn pubkey(&self) -> &Public {
        &self.pubkey
    }

    pub fn delegation(&self) -> StakeQuantity {
        self.delegation
    }
}

#[derive(Serialize, Deserialize)]
pub struct Candidate {
    pub pubkey: Public,
    pub deposit: DepositQuantity,
    pub nomination_ends_at: u64,
    pub metadata: Bytes,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Prisoner {
    pub pubkey: Public,
    pub deposit: DepositQuantity,
    pub custody_until: u64,
    pub released_at: u64,
}

pub enum ReleaseResult {
    NotExists,
    InCustody,
    Released(Prisoner),
}
