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
use vote::general_meeting::{GeneralMeetingId, TimeStamp};

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
