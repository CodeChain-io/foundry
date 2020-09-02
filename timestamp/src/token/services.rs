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
pub use ckey::{Ed25519Private as Private, Ed25519Public as Public};
use coordinator::module::*;
use coordinator::types::*;
use coordinator::{Header, Transaction};
use primitives::H256;
use remote_trait_object::{service, Service};
use std::collections::BTreeSet;

#[service]
pub trait TokenManager: Service {
    // Immutable accesses
    fn get_account(&self, session: SessionId, public: &Public, default: bool) -> Result<Account, Error>;
    fn get_owning_accounts_with_issuer(&self, session: SessionId, issuer: &H256) -> Result<BTreeSet<Public>, Error>;

    // Mutable accesses
    fn issue_token(&self, session: SessionId, issuer: &H256, receiver: &Public) -> Result<(), Error>;
}

impl TokenManager for ServiceHandler {
    fn get_account(&self, session: SessionId, public: &Public, default: bool) -> Result<Account, Error> {
        let state_machine = self.create_state_machine(session);
        state_machine.execute_access(GetAccount {
            public,
            default,
        })
    }

    fn get_owning_accounts_with_issuer(&self, session: SessionId, issuer: &H256) -> Result<BTreeSet<Public>, Error> {
        let state_machine = self.create_state_machine(session);
        state_machine.execute_access(GetOwningAccountsWithIssuer {
            issuer,
        })
    }

    fn issue_token(&self, session: SessionId, issuer: &H256, receiver: &Public) -> Result<(), Error> {
        let state_machine = self.create_state_machine(session);
        state_machine.execute_transition(IssueToken {
            issuer,
            receiver,
        })
    }
}

impl TxOwner for ServiceHandler {
    fn block_opened(&self, _session: SessionId, _header: &Header) -> Result<(), HeaderError> {
        Ok(())
    }

    fn execute_transaction(&self, session: SessionId, transaction: &Transaction) -> Result<TransactionOutcome, ()> {
        let state_machine = self.create_state_machine(session);

        let get_sequence =
            |public: &Public| self.account_manager.read().get_account(session, public, true).map(|x| x.seq);
        let increase_sequence = move |public: &Public| {
            self.account_manager.read().increase_sequence(session, public, true).unwrap();
        };

        if let Err(error) = state_machine.execute_transition(ExecuteTransaction {
            tx: transaction,
            get_sequence: &get_sequence,
            increase_sequence: &increase_sequence,
        }) {
            match error {
                ExecuteError::InvalidMetadata => Err(()),
                ExecuteError::InvalidSign => Err(()),
                ExecuteError::InvalidFormat => Err(()),
                ExecuteError::AccountModuleError(_) => Err(()),
                ExecuteError::InvalidSequence => Err(()),
                ExecuteError::NoSuchAccount => Err(()),
                ExecuteError::InvalidKey => Err(()),
                ExecuteError::NoToken => Err(()),
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
