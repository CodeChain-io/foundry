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

use super::types::{ClientState, ConsensusState, Header};
use super::*;
use crate::consensus::light_client::ClientState as ChainClientState;
use crate::ctypes;
use crate::ibc;
use crate::rlp::Encodable;

pub struct Manager<'a> {
    ctx: &'a mut dyn ibc::Context,
}

impl<'a> Manager<'a> {
    pub fn new(ctx: &'a mut dyn ibc::Context) -> Self {
        Manager {
            ctx,
        }
    }

    pub fn create(
        &mut self,
        id: &str,
        _consensus_state: &ConsensusState,
        // note that this header should be opposite chain's one.
        header: &ctypes::Header,
    ) -> Result<(), String> {
        let kv_store = self.ctx.get_kv_store_mut();

        let client = ClientState {
            raw: ChainClientState::new(header),
        };
        if kv_store.has(&path_client_state(id)) {
            return Err("Client exists".to_owned())
        }
        kv_store.set(&path_client_state(id), &client.rlp_bytes());
        Ok(())
    }

    pub fn update(&mut self, id: &str, header: &Header) -> Result<(), String> {
        let client_state = self.query(id)?;
        let (new_client_state, new_consensus_state) =
            super::client::check_validity_and_update_state(&client_state, header)?;

        let kv_store = self.ctx.get_kv_store_mut();
        kv_store.set(&path_client_state(id), &new_client_state.rlp_bytes());
        kv_store.set(&path_consensus_state(id, new_client_state.raw.number), &new_consensus_state.rlp_bytes());

        Ok(())
    }

    pub fn query(&self, id: &str) -> Result<ClientState, String> {
        let kv_store = self.ctx.get_kv_store();
        if !kv_store.has(&path_client_state(id)) {
            return Err("Client doesn't exist".to_owned())
        }
        let data = kv_store.get(&path_client_state(id));
        Ok(rlp::decode(&data).expect("Illformed client state stored in DB"))
    }

    pub fn query_consensus_state(&self, id: &str, num: ctypes::BlockNumber) -> Result<ConsensusState, String> {
        let kv_store = self.ctx.get_kv_store();
        if !kv_store.has(&path_consensus_state(id, num)) {
            return Err("Consensus state doesn't exist".to_owned())
        }
        let data = kv_store.get(&path_consensus_state(id, num));
        Ok(rlp::decode(&data).expect("Illformed consensus state stored in DB"))
    }
}
