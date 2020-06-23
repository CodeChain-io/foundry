use crate::account::AccountManager;
use crate::common::*;
use crate::token::TokenManager;
use coordinator::context::SubStorageAccess;
use coordinator::module::*;
use coordinator::types::*;
use primitives::H256;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

struct Context {
    account: Arc<dyn AccountManager>,
    token: Arc<dyn TokenManager>,
    token_issuer: H256,
}

impl BlockOpen for Context {
    fn block_opened(&self, _storage: Box<dyn SubStorageAccess>) -> Result<(), HeaderError> {
        // stamp module is very special; it doesn't access to the storage AT ALL.
        Ok(())
    }
}

impl BlockClosed for Context {
    fn block_closed(&self) -> Result<BlockOutcome, CloseBlockError> {
        Ok(BlockOutcome {
            updated_consensus_params: None,
            updated_validator_set: None,
            events: Vec::new(),
        })
    }
}

enum ExecuteError {
    InvalidMetadata,
    InvalidSign,
    InvalidFormat,
    AccountModuleError(crate::account::Error),
    TokenModuleError(crate::token::Error),
    InvalidSequence,
    NotEligibleStamper,
}

impl Context {
    fn excute_tx(&self, transaction: &Transaction) -> Result<(), ExecuteError> {
        if transaction.tx_type() != "Stamp" {
            return Err(ExecuteError::InvalidMetadata)
        }

        let tx: OwnTransaction =
            serde_cbor::from_slice(&transaction.body()).map_err(|_| ExecuteError::InvalidFormat)?;
        tx.verify().map_err(|_| ExecuteError::InvalidSign)?;
        if self.account.get_sequence(&tx.signer_public).map_err(ExecuteError::AccountModuleError)? != tx.tx.seq {
            return Err(ExecuteError::InvalidSequence)
        }

        let account = self.token.get_account(tx.signer_public).map_err(ExecuteError::TokenModuleError)?;
        if account.tokens.iter().any(|x| x.issuer == self.token_issuer) {
            Ok(())
        } else {
            Err(ExecuteError::NotEligibleStamper)
        }
    }
}

impl TxOwner for Context {
    fn execute_transaction(&self, transaction: &Transaction) -> Result<TransactionExecutionOutcome, ()> {
        if let Err(error) = self.excute_tx(transaction) {
            match error {
                ExecuteError::InvalidMetadata => Err(()),
                ExecuteError::InvalidSign => Err(()),
                ExecuteError::InvalidFormat => Err(()),
                ExecuteError::AccountModuleError(_) => Err(()),
                ExecuteError::TokenModuleError(_) => Err(()),
                ExecuteError::InvalidSequence => Err(()),
                ExecuteError::NotEligibleStamper => Ok(Default::default()), // Don't reject; just accept though it fails to mutate something.
            }
        } else {
            Ok(Default::default())
        }
    }

    fn propose_transaction<'a>(&self, _transaction: &TransactionWithMetadata) -> bool {
        unimplemented!()
    }

    fn check_transaction(&self, transaction: &Transaction) -> Result<(), coordinator::types::ErrorCode> {
        let todo_fixthis: coordinator::types::ErrorCode = 3;
        assert_eq!(transaction.tx_type(), "Stamp");
        let tx: OwnTransaction = serde_cbor::from_slice(&transaction.body()).map_err(|_| todo_fixthis)?;
        tx.verify().map_err(|_| todo_fixthis)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TxStamp {
    hash: H256,
}

impl Action for TxStamp {}

pub type OwnTransaction = crate::common::SignedTransaction<TxStamp>;
