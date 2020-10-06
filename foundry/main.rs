// Copyright 2018, 2020 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

#[macro_use]
extern crate log;
#[macro_use]
extern crate codechain_logger as clogger;

use app_dirs::AppInfo;
use clap::load_yaml;

pub use crate::run_node::run_node;
use crate::subcommand::run_subcommand;

mod config;
mod constants;
mod dummy_network_service;
mod json;
mod rpc;
mod rpc_apis;
mod run_node;
mod subcommand;
mod tests;

pub const APP_INFO: AppInfo = AppInfo {
    name: "foundry",
    author: "Kodebox",
};

#[actix_rt::main]
#[cfg(all(unix, target_arch = "x86_64"))]
async fn main() -> Result<(), String> {
    //panic_hook::set();

    // Always print backtrace on panic.
    std::env::set_var("RUST_BACKTRACE", "1");
    run()
}

fn run() -> Result<(), String> {
    let yaml = load_yaml!("foundry.yml");
    let version = env!("CARGO_PKG_VERSION");
    let matches = clap::App::from_yaml(yaml).version(version).get_matches();

    match matches.subcommand_name() {
        Some(_) => run_subcommand(&matches),
        None => run_node(&matches, None),
    }
}

mod timestamp_setup {
    use cmodule::impls::process::{ExecutionScheme, SingleProcess};
    use cmodule::MODULE_INITS;
    use foundry_module_rt::start;
    use foundry_process_sandbox::execution::executor::add_function_pool;
    use linkme::distributed_slice;
    use std::sync::Arc;

    #[distributed_slice(MODULE_INITS)]
    fn account() {
        add_function_pool(
            "a010000000012345678901234567890123456789012345678901234567890123".to_owned(),
            Arc::new(start::<<SingleProcess as ExecutionScheme>::Ipc, codechain_timestamp::account::Module>),
        );
    }

    #[distributed_slice(MODULE_INITS)]
    fn staking() {
        add_function_pool(
            "a020000000012345678901234567890123456789012345678901234567890123".to_owned(),
            Arc::new(start::<<SingleProcess as ExecutionScheme>::Ipc, codechain_timestamp::staking::Module>),
        );
    }

    #[distributed_slice(MODULE_INITS)]
    fn stamp() {
        add_function_pool(
            "a030000000012345678901234567890123456789012345678901234567890123".to_owned(),
            Arc::new(start::<<SingleProcess as ExecutionScheme>::Ipc, codechain_timestamp::stamp::Module>),
        );
    }

    #[distributed_slice(MODULE_INITS)]
    fn token() {
        add_function_pool(
            "a040000000012345678901234567890123456789012345678901234567890123".to_owned(),
            Arc::new(start::<<SingleProcess as ExecutionScheme>::Ipc, codechain_timestamp::token::Module>),
        );
    }

    #[distributed_slice(MODULE_INITS)]
    fn sorting() {
        add_function_pool(
            "a050000000012345678901234567890123456789012345678901234567890123".to_owned(),
            Arc::new(start::<<SingleProcess as ExecutionScheme>::Ipc, codechain_timestamp::sorting::Module>),
        );
    }
}
