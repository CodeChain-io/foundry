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

use ckey::Ed25519Public as Public;

pub type StakeQuantity = u64;
pub type DepositQuantity = u64;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, RlpDecodable, RlpEncodable)]
pub struct Validator {
    weight: StakeQuantity,
    delegation: StakeQuantity,
    deposit: DepositQuantity,
    pubkey: Public,
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

    pub fn weight(&self) -> StakeQuantity {
        self.weight
    }

    pub fn set_weight(&mut self, weight: StakeQuantity) {
        self.weight = weight;
    }

    pub fn pubkey(&self) -> &Public {
        &self.pubkey
    }

    pub fn delegation(&self) -> StakeQuantity {
        self.delegation
    }

    pub fn deposit(&self) -> DepositQuantity {
        self.deposit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ckey::Ed25519Public as Public;
    use rlp::rlp_encode_and_decode_test;

    #[test]
    fn encode_and_decode_pay_action() {
        rlp_encode_and_decode_test!(Validator {
            weight: 1,
            delegation: 2,
            deposit: 3,
            pubkey: Public::random(),
        });
    }
}
