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

use super::super::traits::IBC;
use ccore::{BlockChainClient, StateInfo};
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

impl<C> IBC for IBCClient<C> where C: StateInfo + 'static + Send + Sync + BlockChainClient {}
