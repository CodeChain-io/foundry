// Copyright 2020 Kodebox, Inc.
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

use crate::chain_history_manager;
use crate::state::{Jail, Metadata, NextValidators, Params};
use crate::types::{Approval, DepositQuantity, NetworkId, StakeQuantity, Validator};
use ccrypto::blake256;
use coordinator::Header;
use fkey::{verify, Ed25519Public as Public, Signature};
use primitives::{Bytes, H256};
use std::collections::HashSet;

pub enum Transaction {
    #[allow(dead_code)]
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

pub fn create_close_block_transactions(current_header: &Header) -> Vec<Transaction> {
    let chain_history = chain_history_manager();
    let parent_hash = current_header.parent_hash();
    let parent_header = chain_history.get_block_header(parent_hash.clone().into()).expect("parent header must exist");
    let parent_metadata = Metadata::load_from(parent_hash.clone().into()).expect("parent metadata must exist");
    let metadata = Metadata::load();
    let term = metadata.current_term_id;
    let term_seconds = match term {
        0 => parent_metadata.params.term_seconds,
        _ => parent_metadata.term_params.term_seconds,
    };

    let mut next_validators = NextValidators::load();
    next_validators.update_weight(current_header.author());

    if is_term_close(current_header, &parent_header, term_seconds) {
        vec![Transaction::Auto(AutoAction::ChangeNextValidators {
            validators: next_validators.into(),
        })]
    } else {
        let inactive_validators = match term {
            0 => Vec::new(),
            _ => {
                let start_of_the_current_term = metadata.last_term_finished_block_num + 1;
                let validators = next_validators.iter().map(|val| val.pubkey).collect();
                inactive_validators(current_header, start_of_the_current_term, validators)
            }
        };
        let current_term_id = metadata.current_term_id;
        let (custody_until, kick_at) = {
            let params = metadata.params;
            let custody_period = params.custody_period;
            assert_ne!(0, custody_period);
            let release_period = params.release_period;
            assert_ne!(0, release_period);
            (current_term_id + custody_period, current_term_id + release_period)
        };
        let released_addresses = Jail::load()
            .drain_released_prisoners(current_term_id)
            .into_iter()
            .map(|prisoner| prisoner.pubkey)
            .collect();
        vec![
            Transaction::Auto(AutoAction::CloseTerm {
                inactive_validators,
                next_validators,
                released_addresses,
                custody_until,
                kick_at,
            }),
            Transaction::Auto(AutoAction::Elect {}),
        ]
    }
}

fn is_term_close(header: &Header, parent: &Header, term_seconds: u64) -> bool {
    // Because the genesis block has a fixed generation time, the first block should not change the term.
    if header.number() == 1 {
        return false
    }
    if term_seconds == 0 {
        return false
    }

    header.timestamp() / term_seconds != parent.timestamp() / term_seconds
}

fn inactive_validators(
    current_header: &Header,
    start_of_the_current_term: u64,
    mut validators: HashSet<Public>,
) -> Vec<Public> {
    let chain_history = chain_history_manager();
    validators.remove(current_header.author());
    let hash = *current_header.parent_hash();
    let mut header = chain_history.get_block_header(hash.into()).expect("Header of the parent must exist");
    while start_of_the_current_term <= header.number() {
        validators.remove(&header.author());
        header =
            chain_history.get_block_header((*header.parent_hash()).into()).expect("Header of the parent must exist");
    }

    validators.into_iter().collect()
}

pub fn create_open_block_transactions() -> Vec<Transaction> {
    vec![Transaction::Auto(AutoAction::UpdateValidators {
        validators: NextValidators::load(),
    })]
}
