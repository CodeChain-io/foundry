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
use ckey::Ed25519Public as Public;
use coordinator::context::SubStorageAccess;
use coordinator::module::*;
use coordinator::types::*;
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
    InvalidFormat,
    InvalidSignature,
    TallyingTimePassed,
    TallyingIsBeforeVotingEnd,
    VotingEndPassed,
    NotAuthorized,
}

const ADMIN_STATE_KEY: &str = "admin";
const CREATE_GENERAL_MEETING_TX_TYPE: &str = "CreateGeneralMeeting";

#[derive(Serialize, Deserialize, Debug)]
pub struct TxCreateGeneralMeeting {
    pub number_of_agendas: u32,
    pub voting_end_time: TimeStamp,
    pub tallying_time: TimeStamp,
}

impl Action for TxCreateGeneralMeeting {}
pub type CreateGeneralMeetingOwnTransaction = SignedTransaction<TxCreateGeneralMeeting>;

struct Context {
    pub storage: Option<Box<dyn SubStorageAccess>>,
    pub block_header: Option<Header>,
}

impl Context {
    fn storage(&self) -> &dyn SubStorageAccess {
        self.storage.as_ref().unwrap().as_ref()
    }

    fn storage_mut(&mut self) -> &mut dyn SubStorageAccess {
        self.storage.as_mut().unwrap().as_mut()
    }

    fn admin(&self) -> Public {
        let bytes = self
            .storage()
            .get(ADMIN_STATE_KEY.as_bytes())
            .expect("GeneralMeeting module set the admin in the genesis state");
        serde_cbor::from_slice(&bytes).expect("Admin key is saved in the GeneralMeeting module")
    }

    /// TXs are executed here in this function.
    /// Rigorous conditions are checked here for both TXs although they are checked in `check_transaction` function.
    /// check transaction could be executed not in a block context.
    /// execute_tx always be called in a block context.
    /// We can read the Header information only in `execute_tx`.
    fn excute_tx(&mut self, transaction: &Transaction) -> Result<(), ExecuteError> {
        match transaction.tx_type() {
            CREATE_GENERAL_MEETING_TX_TYPE => {
                let tx: CreateGeneralMeetingOwnTransaction =
                    serde_cbor::from_slice(&transaction.body()).map_err(|_| ExecuteError::InvalidFormat)?;
                tx.verify().map_err(|_| ExecuteError::InvalidSignature)?;

                let admin_public_key = tx.signer_public;
                let valid_admin: Public = self.admin();

                if valid_admin != admin_public_key {
                    return Err(ExecuteError::NotAuthorized)
                }

                let num_agendas = tx.tx.action.number_of_agendas;
                let voting_end_time = tx.tx.action.voting_end_time;
                let tallying_time = tx.tx.action.tallying_time;

                let now = self.block_header.as_ref().unwrap().timestamp();
                if now > voting_end_time.time {
                    return Err(ExecuteError::VotingEndPassed)
                }
                if now > tallying_time.time {
                    return Err(ExecuteError::TallyingTimePassed)
                }
                if tallying_time.time < voting_end_time.time {
                    return Err(ExecuteError::TallyingIsBeforeVotingEnd)
                }

                let meeting = GeneralMeeting::new(voting_end_time, tallying_time, num_agendas);
                let key = meeting.id.clone();

                self.storage_mut().set(key.as_ref(), serde_cbor::to_vec(&meeting).unwrap());

                let vote_box = VoteBox::new(meeting.id.clone());
                let box_key = crate::generate_voting_box_key(&meeting.id);
                self.storage_mut().set(box_key.as_slice(), serde_cbor::to_vec(&vote_box).unwrap());

                Ok(())
            }

            _ => return Err(ExecuteError::InvalidMetadata),
        }
    }
}

impl Service for Context {}

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
    fn block_opened(&mut self, header: &Header) -> Result<(), HeaderError> {
        self.block_header = Some(header.clone());
        Ok(())
    }

    fn execute_transaction(&mut self, transaction: &Transaction) -> Result<TransactionOutcome, ()> {
        if let Err(error) = self.excute_tx(transaction) {
            Err(())
        } else {
            Ok(Default::default())
        }
    }

    fn check_transaction(&self, transaction: &Transaction) -> Result<(), coordinator::types::ErrorCode> {
        let todo_fixthis: coordinator::types::ErrorCode = 3;
        match transaction.tx_type() {
            CREATE_GENERAL_MEETING_TX_TYPE => {
                let tx: CreateGeneralMeetingOwnTransaction =
                    serde_cbor::from_slice(&transaction.body()).map_err(|_| todo_fixthis)?;
                tx.verify().map_err(|_| todo_fixthis)?;

                let tallying_time = tx.tx.action.tallying_time;
                let end_time = tx.tx.action.voting_end_time;

                let admin_public_key = tx.signer_public;
                let valid_admin: Public = self.admin();

                if valid_admin != admin_public_key {
                    return Err(ExecuteError::NotAuthorized).map_err(|_| todo_fixthis)
                }
                if tallying_time.time < end_time.time {
                    return Err(ExecuteError::TallyingIsBeforeVotingEnd).map_err(|_| todo_fixthis)
                }
                Ok(())
            }
            _ => return Err(ExecuteError::InvalidMetadata).map_err(|_| todo_fixthis),
        }
    }

    fn block_closed(&mut self) -> Result<Vec<Event>, CloseBlockError> {
        self.block_header = None;
        Ok(Vec::new())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Error {
    MeetingNotFound,
    VoteBoxNotFound,
}

#[service]
pub trait GeneralMeetingManager: Service {
    fn get_meeting(&mut self, meeting_id: &GeneralMeetingId) -> Result<GeneralMeeting, Error>;
    fn get_vote_box(&mut self, meeting_id: &GeneralMeetingId) -> Result<VoteBox, Error>;
}

impl GeneralMeetingManager for Context {
    fn get_meeting(&mut self, meeting_id: &GeneralMeetingId) -> Result<GeneralMeeting, Error> {
        if !self.storage().has(meeting_id.as_ref()) {
            return Err(Error::MeetingNotFound)
        }
        let meeting: GeneralMeeting = {
            let bytes =
                self.storage().get(meeting_id.as_ref()).expect("We checked the existence in the above if statement");
            serde_cbor::from_slice(&bytes).expect("General meeting is serialized by this code")
        };
        Ok(meeting)
    }

    fn get_vote_box(&mut self, meeting_id: &GeneralMeetingId) -> Result<VoteBox, Error> {
        let key = crate::generate_voting_box_key(meeting_id);

        if !self.storage().has(key.as_slice()) {
            return Err(Error::VoteBoxNotFound)
        }
        let vote_box = {
            let bytes =
                self.storage().get(key.as_slice()).expect("We check the existence of the vote box in above statement");
            serde_cbor::from_slice(&bytes).expect("Vote Box is serialized by this code")
        };
        Ok(vote_box)
    }
}

pub struct Module {
    ctx: Arc<RwLock<Context>>,
}

impl UserModule for Module {
    fn new(_arg: &[u8]) -> Self {
        Module {
            ctx: Arc::new(RwLock::new(Context {
                storage: None,
                block_header: None,
            })),
        }
    }

    fn prepare_service_to_export(&mut self, ctor_name: &str, ctor_arg: &[u8]) -> Skeleton {
        match ctor_name {
            "general_meeting_manager" => {
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(transparent)]
pub struct GeneralMeetingId {
    id: H256,
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

    pub fn get_meeting_id(&self) -> &H256 {
        &self.id
    }
}

type StorageKeyRef = [u8];

impl AsRef<StorageKeyRef> for GeneralMeetingId {
    fn as_ref(&self) -> &StorageKeyRef {
        &self.id.0.as_ref()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TimeStamp {
    time: u64,
}

impl TimeStamp {
    pub fn get_time(&self) -> u64 {
        self.time
    }
}

impl VoteResult {
    pub fn new() -> Self {
        Self {
            favor: 0,
            against: 0,
            absention: 0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VoteResult {
    favor: u32,
    against: u32,
    absention: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneralMeeting {
    id: GeneralMeetingId,
    number_of_agendas: u32,
    voting_end_time: TimeStamp,
    tallying_time: TimeStamp,
    result: Option<Vec<VoteResult>>,
}

impl GeneralMeeting {
    pub fn new(voting_end_time: TimeStamp, tallying_time: TimeStamp, number_of_agendas: u32) -> Self {
        let meeting_id = GeneralMeetingId::new();
        Self {
            id: meeting_id,
            number_of_agendas,
            voting_end_time,
            tallying_time,
            result: None,
        }
    }

    pub fn save_results(&mut self, result: Vec<VoteResult>) {
        self.result.get_or_insert(result);
    }

    pub fn get_end_time(&self) -> &TimeStamp {
        &self.voting_end_time
    }

    pub fn get_tallying_time(&self) -> &TimeStamp {
        &self.tallying_time
    }

    pub fn get_number_of_agendas(&self) -> u32 {
        self.number_of_agendas
    }
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
