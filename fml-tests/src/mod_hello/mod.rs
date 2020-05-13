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
use std::collections::HashMap;
use std::sync::RwLock;

pub struct MyContext {
    number: usize,
    factories: RwLock<HashMap<String, Box<dyn HelloFactory>>>,
}

impl fml::Custom for MyContext {
    fn new(context: &fml::Config) -> Self {
        let number = serde_cbor::from_slice(&context.args).unwrap();
        let mut factories = HashMap::new();
        factories.insert(
            context.id.clone(),
            Box::new(Factory {
                handle: Default::default(),
            }) as Box<dyn HelloFactory>,
        );
        MyContext {
            number,
            factories: RwLock::new(factories),
        }
    }
}

pub struct Preset;

impl HandlePreset for Preset {
    fn export() -> Vec<HandleExchange> {
        let ctx = get_context();
        let mut result = Vec::new();
        for i in 0..ctx.custom.number {
            let exporter = ctx.config.id.clone();
            let importer = format!("Module{}", i);
            if exporter == importer {
                continue
            }

            result.push(HandleExchange {
                exporter,
                importer: importer.clone(),
                handles: vec![TraitHolder::<dyn HelloFactory>::export(
                    ctx.ports.read().unwrap().find(&importer).unwrap(),
                    Box::new(Factory {
                        handle: Default::default(),
                    }),
                )],
                argument: Vec::new(),
            })
        }
        result
    }

    fn import(mut exchange: HandleExchange) {
        let ctx = get_context();
        assert_eq!(exchange.importer, ctx.config.id, "Invalid import request");
        let mut guard = ctx.custom.factories.write().unwrap();
        assert_eq!(exchange.handles.len(), 1);
        let h = TraitHolder::<dyn HelloFactory>::import(exchange.handles.pop().unwrap());
        guard.insert(exchange.exporter, h);
    }
}

pub fn initiate(_arg: Vec<u8>) -> Vec<u8> {
    let ctx = get_context();
    let guard = ctx.custom.factories.read().unwrap();

    for n in 0..ctx.custom.number {
        let factory = guard.get(&format!("Module{}", n)).unwrap();
        for i in 0..10 {
            let robot = factory.create(&format!("Robot{}", i));
            assert_eq!(robot.hello(10 - i), format!("Robot{}{}", i, 10 - i));
        }
    }
    Vec::new()
}

fml_setup!(MyContext, Preset, Some(Box::new(initiate)));
