// Copyright 2018, 2020 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::header::Header;
use ccore::BlockView;
use coordinator::Transaction as CTransaction;
use foundry_graphql_types::*;

// TODO: Fetching and holding the entire data always could be inefficient.
pub struct Block {
    block: Vec<u8>,
}

impl Block {
    /// `block` must be RLP-encoded.
    pub fn new(block: Vec<u8>) -> Self {
        Block {
            block,
        }
    }
}

// TODO: Add evidence
#[async_graphql::Object]
impl Block {
    async fn header(&self) -> Header {
        let view = BlockView::new(&self.block);
        Header::new(view.header_rlp().as_raw().to_vec())
    }

    async fn transactions(&self) -> Vec<Transaction> {
        let view = BlockView::new(&self.block);
        view.transactions()
            .into_iter()
            .map(|transaction| Transaction {
                transaction,
            })
            .collect()
    }
}

struct Transaction {
    transaction: CTransaction,
}

#[async_graphql::Object]
impl Transaction {
    async fn tx_type(&self) -> &str {
        self.transaction.tx_type()
    }

    async fn body(&self) -> GqlBytes {
        GqlBytes(self.transaction.body().to_vec())
    }
}
