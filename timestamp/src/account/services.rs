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

use super::state_machine::*;
use super::types::*;
use super::ServiceHandler;
use crate::common::SignedTransaction;
pub use ckey::{Ed25519Private as Private, Ed25519Public as Public};
use coordinator::module::*;
use coordinator::types::*;
use coordinator::{Header, Transaction};
use remote_trait_object::{service, Service};

#[service]
pub trait AccountManager: Service {
    // Immutable accesses
    fn get_account(&self, session: SessionId, public: &Public, default: bool) -> Result<Account, Error>;

    // Mutable accesses
    fn create_account(&self, session: SessionId, public: &Public) -> Result<(), Error>;
    fn increase_sequence(&self, session: SessionId, public: &Public, default: bool) -> Result<(), Error>;
}

impl AccountManager for ServiceHandler {
    fn get_account(&self, session: SessionId, public: &Public, default: bool) -> Result<Account, Error> {
        let state_machine = self.create_state_machine(session);
        state_machine.execute_access(GetAccount {
            public,
            default,
        })
    }

    fn create_account(&self, session: SessionId, public: &Public) -> Result<(), Error> {
        let state_machine = self.create_state_machine(session);
        state_machine.execute_transition(CreateAccount {
            public,
        })
    }

    fn increase_sequence(&self, session: SessionId, public: &Public, default: bool) -> Result<(), Error> {
        let state_machine = self.create_state_machine(session);
        state_machine.execute_transition(IncreaseSequence {
            public,
            default,
            config: self.config(),
        })
    }
}

impl TxOwner for ServiceHandler {
    fn block_opened(&self, _: SessionId, _: &Header) -> Result<(), HeaderError> {
        Ok(())
    }

    fn execute_transaction(&self, session: SessionId, transaction: &Transaction) -> Result<TransactionOutcome, ()> {
        let state_machine = self.create_state_machine(session);
        if let Err(error) = state_machine.execute_transition(ExecuteTransaction {
            tx: transaction,
            config: self.config(),
        }) {
            match error {
                ExecuteError::InvalidMetadata => Err(()),
                ExecuteError::InvalidSign => Err(()),
                ExecuteError::InvalidFormat => Err(()),
                ExecuteError::AccountError(_) => Err(()),
                ExecuteError::InvalidSequence => Err(()),
                ExecuteError::NotAllowedHello => Err(()),
            }
        } else {
            Ok(Default::default())
        }
    }

    fn check_transaction(&self, transaction: &Transaction) -> Result<(), coordinator::types::ErrorCode> {
        let todo_fixthis: coordinator::types::ErrorCode = 3;
        assert_eq!(transaction.tx_type(), "hello");
        let tx: SignedTransaction = serde_cbor::from_slice(&transaction.body()).map_err(|_| todo_fixthis)?;
        tx.verify().map_err(|_| todo_fixthis)?;
        Ok(())
    }

    fn block_closed(&self, _session: SessionId) -> Result<Vec<Event>, CloseBlockError> {
        Ok(Vec::new())
    }
}
