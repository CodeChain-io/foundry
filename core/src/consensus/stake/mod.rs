// Copyright 2018-2020 Kodebox, Inc.
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

use crate::client::ConsensusClient;
use ckey::{public_to_address, Address};
use cstate::{ban, DoubleVoteHandler, StateResult, TopLevelState};
use ctypes::errors::{RuntimeError, SyntaxError};
use parking_lot::RwLock;
use std::sync::{Arc, Weak};

use super::ValidatorSet;
use crate::consensus::ConsensusMessage;

#[derive(Default)]
pub struct Stake {
    client: RwLock<Option<Weak<dyn ConsensusClient>>>,
    validators: RwLock<Option<Weak<dyn ValidatorSet>>>,
}

impl Stake {
    pub fn register_resources(&self, client: Weak<dyn ConsensusClient>, validators: Weak<dyn ValidatorSet>) {
        *self.client.write() = Some(Weak::clone(&client));
        *self.validators.write() = Some(Weak::clone(&validators));
    }
}

impl DoubleVoteHandler for Stake {
    fn execute(&self, message1: &[u8], state: &mut TopLevelState, sender_address: &Address) -> StateResult<()> {
        let message1: ConsensusMessage =
            rlp::decode(message1).map_err(|err| RuntimeError::FailedToHandleCustomAction(err.to_string()))?;
        let validators =
            self.validators.read().as_ref().and_then(Weak::upgrade).expect("ValidatorSet must be initialized");
        let client = self.client.read().as_ref().and_then(Weak::upgrade).expect("Client must be initialized");

        execute_report_double_vote(message1, state, sender_address, &*client, &*validators)?;
        Ok(())
    }

    fn verify(&self, message1: &[u8], message2: &[u8]) -> Result<(), SyntaxError> {
        let client: Arc<dyn ConsensusClient> =
            self.client.read().as_ref().and_then(Weak::upgrade).expect("Client should be initialized");
        let validators: Arc<dyn ValidatorSet> =
            self.validators.read().as_ref().and_then(Weak::upgrade).expect("ValidatorSet should be initialized");

        let message1: ConsensusMessage =
            rlp::decode(message1).map_err(|err| SyntaxError::InvalidCustomAction(err.to_string()))?;
        let message2: ConsensusMessage =
            rlp::decode(message2).map_err(|err| SyntaxError::InvalidCustomAction(err.to_string()))?;

        verify_report_double_vote(message1, message2, &*client, &*validators)
    }
}

fn execute_report_double_vote(
    message1: ConsensusMessage,
    state: &mut TopLevelState,
    sender_address: &Address,
    client: &dyn ConsensusClient,
    validators: &dyn ValidatorSet,
) -> StateResult<()> {
    let parent_hash = client.block_header(&(message1.height() - 1).into()).expect("Parent header verified").hash();
    let malicious_user_public = validators.get(&parent_hash, message1.signer_index());

    ban(state, sender_address, public_to_address(&malicious_user_public))
}

pub fn verify_report_double_vote(
    message1: ConsensusMessage,
    message2: ConsensusMessage,
    client: &dyn ConsensusClient,
    validators: &dyn ValidatorSet,
) -> Result<(), SyntaxError> {
    if message1.round() != message2.round() {
        return Err(SyntaxError::InvalidCustomAction(String::from("The messages are from two different voting rounds")))
    }

    let signer_idx1 = message1.signer_index();
    let signer_idx2 = message2.signer_index();

    if signer_idx1 != signer_idx2 {
        return Err(SyntaxError::InvalidCustomAction(format!(
            "Two messages have different signer indexes: {}, {}",
            signer_idx1, signer_idx2
        )))
    }

    assert_eq!(
        message1.height(),
        message2.height(),
        "Heights of both messages must be same because message1.round() == message2.round()"
    );

    let signed_block_height = message1.height();
    if signed_block_height == 0 {
        return Err(SyntaxError::InvalidCustomAction(String::from(
            "Double vote on the genesis block does not make sense",
        )))
    }
    let parent_hash = client
        .block_header(&(signed_block_height - 1).into())
        .ok_or_else(|| {
            SyntaxError::InvalidCustomAction(format!("Cannot get header from the height {}", signed_block_height))
        })?
        .hash();
    let signer_idx1 = message1.signer_index();
    let signer = validators.get(&parent_hash, signer_idx1);
    if !message1.verify(&signer) || !message2.verify(&signer) {
        return Err(SyntaxError::InvalidCustomAction(String::from("Ed25519 signature verification fails")))
    }
    Ok(())
}
