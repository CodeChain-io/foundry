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

use crate::common::Action;
use crate::general_meeting::{GeneralMeeting, GeneralMeetingId, GeneralMeetingManager};
pub use ckey::{Ed25519Public as Public, Signature};
use coordinator::context::SubStorageAccess;
use coordinator::module::*;
use coordinator::types::*;
use foundry_module_rt::UserModule;
use parking_lot::RwLock;
use primitives::H256;
use rand::Rng;
use remote_trait_object::raw_exchange::{import_service_from_handle, HandleToExchange, Skeleton};
use remote_trait_object::{Context as RtoContext, Service, ServiceRef};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const VOTE_TX_TYPE: &str = "Vote";
const VOTE_PAPER_TX_TYPE: &str = "Vote_Paper";

enum ExecuteError {
    InvalidMetadata,
    InvalidSign,
    InvalidFormat,
    InvalidMeeting,
    LateVoting,
    VoteAfterTallying,
    InvalidNumberofAgendas,
    InvalidChoice,
    InvalidVotePaper,
    UsedVotePaper,
    WrongVoter,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct VoteId {
    id: H256,
}

type StorageKeyRef = [u8];

impl AsRef<StorageKeyRef> for VoteId {
    fn as_ref(&self) -> &StorageKeyRef {
        self.id.0.as_ref()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct VotePaper {
    vote_paper_id: H256,
    general_meeting_id: GeneralMeetingId,
    agenda_number: u32,
    voter_name: String,
    number_of_shares: u32,
    used_in: Option<VoteId>,
    voter_publickey: Public,
}

impl VotePaper {
    pub fn new(
        id: GeneralMeetingId,
        agenda_number: u32,
        name: String,
        number_of_shares: u32,
        voter_publickey: Public,
    ) -> Self {
        let mut rng = rand::thread_rng();
        let random_id: u64 = rng.gen();
        let vote_paper_id = H256::from(random_id);

        Self {
            vote_paper_id,
            general_meeting_id: id,
            agenda_number,
            voter_name: name,
            number_of_shares,
            used_in: None,
            voter_publickey,
        }
    }

    pub fn set_used_in(&mut self, vote_id: VoteId) -> Result<(), ExecuteError> {
        if self.used_in.is_none() {
            return Err(ExecuteError::UsedVotePaper)
        }
        self.used_in = Some(vote_id);
        Ok(())
    }

    pub fn is_used_vote_paper(&mut self) -> bool {
        self.used_in.is_some()
    }

    pub fn get_general_meeting_id(&self) -> GeneralMeetingId {
        self.general_meeting_id.clone()
    }

    //FIXME: verification should be chekced based on the third-party signature algorithm.
    pub fn verify_signature(&self, _signature: &Signature) -> bool {
        return true
    }
}

const FAVOR: &str = "favor";
const AGAINST: &str = "against";
const ABSENTION: &str = "absention";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Vote {
    vote_id: VoteId,
    choice: String,
    signature: Signature,
}

impl Vote {
    pub fn new(choice: String, signature: Signature) -> Self {
        let mut rng = rand::thread_rng();
        let random_id: u64 = rng.gen();
        let vote_id = VoteId {
            id: H256::from(random_id),
        };
        Self {
            vote_id,
            choice,
            signature,
        }
    }
}

struct Context {
    pub storage: Option<Box<dyn SubStorageAccess>>,
    pub general_meeting: Option<Box<dyn GeneralMeetingManager>>,
    pub block_header: Option<Header>,
}

impl Context {
    fn storage(&self) -> &dyn SubStorageAccess {
        self.storage.as_ref().unwrap().as_ref()
    }

    fn storage_mut(&mut self) -> &mut dyn SubStorageAccess {
        self.storage.as_mut().unwrap().as_mut()
    }

    fn _meeting(&self) -> &dyn GeneralMeetingManager {
        self.general_meeting.as_ref().unwrap().as_ref()
    }

    fn meeting_mut(&mut self) -> &mut dyn GeneralMeetingManager {
        self.general_meeting.as_mut().unwrap().as_mut()
    }
}

impl Service for Context {}

#[derive(Serialize, Deserialize, Debug)]
pub struct TxCreateVotePaper {
    general_meeting_id: GeneralMeetingId,
    agenda_number: u32,
    number_of_shares: u32,
    voter_name: String,
    voter_public_key: Public,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TxVote {
    vote_paper_id: H256,
    choice: String,
    voter_signature: Signature,
}

impl Action for TxCreateVotePaper {}
impl Action for TxVote {}
pub type PaperOwnTransaction = crate::common::SignedTransaction<TxCreateVotePaper>;
pub type VoteOwnTransaction = crate::common::PublicTransaction<TxVote>;

impl Context {
    fn excute_tx(&mut self, transaction: &Transaction) -> Result<H256, ExecuteError> {
        match transaction.tx_type() {
            VOTE_PAPER_TX_TYPE => {
                let tx: PaperOwnTransaction =
                    serde_cbor::from_slice(&transaction.body()).map_err(|_| ExecuteError::InvalidFormat)?;
                tx.verify().map_err(|_| ExecuteError::InvalidSign)?;

                let meeting_id = tx.tx.action.general_meeting_id;
                let name = tx.tx.action.voter_name;
                let shares = tx.tx.action.number_of_shares;
                let agendas = tx.tx.action.agenda_number;
                let voter_pub_key = tx.tx.action.voter_public_key;

                let meeting: GeneralMeeting =
                    self.meeting_mut().get_meeting(&meeting_id).map_err(|_| ExecuteError::InvalidMeeting)?;
                if meeting.tallying_time.time < self.block_header.as_ref().unwrap().timestamp() {
                    return Err(ExecuteError::VoteAfterTallying)
                }
                if 0 == agendas && agendas > meeting.number_of_agendas {
                    return Err(ExecuteError::InvalidNumberofAgendas)
                }
                if meeting.voting_end_time.time > self.block_header.as_ref().unwrap().timestamp() {
                    return Err(ExecuteError::LateVoting)
                }
                let vote_paper = VotePaper::new(meeting_id.clone(), agendas, name, shares, voter_pub_key);
                self.storage_mut().set(vote_paper.vote_paper_id.0.as_ref(), serde_cbor::to_vec(&vote_paper).unwrap());
                Ok(vote_paper.vote_paper_id)
            }
            VOTE_TX_TYPE => {
                let tx: VoteOwnTransaction =
                    serde_cbor::from_slice(&transaction.body()).map_err(|_| ExecuteError::InvalidFormat)?;
                let vote_paper_id = tx.action.vote_paper_id;
                let choice = tx.action.choice;
                let voter_signature = tx.action.voter_signature;
                if choice != FAVOR || choice != AGAINST || choice != ABSENTION {
                    return Err(ExecuteError::InvalidChoice)
                }
                let mut vote_paper: VotePaper =
                    serde_cbor::from_slice(&self.storage().get(vote_paper_id.0.as_ref()).unwrap())
                        .map_err(|_| ExecuteError::InvalidVotePaper)?;

                if !vote_paper.verify_signature(&voter_signature) {
                    return Err(ExecuteError::WrongVoter)
                }

                let vote = Vote::new(choice, voter_signature);
                let vote_id = vote.clone().vote_id;
                vote_paper.set_used_in(vote_id.clone()).map_err(|_| ExecuteError::UsedVotePaper)?;
                self.storage_mut().set(vote_id.as_ref(), serde_cbor::to_vec(&vote).unwrap());
                Ok(vote_id.id)
            }
            _ => return Err(ExecuteError::InvalidMetadata),
        }
    }
}

impl Stateful for Context {
    fn set_storage(&mut self, storage: ServiceRef<dyn SubStorageAccess>) {
        self.storage.replace(storage.unwrap_import().into_remote());
    }
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
                ExecuteError::UsedVotePaper => Err(()),
                ExecuteError::InvalidNumberofAgendas => Err(()),
                ExecuteError::InvalidMeeting => Err(()),
                ExecuteError::VoteAfterTallying => Err(()),
                ExecuteError::LateVoting => Err(()),
                ExecuteError::InvalidChoice => Err(()),
                ExecuteError::InvalidVotePaper => Err(()),
                ExecuteError::WrongVoter => Err(()),
            }
        } else {
            Ok(Default::default())
        }
    }

    fn check_transaction(&self, transaction: &Transaction) -> Result<(), coordinator::types::ErrorCode> {
        let todo_fixthis: coordinator::types::ErrorCode = 3;
        match transaction.tx_type() {
            VOTE_PAPER_TX_TYPE => {
                let tx: PaperOwnTransaction = serde_cbor::from_slice(&transaction.body()).map_err(|_| todo_fixthis)?;
                tx.verify().map_err(|_| todo_fixthis)?;
                let meeting_id = tx.tx.action.general_meeting_id;
                let agendas = tx.tx.action.agenda_number;

                let meeting: GeneralMeeting = serde_cbor::from_slice(&self.storage().get(meeting_id.as_ref()).unwrap())
                    .map_err(|_| todo_fixthis)?;
                if 0 == agendas && agendas > meeting.number_of_agendas {
                    return Err(ExecuteError::InvalidNumberofAgendas).map_err(|_| todo_fixthis)
                }
                Ok(())
            }
            VOTE_TX_TYPE => {
                let tx: VoteOwnTransaction = serde_cbor::from_slice(&transaction.body()).map_err(|_| todo_fixthis)?;
                let vote_paper_id = tx.action.vote_paper_id;
                let choice = tx.action.choice;
                let voter_signature = tx.action.voter_signature;

                if choice != FAVOR || choice != AGAINST || choice != ABSENTION {
                    return Err(ExecuteError::InvalidChoice).map_err(|_| todo_fixthis)
                }
                let mut vote_paper: VotePaper =
                    serde_cbor::from_slice(&self.storage().get(vote_paper_id.0.as_ref()).unwrap())
                        .map_err(|_| ExecuteError::InvalidVotePaper)
                        .map_err(|_| todo_fixthis)?;
                if vote_paper.is_used_vote_paper() {
                    return Err(ExecuteError::UsedVotePaper).map_err(|_| todo_fixthis)
                }
                if !vote_paper.verify_signature(&voter_signature) {
                    return Err(ExecuteError::WrongVoter).map_err(|_| todo_fixthis)
                }
                Ok(())
            }

            _ => return Err(ExecuteError::InvalidMetadata).map_err(|_| todo_fixthis),
        }
    }

    fn block_closed(&mut self) -> Result<Vec<Event>, CloseBlockError> {
        Ok(Vec::new())
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
                general_meeting: None,
                block_header: None,
            })),
        }
    }

    fn prepare_service_to_export(&mut self, ctor_name: &str, ctor_arg: &[u8]) -> Skeleton {
        match ctor_name {
            "tx_owner" => {
                let arg: String = serde_cbor::from_slice(ctor_arg).unwrap();
                assert_eq!(arg, "unused");
                Skeleton::new(Arc::clone(&self.ctx) as Arc<RwLock<dyn TxOwner>>)
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
            "meeting_manager" => {
                self.ctx.write().general_meeting.replace(import_service_from_handle(rto_context, handle));
            }
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
