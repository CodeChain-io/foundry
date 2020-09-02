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

use super::graphql::GraphQlRequestHandler;
use super::ServiceHandler;
use crate::common::*;
pub use ckey::{Ed25519Private as Private, Ed25519Public as Public};
use coordinator::module::*;
use foundry_module_rt::UserModule;
use remote_trait_object::raw_exchange::{import_service_from_handle, HandleToExchange, Skeleton};
use remote_trait_object::Context as RtoContext;
use std::sync::Arc;

pub struct Module {
    service_handler: Arc<ServiceHandler>,
}

impl UserModule for Module {
    fn new(_arg: &[u8]) -> Self {
        Module {
            service_handler: Arc::new(ServiceHandler::new()),
        }
    }

    fn prepare_service_to_export(&mut self, ctor_name: &str, ctor_arg: &[u8]) -> Skeleton {
        match ctor_name {
            "token-manager" => {
                assert_empty_arg(ctor_arg).unwrap();
                Skeleton::new(Arc::clone(&self.service_handler) as Arc<dyn super::services::TokenManager>)
            }
            "stateful" => {
                assert_empty_arg(ctor_arg).unwrap();
                Skeleton::new(self.service_handler.get_stateful())
            }
            "tx-owner" => {
                assert_empty_arg(ctor_arg).unwrap();
                Skeleton::new(Arc::clone(&self.service_handler) as Arc<dyn TxOwner>)
            }
            "get-account-and-seq" => {
                assert_empty_arg(ctor_arg).unwrap();
                Skeleton::new(Box::new(super::types::GetAccountAndSeq) as Box<dyn crate::sorting::GetAccountAndSeq>)
            }
            "handle-graphql-request" => {
                assert_empty_arg(ctor_arg).unwrap();
                Skeleton::new(Box::new(GraphQlRequestHandler::new(Arc::clone(&self.service_handler)))
                    as Box<dyn HandleGraphQlRequest>)
            }
            _ => panic!("Unsupported ctor_name in prepare_service_to_export() : {}", ctor_name),
        }
    }

    fn import_service(&mut self, rto_context: &RtoContext, name: &str, handle: HandleToExchange) {
        match name {
            "account-manager" => {
                *(self.service_handler.account_manager().write()) = import_service_from_handle(rto_context, handle);
            }
            _ => panic!("Unsupported name in import_service() : {}", name),
        }
    }

    fn debug(&mut self, _arg: &[u8]) -> Vec<u8> {
        unimplemented!()
    }
}
