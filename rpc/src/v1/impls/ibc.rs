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

use super::super::errors;
use super::super::traits::IBC;
use super::super::types::{ClientState, ConsensusState, IBCQuery};
use ccore::ibc;
use ccore::ibc::querier;
use ccore::{BlockChainClient, BlockId, StateInfo};
use ibc::client_02::types::Header;
use jsonrpc_core::Result;
use primitives::Bytes;
use std::sync::Arc;

#[allow(dead_code)]
pub struct IBCClient<C>
where
    C: StateInfo + BlockChainClient, {
    client: Arc<C>,
}

impl<C> IBCClient<C>
where
    C: StateInfo + BlockChainClient,
{
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
        }
    }
}

impl<C> IBC for IBCClient<C>
where
    C: StateInfo + 'static + Send + Sync + BlockChainClient,
{
    fn query_client_state(
        &self,
        client_id: String,
        block_number: Option<u64>,
    ) -> Result<Option<IBCQuery<ClientState>>> {
        let block_id = block_number.map(BlockId::Number).unwrap_or(BlockId::Latest);
        if self.client.state_at(block_id).is_none() {
            return Ok(None)
        }
        let mut state = self.client.state_at(block_id).unwrap();
        let block_number = match self.client.block_number(&block_id) {
            None => return Ok(None),
            Some(block_number) => block_number,
        };

        let context = ibc::context::TopLevelContext::new(&mut state, block_number);
        let path = querier::path_client_state(&client_id);
        let client_state: Option<ibc::client_02::types::ClientState> = querier::query(&context, &path);

        Ok(Some(IBCQuery {
            number: block_number,
            data: client_state.map(|x| ClientState::from_core(&x)),
            proof: querier::make_proof(&context, &path),
        }))
    }

    fn query_consensus_state(
        &self,
        client_id: String,
        counterparty_block_number: u64,
        block_number: Option<u64>,
    ) -> Result<Option<IBCQuery<ConsensusState>>> {
        let block_id = block_number.map(BlockId::Number).unwrap_or(BlockId::Latest);
        if self.client.state_at(block_id).is_none() {
            return Ok(None)
        }
        let mut state = self.client.state_at(block_id).unwrap();
        let block_number = match self.client.block_number(&block_id) {
            None => return Ok(None),
            Some(block_number) => block_number,
        };

        let context = ibc::context::TopLevelContext::new(&mut state, block_number);
        let path = querier::path_consensus_state(&client_id, counterparty_block_number);
        let consensus_state: Option<ibc::client_02::types::ConsensusState> = querier::query(&context, &path);

        Ok(Some(IBCQuery {
            number: block_number,
            data: consensus_state.map(|x| ConsensusState::from_core(&x)),
            proof: querier::make_proof(&context, &path),
        }))
    }

    fn compose_header(&self, block_number: u64) -> Result<Option<Bytes>> {
        let block_id = BlockId::Number(block_number);
        if self.client.state_at(block_id).is_none() {
            return Ok(None)
        }
        let state = self.client.state_at(block_id).unwrap();

        let header_core = self.client.block_header(&block_id).unwrap();
        let vset_raw = ccore::stake::NextValidators::load_from_state(&state).map_err(errors::core)?;

        let vset = vset_raw.create_compact_validator_set();
        let header = Header {
            header_proposal: ccore::consensus::light_client::UpdateHeader {
                number: block_number,
                hash: *header_core.hash(),
                seal: ccore::consensus::light_client::Seal {
                    raw: header_core.seal(),
                },
                validator_set: vset,
            },
            state_root: ccore::ibc::commitment_23::CommitmentRoot {
                raw: header_core.state_root(),
            },
        };
        Ok(Some(rlp::encode(&header)))
    }
}
