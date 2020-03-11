#![allow(dead_code, unused_variables)]
use std::unimplemented;
use validator::*;

pub mod context;
pub mod validator;

/// The `Coordinator` encapsulates all the logic for a Foundry application.
///
/// It assembles modules and feeds them various events from the underlying
/// consensus engine.

#[derive(Default)]
pub struct Coordinator<C: context::Context> {
    context: C,
}

impl<C: context::Context> validator::Validator for Coordinator<C> {
    fn initialize_chain(&mut self) -> ConsensusParams {
        unimplemented!()
    }

    fn execute_block(&mut self, header: &Header, transactions: &[Transaction], evidences: &[Evidence]) -> BlockOutcome {
        unimplemented!()
    }

    fn check_transaction(&mut self, transaction: &Transaction) -> bool {
        unimplemented!()
    }

    fn fetch_transactions_for_block(&self, transactions: &[TransactionWithMetadata]) -> Vec<TransactionWithGas> {
        unimplemented!()
    }

    fn remove_old_transactions(&self, all_transactions: &[TransactionWithMetadata]) -> Vec<TransactionWithMetadata> {
        unimplemented!()
    }
}

impl<C: context::Context> Coordinator<C> {}

pub struct Builder<C: context::Context> {
    context: C,
}

impl<C: context::Context> Builder<C> {
    fn new<CTX: context::Context>(ctx: CTX) -> Builder<CTX> {
        Builder {
            context: ctx,
        }
    }

    fn build(self) -> Coordinator<C> {
        Coordinator {
            context: self.context,
        }
    }
}
