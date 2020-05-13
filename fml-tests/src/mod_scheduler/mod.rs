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

mod impls;

use crate::services::*;
use fml::*;
use impls::*;
use std::sync::{Condvar, Mutex};

pub struct MyContext {
    number: usize,
    map: Mutex<AvailiableMap>,
    lock: Mutex<bool>,
    cvar: Condvar,
}

impl fml::Custom for MyContext {
    fn new(context: &fml::Config) -> Self {
        let (number, threads): (usize, usize) = serde_cbor::from_slice(&context.args).unwrap();
        let map = new_avail_map(number, threads);
        MyContext {
            number,
            map: Mutex::new(map),
            lock: Mutex::new(true),
            cvar: Condvar::new(),
        }
    }
}

pub struct Preset;

impl HandlePreset for Preset {
    fn export() -> Vec<HandleExchange> {
        let ctx = get_context();
        let mut result = Vec::new();
        for i in 0..ctx.custom.number {
            let importer = format!("Module{}", i);

            result.push(HandleExchange {
                exporter: "Schedule".to_owned(),
                importer: importer.clone(),
                handles: vec![TraitHolder::<dyn Schedule>::export(
                    ctx.ports.read().unwrap().find(&importer).unwrap(),
                    Box::new(MySchedule {
                        handle: Default::default(),
                    }),
                )],
                argument: Vec::new(),
            })
        }
        result
    }

    fn import(_exchange: HandleExchange) {
        panic!("Nothing to import!")
    }
}

fml_setup!(MyContext, Preset, None);
