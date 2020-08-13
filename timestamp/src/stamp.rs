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

use crate::account::AccountManager;
use crate::common::*;
use crate::token::TokenManager;
use ccrypto::blake256;
pub use ckey::Ed25519Public as Public;
use coordinator::module::*;
use coordinator::types::*;
use coordinator::{Header, Transaction};
use foundry_module_rt::UserModule;
use parking_lot::RwLock;
use primitives::H256;
use remote_trait_object::raw_exchange::{import_service_from_handle, HandleToExchange, Skeleton};
use remote_trait_object::{Context as RtoContext, Service};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

struct Context {
    account: Option<Box<dyn AccountManager>>,
    token: Option<Box<dyn TokenManager>>,
    token_issuer: H256,
}

impl Context {
    fn account(&self) -> &dyn AccountManager {
        self.account.as_ref().unwrap().as_ref()
    }

    fn account_mut(&mut self) -> &mut dyn AccountManager {
        self.account.as_mut().unwrap().as_mut()
    }

    fn token(&self) -> &dyn TokenManager {
        self.token.as_ref().unwrap().as_ref()
    }

    fn token_mut(&mut self) -> &mut dyn TokenManager {
        self.token.as_mut().unwrap().as_mut()
    }
}

impl Service for Context {}

pub struct Module {
    ctx: Arc<RwLock<Context>>,
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
    fn excute_tx(&mut self, transaction: &Transaction) -> Result<(), ExecuteError> {
        if transaction.tx_type() != "stamp" {
            return Err(ExecuteError::InvalidMetadata)
        }

        let tx: OwnTransaction =
            serde_cbor::from_slice(&transaction.body()).map_err(|_| ExecuteError::InvalidFormat)?;
        tx.verify().map_err(|_| ExecuteError::InvalidSign)?;
        if self.account().get_sequence(&tx.signer_public, true).map_err(ExecuteError::AccountModuleError)? != tx.tx.seq
        {
            return Err(ExecuteError::InvalidSequence)
        }

        let account = self.token().get_account(&tx.signer_public).map_err(ExecuteError::TokenModuleError)?;
        if account.tokens.iter().any(|x| x.issuer == self.token_issuer) {
            self.account_mut().increase_sequence(&tx.signer_public, true).unwrap();
            Ok(())
        } else {
            Err(ExecuteError::NotEligibleStamper)
        }
    }
}

impl InitGenesis for Context {
    fn begin_genesis(&mut self) {}

    fn init_genesis(&mut self, config: &[u8]) {
        let stampers: HashMap<Public, usize> = serde_cbor::from_slice(&config).unwrap();
        for (stamper, number) in stampers {
            for _ in 0..number {
                let token_issuer = self.token_issuer;
                self.token_mut().issue_token(&token_issuer, &stamper).unwrap()
            }
        }
    }

    fn end_genesis(&mut self) {}
}

impl TxOwner for Context {
    fn block_opened(&mut self, _: &Header) -> Result<(), HeaderError> {
        Ok(())
    }

    fn execute_transaction(&mut self, transaction: &Transaction) -> Result<TransactionOutcome, ()> {
        if let Err(error) = self.excute_tx(transaction) {
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

    fn block_closed(&mut self) -> Result<Vec<Event>, CloseBlockError> {
        Ok(Vec::new())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TxStamp {
    pub hash: H256,
}

impl Action for TxStamp {}

pub type OwnTransaction = crate::common::SignedTransaction<TxStamp>;

struct GetAccountAndSeq;

impl Service for GetAccountAndSeq {}

impl crate::sorting::GetAccountAndSeq for GetAccountAndSeq {
    fn get_account_and_seq(&self, tx: &Transaction) -> Result<(Public, u64), ()> {
        assert_eq!(tx.tx_type(), "stamp");
        let tx: OwnTransaction = serde_cbor::from_slice(&tx.body()).map_err(|_| ())?;
        Ok((tx.signer_public, tx.tx.seq))
    }
}

impl UserModule for Module {
    fn new(_arg: &[u8]) -> Self {
        Module {
            ctx: Arc::new(RwLock::new(Context {
                account: None,
                token: None,
                token_issuer: blake256("stamp"),
            })),
        }
    }

    fn prepare_service_to_export(&mut self, ctor_name: &str, ctor_arg: &[u8]) -> Skeleton {
        match ctor_name {
            "tx_owner" => {
                let arg: String = serde_cbor::from_slice(ctor_arg).unwrap();
                assert_eq!(arg, "unused");
                Skeleton::new(Arc::clone(&self.ctx) as Arc<RwLock<dyn TxOwner>>)
            }
            "init_genesis" => {
                let arg: String = serde_cbor::from_slice(ctor_arg).unwrap();
                assert_eq!(arg, "unused");
                Skeleton::new(Arc::clone(&self.ctx) as Arc<RwLock<dyn InitGenesis>>)
            }
            "get_account_and_seq" => {
                let arg: String = serde_cbor::from_slice(ctor_arg).unwrap();
                assert_eq!(arg, "unused");
                Skeleton::new(Box::new(GetAccountAndSeq) as Box<dyn crate::sorting::GetAccountAndSeq>)
            }
            _ => panic!("Unsupported ctor_name in prepare_service_to_export() : {}", ctor_name),
        }
    }

    fn import_service(&mut self, rto_context: &RtoContext, name: &str, handle: HandleToExchange) {
        match name {
            "account_manager" => {
                self.ctx.write().account.replace(import_service_from_handle(rto_context, handle));
            }
            "token_manager" => {
                self.ctx.write().token.replace(import_service_from_handle(rto_context, handle));
            }
            _ => panic!("Invalid name in import_service()"),
        }
    }

    fn debug(&mut self, _arg: &[u8]) -> Vec<u8> {
        unimplemented!()
    }
}
