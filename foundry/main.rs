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
extern crate serde_derive;
#[macro_use]
extern crate codechain_logger as clogger;

mod config;
mod constants;
mod dummy_network_service;
mod json;
mod rpc;
mod rpc_apis;
mod run_node;
mod subcommand;

use crate::run_node::run_node;
use crate::subcommand::run_subcommand;
use app_dirs::AppInfo;
use clap::load_yaml;

pub const APP_INFO: AppInfo = AppInfo {
    name: "foundry",
    author: "Kodebox",
};

#[cfg(all(unix, target_arch = "x86_64"))]
fn main() -> Result<(), String> {
    panic_hook::set();

    // Always print backtrace on panic.
    ::std::env::set_var("RUST_BACKTRACE", "1");

    run()
}

fn run() -> Result<(), String> {
    let yaml = load_yaml!("foundry.yml");
    let version = env!("CARGO_PKG_VERSION");
    let matches = clap::App::from_yaml(yaml).version(version).get_matches();

    match matches.subcommand {
        Some(_) => run_subcommand(&matches),
        None => run_node(&matches),
    }
}
