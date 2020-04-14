#![allow(dead_code, unused_variables)]
use self::traits::{BlockExecutor, Initializer, TxFilter};
use self::types::*;
use context::SubStorageAccess;
use ctypes::{CompactValidatorSet, ConsensusParams};

pub mod context;
pub mod test_coordinator;
pub mod traits;
pub mod types;

/// The `Coordinator` encapsulates all the logic for a Foundry application.
///
/// It assembles modules and feeds them various events from the underlying
/// consensus engine.

#[derive(Default)]
pub struct Coordinator {}

impl Initializer for Coordinator {
    fn initialize_chain(&self, app_state: String) -> (CompactValidatorSet, ConsensusParams) {
        unimplemented!()
    }
}

impl BlockExecutor for Coordinator {
    fn open_block(&self, context: &mut dyn SubStorageAccess, header: &Header, verified_crime: &[VerifiedCrime]) {
        unimplemented!()
    }

    fn execute_transactions(&self, context: &mut dyn SubStorageAccess, transactions: &[Transaction]) {
        unimplemented!()
    }

    fn close_block(&self, context: &mut dyn SubStorageAccess) -> BlockOutcome {
        unimplemented!()
    }
}

impl TxFilter for Coordinator {
    fn check_transaction(&self, transaction: &Transaction) -> Result<(), ErrorCode> {
        unimplemented!()
    }

    fn fetch_transactions_for_block<'a>(
        &self,
        transactions: &'a [&'a TransactionWithMetadata],
    ) -> Vec<TransactionWithGas<'a>> {
        unimplemented!()
    }

    fn filter_transactions<'a>(
        &self,
        transactions: &'a [&'a TransactionWithMetadata],
        memory_limit: Option<usize>,
        size_limit: Option<usize>,
    ) -> (Vec<&'a TransactionWithMetadata>, Vec<&'a TransactionWithMetadata>) {
        unimplemented!()
    }
}

pub struct Builder<C: context::Context> {
    context: C,
}

impl<C: context::Context> Builder<C> {
    fn new(context: C) -> Self {
        Builder {
            context,
        }
    }

    fn build(self) -> Coordinator {
        Coordinator {}
    }
}
