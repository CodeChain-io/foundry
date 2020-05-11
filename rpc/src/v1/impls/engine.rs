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

use super::super::errors;
use super::super::traits::Engine;
use ccore::{EngineInfo, StateInfo};
use cjson::bytes::{Bytes, WithoutPrefix};
use cstate::{query_stake_state, FindDoubleVoteHandler};
use ctypes::BlockId;
use jsonrpc_core::Result;
use std::sync::Arc;

pub struct EngineClient<C>
where
    C: EngineInfo + StateInfo + FindDoubleVoteHandler, {
    client: Arc<C>,
}

impl<C> EngineClient<C>
where
    C: EngineInfo + StateInfo + FindDoubleVoteHandler,
{
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
        }
    }
}

impl<C> Engine for EngineClient<C>
where
    C: EngineInfo + StateInfo + FindDoubleVoteHandler + 'static,
{
    fn get_custom_action_data(
        &self,
        _handler_id: u64,
        key_fragment: Bytes,
        block_number: Option<u64>,
    ) -> Result<Option<WithoutPrefix<Bytes>>> {
        let block_id = block_number.map(BlockId::Number).unwrap_or(BlockId::Latest);
        let state = self.client.state_at(block_id).ok_or_else(errors::state_not_exist)?;

        match query_stake_state(&key_fragment, &state) {
            Ok(Some(action_data)) => Ok(Some(Bytes::new(action_data).into_without_prefix())),
            Ok(None) => Ok(None),
            Err(e) => Err(errors::transaction_core(e)),
        }
    }
}
