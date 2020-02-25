// Copyright 2019-2020 Kodebox, Inc.
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

use self::commitment_23::{verify_membership, CommitmentPathCounter, CommitmentProofCounter};
use super::types::{ClientState, ConsensusState, Header};
use super::*;
use crate::consensus::light_client::ClientState as ChainClientState;
use crate::ctypes::BlockNumber;
use crate::ibc;
use crate::ibc::connection_03::types::ConnectionEnd;
use crate::ibc::IdentifierSlice;
use crate::rlp::Encodable;
use primitives::Bytes;
use rlp;

pub struct Manager<'a> {
    ctx: &'a mut dyn ibc::Context,
}

impl<'a> Manager<'a> {
    pub fn new(ctx: &'a mut dyn ibc::Context) -> Self {
        Manager {
            ctx,
        }
    }

    pub fn create(&mut self, id: IdentifierSlice, _consensus_state: Bytes, header: Bytes) -> Result<(), String> {
        // NOTE: create() takes counterparty chain's header and decode it by itself.
        let header_dec: crate::ctypes::Header =
            rlp::decode(&header).map_err(|_| "Failed to decode counterparty chain's header")?;

        let client = ClientState {
            raw: ChainClientState::new(&header_dec),
        };

        let kv_store = self.ctx.get_kv_store_mut();
        if kv_store.contains_key(&path_client_state(id)) {
            return Err("Client exists".to_owned())
        }
        kv_store.set(&path_client_state(id), &client.rlp_bytes());
        Ok(())
    }

    pub fn update(&mut self, id: IdentifierSlice, header: Bytes) -> Result<(), String> {
        let header_dec: Header = rlp::decode(&header).map_err(|_| "Failed to decode IBC header")?;
        let client_state = self.query(id)?;
        let (new_client_state, new_consensus_state) =
            super::client::check_validity_and_update_state(&client_state, &header_dec)?;

        let kv_store = self.ctx.get_kv_store_mut();
        kv_store.set(&path_client_state(id), &new_client_state.rlp_bytes());
        kv_store.set(&path_consensus_state(id, new_client_state.raw.number), &new_consensus_state.rlp_bytes());

        Ok(())
    }

    pub fn query(&self, id: IdentifierSlice) -> Result<ClientState, String> {
        let kv_store = self.ctx.get_kv_store();
        let data = kv_store.get(&path_client_state(id)).ok_or_else(|| "Client doesn't exist".to_owned())?;
        Ok(rlp::decode(&data).expect("Illformed client state stored in DB"))
    }

    pub fn query_consensus_state(
        &self,
        id: IdentifierSlice,
        num: ctypes::BlockNumber,
    ) -> Result<ConsensusState, String> {
        let kv_store = self.ctx.get_kv_store();
        let data =
            kv_store.get(&path_consensus_state(id, num)).ok_or_else(|| "Consensus state doesn't exist".to_owned())?;
        Ok(rlp::decode(&data).expect("Illformed consensus state stored in DB"))
    }

    /* -------- Various verifiers -------- */
    /// There are some difference from original ICS specification for these verifiers
    /// 1. They don't take a prefix as an argument: each function should decide it by itself.
    /// 2. They run on `Manager`, which has a state DB context.
    /// 3. They don't take a ClientState: each function retrieves it by itself.
    /// 4. They all take `id`, which indicates the counterparty chain.

    pub fn verify_connection_state(
        &self,
        id: IdentifierSlice,
        proof_height: BlockNumber,
        proof: Bytes,
        connection_identifier: IdentifierSlice,
        connection_end: &ConnectionEnd,
    ) -> Result<(), String> {
        let path = format!("connections/{}", connection_identifier);
        let client_state = self.query(&id)?;
        if client_state.raw.number < proof_height {
            return Err("Invalid proof height".to_owned())
        }
        let consensus_state = self.query_consensus_state(&id, proof_height)?;
        let proof_dec: CommitmentProofCounter = rlp::decode(&proof).map_err(|_| "Illformed proof")?;
        let value_dec = rlp::encode(connection_end);

        if verify_membership(
            &consensus_state.state_root,
            &proof_dec,
            CommitmentPathCounter {
                raw: path,
            },
            value_dec,
        ) {
            Ok(())
        } else {
            Err("Invalid proof".to_owned())
        }
    }
}
