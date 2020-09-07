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

extern crate codechain_module as cmodule;
extern crate codechain_timestamp as timestamp;
extern crate foundry_process_sandbox as fproc_sndbx;

use coordinator::Coordinator;

mod timestamp_setup {
    use super::*;
    use codechain_module::impls::process::{ExecutionScheme, SingleProcess};
    use codechain_module::MODULE_INITS;
    use foundry_module_rt::start;
    use foundry_process_sandbox::execution::executor::add_function_pool;
    use linkme::distributed_slice;
    use std::sync::Arc;

    #[distributed_slice(MODULE_INITS)]
    fn account() {
        add_function_pool(
            "a010000000012345678901234567890123456789012345678901234567890123".to_owned(),
            Arc::new(start::<<SingleProcess as ExecutionScheme>::Ipc, timestamp::account::Module>),
        );
    }

    #[distributed_slice(MODULE_INITS)]
    fn staking() {
        add_function_pool(
            "a020000000012345678901234567890123456789012345678901234567890123".to_owned(),
            Arc::new(start::<<SingleProcess as ExecutionScheme>::Ipc, timestamp::staking::Module>),
        );
    }

    #[distributed_slice(MODULE_INITS)]
    fn stamp() {
        add_function_pool(
            "a030000000012345678901234567890123456789012345678901234567890123".to_owned(),
            Arc::new(start::<<SingleProcess as ExecutionScheme>::Ipc, timestamp::stamp::Module>),
        );
    }

    #[distributed_slice(MODULE_INITS)]
    fn token() {
        add_function_pool(
            "a040000000012345678901234567890123456789012345678901234567890123".to_owned(),
            Arc::new(start::<<SingleProcess as ExecutionScheme>::Ipc, timestamp::token::Module>),
        );
    }

    #[distributed_slice(MODULE_INITS)]
    fn sorting() {
        add_function_pool(
            "a050000000012345678901234567890123456789012345678901234567890123".to_owned(),
            Arc::new(start::<<SingleProcess as ExecutionScheme>::Ipc, timestamp::sorting::Module>),
        );
    }
}

fn app_desc_path() -> &'static str {
    if std::path::Path::exists(std::path::Path::new("../app-desc.yml")) {
        "../app-desc.yml"
    } else {
        "./app-desc.yml"
    }
}

#[test]
fn weave() {
    let app_desc = std::fs::read_to_string(app_desc_path()).unwrap();
    let c = Coordinator::from_app_desc(&app_desc).unwrap();

    assert_eq!(c.services().stateful.lock().len(), 2);
    assert_eq!(c.services().init_genesis.len(), 2);
    assert_eq!(c.services().tx_owner.len(), 3);
    assert_eq!(c.services().handle_graphqls.len(), 2);
}

#[test]
fn weave_conccurent() {
    for i in 0..8 {
        let n = 8;
        let mut joins = Vec::new();
        for _ in 0..n {
            joins.push(std::thread::spawn(|| {
                let app_desc = std::fs::read_to_string(app_desc_path()).unwrap();
                let c = Coordinator::from_app_desc(&app_desc).unwrap();

                assert_eq!(c.services().stateful.lock().len(), 2);
                assert_eq!(c.services().init_genesis.len(), 2);
                assert_eq!(c.services().tx_owner.len(), 3);
                assert_eq!(c.services().handle_graphqls.len(), 2);
            }))
        }
        for j in joins {
            j.join().unwrap();
        }
        println!("{}", i);
    }
}
