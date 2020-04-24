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

use crate::state::{NextValidators, Params};
use crate::types::{verify, Approval, Bytes, DepositQuantity, NetworkId, Public, Signature, StakeQuantity, Validator};
use codechain_crypto::blake256;
use primitives::H256;

#[allow(dead_code)]
pub enum Transaction {
    User(SignedTransaction),
    Auto(AutoAction),
}

pub struct SignedTransaction {
    pub signature: Signature,
    pub signer_public: Public,
    pub tx: UserTransaction,
}

impl SignedTransaction {
    pub fn verify(&self) -> bool {
        let message = self.tx.hash();
        verify(&self.signature, &message, &self.signer_public)
    }
}

#[derive(Serialize)]
pub struct UserTransaction {
    /// Seq
    pub seq: u64,
    /// Quantity of CCC to be paid as a cost for distributing this transaction to the network.
    pub fee: u64,
    // Network id
    pub network_id: NetworkId,
    pub action: UserAction,
}

impl UserTransaction {
    pub fn hash(&self) -> H256 {
        let serialized = serde_cbor::to_vec(&self).unwrap();
        blake256(serialized)
    }
}

#[allow(dead_code)]
#[derive(Serialize)]
pub enum UserAction {
    TransferCCS {
        receiver_public: Public,
        quantity: StakeQuantity,
    },
    DelegateCCS {
        delegatee_public: Public,
        quantity: StakeQuantity,
    },
    Revoke {
        delegatee_public: Public,
        quantity: StakeQuantity,
    },
    Redelegate {
        prev_delegatee: Public,
        next_delegatee: Public,
        quantity: StakeQuantity,
    },
    SelfNominate {
        deposit: DepositQuantity,
        metadata: Bytes,
    },
    ChangeParams {
        metadata_seq: u64,
        params: Params,
        approvals: Vec<Approval>,
    },
    ReportDoubleVote {
        message1: Bytes,
        message2: Bytes,
    },
}

pub enum AutoAction {
    UpdateValidators {
        validators: NextValidators,
    },
    CloseTerm {
        inactive_validators: Vec<Public>,
        next_validators: NextValidators,
        released_addresses: Vec<Public>,
        custody_until: u64,
        kick_at: u64,
    },
    Elect,
    ChangeNextValidators {
        validators: Vec<Validator>,
    },
}

impl UserAction {
    pub fn min_fee(&self) -> u64 {
        // Where can we initialize the min fee
        // We need both consensus-defined minimum fee and machine-defined minimum fee
        unimplemented!()
    }
}
