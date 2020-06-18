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

use super::context::SubStorageAccess;
use super::types::*;
use ctypes::{CompactValidatorSet, ConsensusParams};

pub trait OnChainInit: Send + Sync {
    fn chain_init(&self) -> (CompactValidatorSet, ConsensusParams);
}

pub trait OnBlockOpen: Send + Sync {
    fn block_opened() -> Result<(), HeaderError>;
}

pub trait OnBlockClosed: Send + Sync {
    fn block_closed(&self, storage: &mut dyn SubStorageAccess) -> Result<BlockOutcome, CloseBlockError>;
}

pub trait TxOwner: Send + Sync {
    fn execute_transactions(
        &self,
        context: &mut dyn SubStorageAccess,
        transaction: &Transaction,
    ) -> Result<Vec<TransactionExecutionOutcome>, ()>;

    fn propose_transaction<'a>(
        &self,
        context: &mut dyn SubStorageAccess,
        transaction: &TransactionWithMetadata,
    ) -> bool;

    fn check_transaction(&self, transaction: &Transaction) -> Result<(), ErrorCode>;
}
