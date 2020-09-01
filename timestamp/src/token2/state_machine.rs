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
use crate::common::state_machine::{StateAccess, StateTransition};
pub use ckey::{Ed25519Private as Private, Ed25519Public as Public};
use coordinator::context::SubStorageAccess;
use coordinator::Transaction;
use primitives::H256;
use std::collections::BTreeSet;

/// Facades of the AccountManager
type GetSequence<'a> = dyn 'a + Fn(&Public) -> Result<crate::common::TxSeq, crate::account2::Error>;
type IncreaseSequence<'a> = dyn 'a + Fn(&Public);

pub struct GetAccount<'a> {
    pub public: &'a Public,
    pub default: bool,
}

impl<'a> StateAccess for GetAccount<'a> {
    type Outcome = Result<Account, Error>;

    fn execute(self, state: &dyn SubStorageAccess) -> Result<Account, Error> {
        let bytes = match state.get(self.public.as_ref()) {
            Some(bytes) => bytes,
            None => {
                if self.default {
                    return Ok(Default::default())
                } else {
                    return Err(Error::NoSuchAccount)
                }
            }
        };
        Ok(serde_cbor::from_slice(&bytes).map_err(|_| Error::InvalidKey)?)
    }
}

fn set_account(state: &mut dyn SubStorageAccess, key: &Public, account: &Account) {
    state.set(get_state_key(key).as_bytes(), serde_cbor::to_vec(account).unwrap());
}

fn set_owning_accounts_with_issuer(state: &mut dyn SubStorageAccess, issuer: &H256, set: BTreeSet<Public>) {
    state.set(get_state_key_account_set(issuer).as_bytes(), serde_cbor::to_vec(&set).unwrap());
}

pub struct IssueToken<'a> {
    pub issuer: &'a H256,
    pub receiver: &'a Public,
}

impl<'a> StateTransition for IssueToken<'a> {
    type Outcome = Result<(), Error>;

    fn execute(self, state: &mut dyn SubStorageAccess) -> Result<(), Error> {
        let mut account = GetAccount {
            public: self.receiver,
            default: true,
        }
        .execute(state)?;
        account.tokens.push(Token {
            issuer: *self.issuer,
        });
        set_account(state, self.receiver, &account);
        let mut set = GetOwningAccountsWithIssuer {
            issuer: self.issuer,
        }
        .execute(state)?;
        set.insert(*self.receiver);
        set_owning_accounts_with_issuer(state, self.issuer, set);

        Ok(())
    }
}

pub struct GetOwningAccountsWithIssuer<'a> {
    pub issuer: &'a H256,
}

impl<'a> StateAccess for GetOwningAccountsWithIssuer<'a> {
    type Outcome = Result<BTreeSet<Public>, Error>;

    fn execute(self, state: &dyn SubStorageAccess) -> Result<BTreeSet<Public>, Error> {
        Ok(if let Some(x) = state.get(get_state_key_account_set(self.issuer).as_bytes()) {
            serde_cbor::from_slice(&x).map_err(|_| Error::InvalidKey)?
        } else {
            BTreeSet::new()
        })
    }
}

#[derive(Debug)]
pub(super) enum ExecuteError {
    InvalidMetadata,
    InvalidSign,
    InvalidFormat,
    AccountModuleError(crate::account2::Error),
    InvalidSequence,
    NoSuchAccount,
    InvalidKey,
    NoToken,
}

impl From<Error> for ExecuteError {
    fn from(e: Error) -> Self {
        match e {
            Error::InvalidKey => ExecuteError::InvalidKey,
            Error::NoSuchAccount => ExecuteError::NoSuchAccount,
        }
    }
}

pub(super) struct ExecuteTransaction<'a, 'b> {
    pub tx: &'a Transaction,
    pub get_sequence: &'b GetSequence<'a>,
    pub increase_sequence: &'b IncreaseSequence<'a>,
}

impl<'a, 'b> StateTransition for ExecuteTransaction<'a, 'b> {
    type Outcome = Result<(), ExecuteError>;

    fn execute(self, state: &mut dyn SubStorageAccess) -> Result<(), ExecuteError> {
        if self.tx.tx_type() != "token" {
            return Err(ExecuteError::InvalidMetadata)
        }
        let tx: OwnTransaction = serde_cbor::from_slice(&self.tx.body()).map_err(|_| ExecuteError::InvalidFormat)?;
        tx.verify().map_err(|_| ExecuteError::InvalidSign)?;
        if (*self.get_sequence)(&tx.signer_public).map_err(ExecuteError::AccountModuleError)? != tx.tx.seq {
            return Err(ExecuteError::InvalidSequence)
        }

        let ActionTransferToken {
            receiver,
            issuer,
        } = tx.tx.action;

        let mut sender_account: Account = serde_cbor::from_slice(
            &state.get(get_state_key(&tx.signer_public).as_bytes()).ok_or_else(|| ExecuteError::NoSuchAccount)?,
        )
        .map_err(|_| ExecuteError::InvalidKey)?;

        let mut found = None;
        for (i, token) in sender_account.tokens.iter().enumerate() {
            if token.issuer == issuer {
                found = Some(i)
            }
        }
        let index = found.ok_or_else(|| ExecuteError::NoToken)?;
        let token = sender_account.tokens.remove(index);
        let mut recipient_account = GetAccount {
            public: &receiver,
            default: true,
        }
        .execute(state)?;
        let mut set = GetOwningAccountsWithIssuer {
            issuer: &issuer,
        }
        .execute(state)?;

        // From now on, it will actually mutate the state and must not fail
        // to keep the consistency of the state.

        // If that was the last token with the issuer
        if sender_account.tokens.iter().find(|&x| x.issuer == issuer).is_none() {
            assert!(set.remove(&tx.signer_public));
        }
        set.insert(receiver);

        set_owning_accounts_with_issuer(state, &issuer, set);

        recipient_account.tokens.push(token);
        set_account(state, &tx.signer_public, &sender_account);
        set_account(state, &receiver, &recipient_account);
        (*self.increase_sequence)(&tx.signer_public);
        Ok(())
    }
}
