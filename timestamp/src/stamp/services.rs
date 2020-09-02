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

use super::types::*;
use super::ServiceHandler;
pub use ckey::{Ed25519Private as Private, Ed25519Public as Public};
use coordinator::module::*;
use coordinator::types::*;
use coordinator::{Header, Transaction};
use std::collections::HashMap;

enum ExecuteError {
    InvalidMetadata,
    InvalidSign,
    InvalidFormat,
    AccountModuleError(crate::account::Error),
    TokenModuleError(crate::token::Error),
    InvalidSequence,
    NotEligibleStamper,
}

/// As this module is stateless, we implement execute_tx() right on the ServiceHandler.
impl ServiceHandler {
    fn excute_tx(&self, session: SessionId, transaction: &Transaction) -> Result<(), ExecuteError> {
        if transaction.tx_type() != "stamp" {
            return Err(ExecuteError::InvalidMetadata)
        }

        let tx: OwnTransaction =
            serde_cbor::from_slice(&transaction.body()).map_err(|_| ExecuteError::InvalidFormat)?;
        tx.verify().map_err(|_| ExecuteError::InvalidSign)?;
        if self
            .account_manager
            .read()
            .get_account(session, &tx.signer_public, true)
            .map_err(ExecuteError::AccountModuleError)?
            .seq
            != tx.tx.seq
        {
            return Err(ExecuteError::InvalidSequence)
        }

        let account = self
            .token_manager
            .read()
            .get_account(session, &tx.signer_public, false)
            .map_err(ExecuteError::TokenModuleError)?;
        if account.tokens.iter().any(|x| x.issuer == self.config.token_issuer) {
            self.account_manager.read().increase_sequence(session, &tx.signer_public, true).unwrap();
            Ok(())
        } else {
            Err(ExecuteError::NotEligibleStamper)
        }
    }
}

impl InitGenesis for ServiceHandler {
    fn init_genesis(&self, session: SessionId, config: &[u8]) {
        let stampers: HashMap<Public, usize> = serde_cbor::from_slice(&config).unwrap();
        for (stamper, number) in stampers {
            for _ in 0..number {
                let token_issuer = self.config.token_issuer;
                self.token_manager.read().issue_token(session, &token_issuer, &stamper).unwrap()
            }
        }
    }
}

impl TxOwner for ServiceHandler {
    fn block_opened(&self, _session: SessionId, _: &Header) -> Result<(), HeaderError> {
        Ok(())
    }

    fn execute_transaction(&self, session: SessionId, transaction: &Transaction) -> Result<TransactionOutcome, ()> {
        if let Err(error) = self.excute_tx(session, transaction) {
            match error {
                ExecuteError::InvalidMetadata => Err(()),
                ExecuteError::InvalidSign => Err(()),
                ExecuteError::InvalidFormat => Err(()),
                ExecuteError::AccountModuleError(_) => Err(()),
                ExecuteError::TokenModuleError(_) => Err(()),
                ExecuteError::InvalidSequence => Err(()),
                ExecuteError::NotEligibleStamper => Err(()),
            }
        } else {
            Ok(Default::default())
        }
    }

    fn check_transaction(&self, transaction: &Transaction) -> Result<(), coordinator::types::ErrorCode> {
        let todo_fixthis: coordinator::types::ErrorCode = 3;
        assert_eq!(transaction.tx_type(), "stamp");
        let tx: OwnTransaction = serde_cbor::from_slice(&transaction.body()).map_err(|_| todo_fixthis)?;
        tx.verify().map_err(|_| todo_fixthis)?;
        Ok(())
    }

    fn block_closed(&self, _session: SessionId) -> Result<Vec<Event>, CloseBlockError> {
        Ok(Vec::new())
    }
}
