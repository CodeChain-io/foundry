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

use crate::common::*;
use crate::voting::VoteId;
pub use ckey::Ed25519Public as Public;
use coordinator::context::SubStorageAccess;
use coordinator::module::*;
use coordinator::types::*;
pub use ctypes::BlockId;
use foundry_module_rt::UserModule;
use parking_lot::RwLock;
use primitives::H256;
use rand::Rng;
use remote_trait_object::raw_exchange::{import_service_from_handle, HandleToExchange, Skeleton};
use remote_trait_object::{service, Context as RtoContext, Service, ServiceRef};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

enum ExecuteError {
    InvalidMetadata,
    InvalidSign,
    InvalidFormat,
    InvalidAdmin,
}

const ADMIN_STATE_KEY: &str = "admin";
const TX_TYPE: &str = "GeneralMeeting";

struct Context {
    pub storage: Option<Box<dyn SubStorageAccess>>,
}

impl Context {
    fn storage(&self) -> &dyn SubStorageAccess {
        self.storage.as_ref().unwrap().as_ref()
    }

    fn storage_mut(&mut self) -> &mut dyn SubStorageAccess {
        self.storage.as_mut().unwrap().as_mut()
    }
}

impl Service for Context {}

#[derive(Serialize, Deserialize, Debug)]
pub struct TxGeneralMeeting {
    pub number_of_agendas: u32,
    pub end_time: TimeStamp,
    pub tallying_time: TimeStamp,
}

impl Action for TxGeneralMeeting {}
pub type OwnTransaction = crate::common::SignedTransaction<TxGeneralMeeting>;

impl Context {
    fn excute_tx(&mut self, transaction: &Transaction) -> Result<GeneralMeetingId, ExecuteError> {
        if transaction.tx_type() != TX_TYPE {
            return Err(ExecuteError::InvalidMetadata)
        }
        let tx: OwnTransaction =
            serde_cbor::from_slice(&transaction.body()).map_err(|_| ExecuteError::InvalidFormat)?;
        tx.verify().map_err(|_| ExecuteError::InvalidSign)?;

        let num_agandas = tx.tx.action.number_of_agendas;
        let end_time = tx.tx.action.end_time;
        let tallying_time = tx.tx.action.tallying_time;
        let meeting = GeneralMeeting::new(end_time, tallying_time, num_agandas);
        let key = meeting.id.clone();

        self.storage_mut().set(key.as_ref(), serde_cbor::to_vec(&meeting).unwrap());

        let vote_box = VoteBox::new(meeting.id.clone());
        let box_key = crate::generate_voting_box_key(&meeting.id);
        self.storage_mut().set(box_key.as_slice(), serde_cbor::to_vec(&vote_box).unwrap());
        Ok(meeting.id)
    }
}

impl Stateful for Context {
    fn set_storage(&mut self, storage: ServiceRef<dyn SubStorageAccess>) {
        self.storage.replace(storage.unwrap_import().into_remote());
    }
}

impl InitGenesis for Context {
    fn begin_genesis(&mut self) {}

    fn init_genesis(&mut self, config: &[u8]) {
        let admin: Public = serde_cbor::from_slice(&config).unwrap();
        self.storage_mut().set(ADMIN_STATE_KEY.as_bytes(), serde_cbor::to_vec(&admin).unwrap());
    }

    fn end_genesis(&mut self) {}
}

impl TxOwner for Context {
    fn block_opened(&mut self, _: &Header) -> Result<(), HeaderError> {
        Ok(())
    }

    fn execute_transaction(&mut self, transaction: &Transaction) -> Result<TransactionOutcome, ()> {
        if let Err(error) = self.excute_tx(transaction) {
            match error {
                ExecuteError::InvalidMetadata => Err(()),
                ExecuteError::InvalidSign => Err(()),
                ExecuteError::InvalidFormat => Err(()),
                ExecuteError::InvalidAdmin => Err(()),
            }
        } else {
            Ok(Default::default())
        }
    }

    fn check_transaction(&self, transaction: &Transaction) -> Result<(), coordinator::types::ErrorCode> {
        let todo_fixthis: coordinator::types::ErrorCode = 3;
        assert_eq!(transaction.tx_type(), TX_TYPE);
        let tx: OwnTransaction = serde_cbor::from_slice(&transaction.body()).map_err(|_| todo_fixthis)?;
        tx.verify().map_err(|_| todo_fixthis)?;

        let admin_public_key = tx.signer_public;
        let written_admin: Public =
            serde_cbor::from_slice(&self.storage().get(ADMIN_STATE_KEY.as_bytes()).unwrap()).unwrap();

        if written_admin != admin_public_key {
            return Err(ExecuteError::InvalidAdmin).map_err(|_| todo_fixthis)
        }
        Ok(())
    }

    fn block_closed(&mut self) -> Result<Vec<Event>, CloseBlockError> {
        Ok(Vec::new())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TimeStamp {
    pub time: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(transparent)]
pub struct GeneralMeetingId {
    pub id: H256,
}

impl GeneralMeetingId {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let random_id: u64 = rng.gen();
        let meeting_id = H256::from(random_id);
        Self {
            id: meeting_id,
        }
    }
}

type StorageKeyRef = [u8];

impl AsRef<StorageKeyRef> for GeneralMeetingId {
    fn as_ref(&self) -> &StorageKeyRef {
        &self.id.0.as_ref()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Error {
    InvalidMeeting,
}

#[service]
pub trait GeneralMeetingManager: Service {
    fn get_meeting(&mut self, meeting_id: &GeneralMeetingId) -> Result<GeneralMeeting, Error>;
    fn get_box(&mut self, key: &[u8]) -> Result<VoteBox, Error>;
}

impl GeneralMeetingManager for Context {
    fn get_meeting(&mut self, meeting_id: &GeneralMeetingId) -> Result<GeneralMeeting, Error> {
        if !self.storage().has(meeting_id.as_ref()) {
            return Err(Error::InvalidMeeting)
        }
        let meeting: GeneralMeeting =
            serde_cbor::from_slice(&self.storage().get(meeting_id.as_ref()).unwrap()).unwrap();
        Ok(meeting)
    }

    fn get_box(&mut self, key: &[u8]) -> Result<VoteBox, Error> {
        if !self.storage().has(key) {
            return Err(Error::InvalidMeeting)
        }

        Ok(serde_cbor::from_slice(&self.storage().get(key).unwrap()).unwrap())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VoteResult {
    favor: u32,
    against: u32,
    absention: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GeneralMeetingError {
    NoSuchMeeting,
    MeetingExist,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneralMeeting {
    pub id: GeneralMeetingId,
    pub number_of_agendas: u32,
    pub voting_end_time: TimeStamp,
    pub tallying_time: TimeStamp,
    result: Option<Vec<VoteResult>>,
}

impl GeneralMeeting {
    pub fn new(end_time: TimeStamp, tallying_time: TimeStamp, number_of_agendas: u32) -> Self {
        let meeting_id = GeneralMeetingId::new();
        Self {
            id: meeting_id,
            number_of_agendas,
            voting_end_time: end_time,
            tallying_time,
            result: None,
        }
    }
}

pub struct Module {
    ctx: Arc<RwLock<Context>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VoteBox {
    pub meeting_id: GeneralMeetingId,
    pub votes: Vec<VoteId>,
}
impl VoteBox {
    pub fn new(meeting_id: GeneralMeetingId) -> Self {
        Self {
            meeting_id,
            votes: Vec::new(),
        }
    }

    pub fn drop_in_box(&mut self, vote: VoteId) {
        self.votes.push(vote);
    }
}

impl UserModule for Module {
    fn new(_arg: &[u8]) -> Self {
        Module {
            ctx: Arc::new(RwLock::new(Context {
                storage: None,
            })),
        }
    }

    fn prepare_service_to_export(&mut self, ctor_name: &str, ctor_arg: &[u8]) -> Skeleton {
        match ctor_name {
            "meeting_manager" => {
                let arg: String = serde_cbor::from_slice(ctor_arg).unwrap();
                assert_eq!(arg, "unused");
                Skeleton::new(Arc::clone(&self.ctx) as Arc<RwLock<dyn GeneralMeetingManager>>)
            }
            "tx_owner" => {
                let arg: String = serde_cbor::from_slice(ctor_arg).unwrap();
                assert_eq!(arg, "unused");
                Skeleton::new(Arc::clone(&self.ctx) as Arc<RwLock<dyn TxOwner>>)
            }
            "init_genesis" => {
                let arg: String = serde_cbor::from_slice(ctor_arg).unwrap();
                assert_eq!(arg, "unused");
                Skeleton::new(Arc::clone(&self.ctx) as Arc<RwLock<dyn InitGenesis>>)
            }
            _ => panic!("Unsupported ctor_name in prepare_service_to_export() : {}", ctor_name),
        }
    }

    fn import_service(
        &mut self,
        rto_context: &RtoContext,
        _exporter_module: &str,
        name: &str,
        handle: HandleToExchange,
    ) {
        match name {
            "sub_storage_access" => {
                self.ctx.write().storage.replace(import_service_from_handle(rto_context, handle));
            }
            _ => panic!("Invalid name in import_service()"),
        }
    }

    fn debug(&mut self, _arg: &[u8]) -> Vec<u8> {
        unimplemented!()
    }
}
