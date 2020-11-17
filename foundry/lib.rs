// Copyright 2020 Kodebox, Inc.
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
use clap::{values_t, SubCommand};
use config::Config;
use std::{collections::BTreeMap, str::FromStr};
use structconf::StructConf;

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

pub fn run() -> Result<(), String> {
    let version = env!("CARGO_PKG_VERSION");
    let app = clap::App::new("foundry")
        .version(version)
        .author("CodeChain Team <hi@codechain.io>")
        .about("Foundry client")
        .subcommand(SubCommand::with_name("commit-hash").about("Print the commit hash of the source tree"));

    let app = app.arg(
        clap::Arg::with_name("module_arguments")
            .short("D")
            .help("Override config values in app-descriptor. For example  '-D foo=bar -D x=y'")
            .takes_value(true)
            .multiple(true),
    );

    let matches = Config::parse_args(app);
    let config_path = matches.value_of("config").unwrap_or("./config.ini");
    let conf = Config::parse_file(&matches, config_path).expect("read config file");

    let module_arguments = if matches.is_present("module_arguments") {
        let values = values_t!(matches, "module_arguments", EqualSeparated).unwrap_or_else(|e| e.exit());
        EqualSeparated::vec_into_map(values)
    } else {
        Default::default()
    };

    match matches.subcommand_name() {
        Some(_) => run_subcommand(&matches),
        None => run_node(conf, module_arguments, None),
    }
}

struct EqualSeparated {
    key: String,
    value: String,
}

impl EqualSeparated {
    fn vec_into_map(values: Vec<EqualSeparated>) -> BTreeMap<String, String> {
        values
            .into_iter()
            .map(
                |EqualSeparated {
                     key,
                     value,
                 }| (key, value),
            )
            .collect()
    }
}

impl FromStr for EqualSeparated {
    type Err = String;

    fn from_str(keyvalue: &str) -> Result<Self, Self::Err> {
        let splitted: Vec<&str> = keyvalue.split('=').collect();
        if splitted.len() < 2 {
            return Err("Invalid module arguments. Please use -D key=value format".to_string())
        }
        let key = splitted[0].to_string();
        let value = splitted[1].to_string();
        Ok(EqualSeparated {
            key,
            value,
        })
    }
}
