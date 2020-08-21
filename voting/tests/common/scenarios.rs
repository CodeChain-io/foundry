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

use super::mock_coordinator::Context;
use ccrypto::blake256;
use ckey::{Ed25519KeyPair, Generator, KeyPairTrait, Random};
use ckey::{Ed25519Private as Private, Ed25519Public as Public, Signature};
use coordinator::context::SubStorageAccess;
use coordinator::{Header, Transaction};
use ctypes::BlockHash;
use parking_lot::RwLock;
use primitives::{Bytes, H256};
use remote_trait_object::{Service, ServiceRef};
use std::borrow::ToOwned;
use std::collections::HashMap;
use vote::common::*;
use vote::general_meeting::{GeneralMeetingId, TimeStamp, TxResult};
use vote::voting::{TxCreateVotePaper, VoteChoice};

#[derive(Default)]
pub struct MockDb {
    map: HashMap<H256, Vec<u8>>,
}

impl Service for MockDb {}

impl SubStorageAccess for MockDb {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.map.get(&blake256(key)).cloned()
    }

    fn set(&mut self, key: &[u8], value: Vec<u8>) {
        self.map.insert(blake256(key), value);
    }

    fn remove(&mut self, key: &[u8]) {
        self.map.remove(&blake256(key));
    }

    fn has(&self, key: &[u8]) -> bool {
        self.map.get(&blake256(key)).is_some()
    }

    fn create_checkpoint(&mut self) {
        unimplemented!()
    }

    fn discard_checkpoint(&mut self) {
        unimplemented!()
    }

    fn revert_to_the_checkpoint(&mut self) {
        unimplemented!()
    }
}

fn tx_create_general_meeting(
    public: &Public,
    private: &Private,
    number_of_agendas: u32,
    end_time: u64,
    tallying_time: u64,
) -> Transaction {
    let tx = vote::general_meeting::TxCreateGeneralMeeting {
        number_of_agendas,
        voting_end_time: TimeStamp {
            time: end_time,
        },
        tallying_time: TimeStamp {
            time: tallying_time,
        },
    };
    let tx = UserTransaction {
        network_id: Default::default(),
        action: tx,
    };
    let tx_hash = tx.hash();
    let tx = SignedTransaction {
        signature: ckey::sign(&tx_hash.as_bytes(), private),
        signer_public: *public,
        tx,
    };
    Transaction::new("CreateGeneralMeeting".to_owned(), serde_cbor::to_vec(&tx).unwrap())
}

fn tx_vote_paper(
    public: &Public,
    private: &Private,
    general_meeting_id: H256,
    agenda_number: u32,
    number_of_shares: u32,
    voter_name: String,
    voter_public_key: Public,
) -> Transaction {
    let tx = TxCreateVotePaper {
        general_meeting_id: GeneralMeetingId {
            id: general_meeting_id,
        },
        agenda_number,
        number_of_shares,
        voter_name,
        voter_public_key,
    };
    let tx = UserTransaction {
        network_id: Default::default(),
        action: tx,
    };
    let tx_hash = tx.hash();
    let tx = SignedTransaction {
        signature: ckey::sign(&tx_hash.as_bytes(), private),
        signer_public: *public,
        tx,
    };
    Transaction::new("Create_Vote_Paper".to_owned(), serde_cbor::to_vec(&tx).unwrap())
}

fn tx_vote(vote_paper_id: H256, choice: VoteChoice, voter_signature: Signature) -> Transaction {
    let tx = vote::voting::TxVote {
        vote_paper_id,
        choice,
        voter_signature,
    };
    let tx = UserTransaction {
        network_id: Default::default(),
        action: tx,
    };
    Transaction::new("Vote".to_owned(), serde_cbor::to_vec(&tx).unwrap())
}

pub fn test_create_meeting(ctx: &RwLock<Context>) {
    for stateful in ctx.write().statefuls.values_mut() {
        stateful.set_storage(ServiceRef::create_export(Box::new(MockDb::default()) as Box<dyn SubStorageAccess>))
    }

    let admin: Ed25519KeyPair = Random.generate().unwrap();
    let fake_admin: Ed25519KeyPair = Random.generate().unwrap();

    let admin_key = admin.public();

    ctx.write()
        .init_genesises
        .get_mut("general_meeting")
        .unwrap()
        .init_genesis(&serde_cbor::to_vec(&admin_key).unwrap());

    let create_meeting = tx_create_general_meeting(&admin.public(), &admin.private(), 2, 12, 18);
    let create_meeting_invalid = tx_create_general_meeting(fake_admin.public(), fake_admin.private(), 2, 12, 18);

    let create_meeting_invalid_end_time = tx_create_general_meeting(&admin.public(), &admin.private(), 2, 19, 18);
    let create_meeting_end_passed = tx_create_general_meeting(&admin.public(), &admin.private(), 2, 0, 18);
    let create_meeting_tallying_passed = tx_create_general_meeting(&admin.public(), &admin.private(), 2, 0, 0);

    let block_hash: BlockHash = BlockHash::default();
    let author: Public = Public::default();
    let validators: Vec<Public> = Vec::new();
    let extera_data: Bytes = Bytes::default();

    let header = Header::new(block_hash, 1, 0, author, validators, extera_data);
    ctx.write().tx_owners.get_mut("general_meeting").unwrap().block_opened(&header).unwrap();
    ctx.write().tx_owners.get_mut("general_meeting").unwrap().execute_transaction(&create_meeting).unwrap();

    //FIXME: The actual returned error should be checked.
    let invalid_admin_execution =
        ctx.write().tx_owners.get_mut("general_meeting").unwrap().execute_transaction(&create_meeting_invalid);
    assert!(invalid_admin_execution.is_err());

    let invalid_end_time_execution =
        ctx.write().tx_owners.get_mut("general_meeting").unwrap().execute_transaction(&create_meeting_invalid_end_time);
    assert!(invalid_end_time_execution.is_err());

    let end_passed_execution =
        ctx.write().tx_owners.get_mut("general_meeting").unwrap().execute_transaction(&create_meeting_end_passed);
    assert!(end_passed_execution.is_err());

    let tallying_passed_execution =
        ctx.write().tx_owners.get_mut("general_meeting").unwrap().execute_transaction(&create_meeting_tallying_passed);
    assert!(tallying_passed_execution.is_err());
}

pub fn test_create_vote_paper(ctx: &RwLock<Context>) {
    for stateful in ctx.write().statefuls.values_mut() {
        stateful.set_storage(ServiceRef::create_export(Box::new(MockDb::default()) as Box<dyn SubStorageAccess>))
    }

    let admin: Ed25519KeyPair = Random.generate().unwrap();
    let admin_key = admin.public();

    ctx.write()
        .init_genesises
        .get_mut("general_meeting")
        .unwrap()
        .init_genesis(&serde_cbor::to_vec(&admin_key).unwrap());

    let create_meeting = tx_create_general_meeting(&admin.public(), &admin.private(), 2, 12, 18);

    let block_hash: BlockHash = BlockHash::default();
    let author: Public = Public::default();
    let validators: Vec<Public> = Vec::new();
    let extera_data: Bytes = Bytes::default();

    let header = Header::new(block_hash, 1, 0, author, validators.clone(), extera_data.clone());
    ctx.write().tx_owners.get_mut("general_meeting").unwrap().block_opened(&header).unwrap();

    let creat_meeting_outcome =
        ctx.write().tx_owners.get_mut("general_meeting").unwrap().execute_transaction(&create_meeting).unwrap();
    let meeting_id: H256 = serde_cbor::from_slice(&creat_meeting_outcome.events.get(0).unwrap().value).unwrap();

    let voter1: Ed25519KeyPair = Random.generate().unwrap();
    let header = Header::new(block_hash, 1, 0, author, validators.clone(), extera_data.clone());
    ctx.write().tx_owners.get_mut("voting").unwrap().block_opened(&header).unwrap();

    let create_vote_paper = tx_vote_paper(
        voter1.public(),
        voter1.private(),
        meeting_id,
        2,
        10,
        String::from("CodeChain"),
        *voter1.public(),
    );
    let transaction_execution =
        ctx.write().tx_owners.get_mut("voting").unwrap().execute_transaction(&create_vote_paper);
    assert!(transaction_execution.is_ok());

    // Number of agendas are bigger than the meeting number of agendas. Should return and error.
    //FIXME: The actual returned error should be checked.
    let voter2: Ed25519KeyPair = Random.generate().unwrap();
    let meeting_id2: H256 = serde_cbor::from_slice(&creat_meeting_outcome.events.get(0).unwrap().value).unwrap();
    let create_vote_paper = tx_vote_paper(
        voter2.public(),
        voter2.private(),
        meeting_id2,
        4,
        10,
        String::from("CodeChain"),
        *voter1.public(),
    );
    let transaction_execution =
        ctx.write().tx_owners.get_mut("voting").unwrap().execute_transaction(&create_vote_paper);
    assert!(transaction_execution.is_err());

    //FIXME: The rest of for vote_papaer should be done after coordinator full implementation.
}

pub fn test_vote(ctx: &RwLock<Context>) {
    for stateful in ctx.write().statefuls.values_mut() {
        stateful.set_storage(ServiceRef::create_export(Box::new(MockDb::default()) as Box<dyn SubStorageAccess>))
    }

    let admin: Ed25519KeyPair = Random.generate().unwrap();
    let admin_key = admin.public();

    ctx.write()
        .init_genesises
        .get_mut("general_meeting")
        .unwrap()
        .init_genesis(&serde_cbor::to_vec(&admin_key).unwrap());

    let create_meeting = tx_create_general_meeting(&admin.public(), &admin.private(), 2, 12, 18);

    let block_hash: BlockHash = BlockHash::default();
    let author: Public = Public::default();
    let validators: Vec<Public> = Vec::new();
    let extera_data: Bytes = Bytes::default();

    let header = Header::new(block_hash, 1, 0, author, validators.clone(), extera_data.clone());
    ctx.write().tx_owners.get_mut("general_meeting").unwrap().block_opened(&header).unwrap();

    let creat_meeting_outcome =
        ctx.write().tx_owners.get_mut("general_meeting").unwrap().execute_transaction(&create_meeting).unwrap();
    let meeting_id: H256 = serde_cbor::from_slice(&creat_meeting_outcome.events.get(0).unwrap().value).unwrap();

    let voter1: Ed25519KeyPair = Random.generate().unwrap();
    let header = Header::new(block_hash, 1, 0, author, validators.clone(), extera_data.clone());
    ctx.write().tx_owners.get_mut("voting").unwrap().block_opened(&header).unwrap();

    let create_vote_paper = tx_vote_paper(
        voter1.public(),
        voter1.private(),
        meeting_id,
        2,
        10,
        String::from("CodeChain"),
        *voter1.public(),
    );
    let vote_paper_execution =
        ctx.write().tx_owners.get_mut("voting").unwrap().execute_transaction(&create_vote_paper).unwrap();

    let vote_paper_id: H256 = serde_cbor::from_slice(&vote_paper_execution.events.get(0).unwrap().value).unwrap();
    let signature = ckey::sign("vote".as_ref(), voter1.private());
    let choice = VoteChoice::Favor;
    let vote = tx_vote(vote_paper_id, choice.clone(), signature);

    let vote_transaction = ctx.write().tx_owners.get_mut("voting").unwrap().execute_transaction(&vote);

    assert!(vote_transaction.is_ok());
    //FIXME: The actual returned error should be checked.
    /// This will test the error of `UsedVotePaper` for vote transaction.
    let same_vote_paper_id: H256 = serde_cbor::from_slice(&vote_paper_execution.events.get(0).unwrap().value).unwrap();
    let vote = tx_vote(same_vote_paper_id, choice.clone(), signature);

    let used_vote_transaction = ctx.write().tx_owners.get_mut("voting").unwrap().execute_transaction(&vote);
    assert!(used_vote_transaction.is_err());

    //FIXME: `InvalidVoterSignature` should be checked after it is fully implemented.
}
