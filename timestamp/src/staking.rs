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

use super::common::*;
use crate::token::TokenManager;
use ccrypto::blake256;
use ckey::Ed25519Public as Public;
use coordinator::module::*;
use ctypes::{CompactValidatorEntry, CompactValidatorSet, ConsensusParams};
use foundry_module_rt::UserModule;
use parking_lot::RwLock;
use primitives::H256;
use remote_trait_object::raw_exchange::{import_service_from_handle, HandleToExchange, Skeleton};
use remote_trait_object::{Context as RtoContext, Service};
use std::sync::Arc;

pub type Validators = Vec<Public>;

pub struct Context {
    pub token_manager: Option<Box<dyn TokenManager>>,
    pub validator_token_issuer: H256,
}

impl Context {
    fn token_manager(&self) -> &dyn TokenManager {
        self.token_manager.as_ref().unwrap().as_ref()
    }

    fn token_manager_mut(&mut self) -> &mut dyn TokenManager {
        self.token_manager.as_mut().unwrap().as_mut()
    }

    fn track_validator_set(&self) -> CompactValidatorSet {
        let validator_set: Validators = self
            .token_manager()
            .get_owning_accounts_with_issuer(&self.validator_token_issuer)
            .unwrap()
            .into_iter()
            .collect();
        CompactValidatorSet::new(
            validator_set
                .into_iter()
                .map(|x| CompactValidatorEntry {
                    public_key: x,
                    delegation: 1,
                })
                .collect(),
        )
    }
}

impl Service for Context {}

pub struct Module {
    ctx: Arc<RwLock<Context>>,
}

pub trait StakeManager: Send + Sync {
    fn get_validators(&self) -> Validators;
}

impl StakeManager for Context {
    fn get_validators(&self) -> Validators {
        self.token_manager()
            .get_owning_accounts_with_issuer(&self.validator_token_issuer)
            .unwrap()
            .into_iter()
            .collect()
    }
}

impl InitGenesis for Context {
    fn begin_genesis(&mut self) {}

    fn init_genesis(&mut self, config: &[u8]) {
        let initial_validator_set: Vec<String> = serde_cbor::from_slice(config).unwrap();
        let initial_validator_set: Validators =
            initial_validator_set.into_iter().map(|x| std::str::FromStr::from_str(&x).unwrap()).collect();
        let validator_token_issuer = self.validator_token_issuer;
        for public in initial_validator_set {
            self.token_manager_mut().issue_token(&validator_token_issuer, &public).unwrap();
        }
    }

    fn end_genesis(&mut self) {}
}

impl InitChain for Context {
    fn init_chain(&mut self) -> (CompactValidatorSet, ConsensusParams) {
        let validator_set = self.track_validator_set();
        let consensus_params = ConsensusParams::default_for_test();
        (validator_set, consensus_params)
    }
}

impl UpdateChain for Context {
    fn update_chain(&mut self) -> (Option<CompactValidatorSet>, Option<ConsensusParams>) {
        let validator_set = self.track_validator_set();
        let consensus_params = ConsensusParams::default_for_test();
        (Some(validator_set), Some(consensus_params))
    }
}

impl UserModule for Module {
    fn new(_arg: &[u8]) -> Self {
        Module {
            ctx: Arc::new(RwLock::new(Context {
                token_manager: None,
                validator_token_issuer: blake256("validator"),
            })),
        }
    }

    fn prepare_service_to_export(&mut self, ctor_name: &str, ctor_arg: &[u8]) -> Skeleton {
        match ctor_name {
            "init-genesis" => {
                assert_empty_arg(ctor_arg).unwrap();
                Skeleton::new(Arc::clone(&self.ctx) as Arc<RwLock<dyn InitGenesis>>)
            }
            "init-chain" => {
                assert_empty_arg(ctor_arg).unwrap();
                Skeleton::new(Arc::clone(&self.ctx) as Arc<RwLock<dyn InitChain>>)
            }
            "update-chain" => {
                assert_empty_arg(ctor_arg).unwrap();
                Skeleton::new(Arc::clone(&self.ctx) as Arc<RwLock<dyn UpdateChain>>)
            }
            _ => panic!("Unsupported ctor_name in prepare_service_to_export() : {}", ctor_name),
        }
    }

    fn import_service(&mut self, rto_context: &RtoContext, name: &str, handle: HandleToExchange) {
        match name {
            "token-manager" => {
                self.ctx.write().token_manager.replace(import_service_from_handle(rto_context, handle));
            }
            _ => panic!("Invalid name in import_service()"),
        }
    }

    fn debug(&mut self, _arg: &[u8]) -> Vec<u8> {
        unimplemented!()
    }
}
