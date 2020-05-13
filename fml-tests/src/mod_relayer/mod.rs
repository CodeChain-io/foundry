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
use rand::{rngs::StdRng, Rng};
use std::collections::HashMap;
use std::sync::RwLock;
use std::thread;

pub struct MyContext {
    /// total number of relayers
    number: usize,
    /// My index
    index: usize,
    schedule: RwLock<Option<Box<dyn Schedule>>>,
    factories: RwLock<HashMap<String, Box<dyn RelayerFactory>>>,
    answers: RwLock<HashMap<String, (Vec<String>, String)>>,
}

impl fml::Custom for MyContext {
    fn new(context: &fml::Config) -> Self {
        let (number, index) = serde_cbor::from_slice(&context.args).unwrap();
        let mut factories = HashMap::new();
        factories.insert(
            context.id.clone(),
            Box::new(OrdinaryFactory {
                handle: Default::default(),
            }) as Box<dyn RelayerFactory>,
        );

        MyContext {
            number,
            index,
            schedule: Default::default(),
            factories: RwLock::new(factories),
            answers: Default::default(),
        }
    }
}

pub struct Preset;

impl HandlePreset for Preset {
    fn export() -> Vec<HandleExchange> {
        let ctx = get_context();
        let number = ctx.custom.number;
        let mut exchanges = Vec::<HandleExchange>::new();

        for i in 0..number {
            let name = format!("Module{}", i);
            if name == get_context().config.id {
                // myself
                continue
            }
            let id = TraitHolder::<dyn RelayerFactory>::export(
                ctx.ports.read().unwrap().find(&format!("Module{}", i)).unwrap(),
                Box::new(OrdinaryFactory {
                    handle: Default::default(),
                }),
            );
            exchanges.push(HandleExchange {
                exporter: ctx.config.id.clone(),
                importer: name,
                handles: vec![id],
                argument: Vec::new(),
            })
        }
        exchanges
    }

    fn import(mut exchange: HandleExchange) {
        let ctx = get_context();
        assert_eq!(exchange.importer, ctx.config.id, "Invalid import request");
        if exchange.exporter == "Schedule" {
            assert_eq!(exchange.handles.len(), 1);
            ctx.custom
                .schedule
                .write()
                .unwrap()
                .replace(TraitHolder::<dyn Schedule>::import(exchange.handles.pop().unwrap()));
        } else {
            let mut guard = ctx.custom.factories.write().unwrap();
            assert_eq!(exchange.handles.len(), 1);
            let h = TraitHolder::<dyn RelayerFactory>::import(exchange.handles.pop().unwrap());
            guard.insert(exchange.exporter, h);
        }
    }
}

pub fn initiate(_arg: Vec<u8>) -> Vec<u8> {
    let my_factory = OrdinaryFactory {
        handle: Default::default(),
    };

    let mut rng: StdRng = rand::SeedableRng::from_entropy();

    let ctx = get_context();
    let iteration = 32;
    let parllel: usize = 2;
    let my_index = ctx.custom.index;
    let number = ctx.custom.number;

    for _ in 0..iteration {
        let mut used_map_list = HashMap::new();
        let mut paths = HashMap::new();
        for i in 0..parllel {
            let mut used_map = new_avail_map(ctx.custom.number, 0);
            let avail = ctx.custom.schedule.read().unwrap().as_ref().unwrap().get();
            // RelayerFactory::ask_path will require one thread always
            let mut at_least_1_for_all = true;
            for j in 0..number {
                if j == my_index {
                    continue
                }
                if avail[my_index][j] < 1 {
                    at_least_1_for_all = false;
                    break
                }
            }
            if !at_least_1_for_all {
                ctx.custom.schedule.read().unwrap().as_ref().unwrap().set(avail);
                continue
            }
            let mut avail = avail;
            for j in 0..number {
                if j == my_index {
                    continue
                }
                avail[my_index][j] -= 1;
                used_map[my_index][j] += 1;
            }

            // path generation
            let mut path = Vec::new();
            let mut last = my_index;
            for _ in 0..30 {
                let mut suc = false;
                for _ in 0..5 {
                    let next = rng.gen_range(0, number);
                    // no consumption of thread here (See how we set the factory handle for itself)
                    if next == last {
                        path.push(format!("Module{}", next));
                        suc = true;
                        break
                    } else if avail[next][last] > 0 {
                        path.push(format!("Module{}", next));
                        avail[next][last] -= 1;
                        used_map[next][last] += 1;
                        last = next;
                        suc = true;
                        break
                    }
                }
                if !suc {
                    break
                }
            }
            path.insert(0, ctx.config.id.clone());
            let key = format!("Key{}", i);
            paths.insert(key.clone(), path);
            used_map_list.insert(key.clone(), used_map);

            ctx.custom.schedule.read().unwrap().as_ref().unwrap().set(avail);
        }

        {
            let mut guard_answers = get_context().custom.answers.write().unwrap();
            guard_answers.clear();
            for (key, path) in paths.drain() {
                guard_answers.insert(key, (path, format!("{}", rng.gen_range(0, 10000))));
            }
        }

        let guard_answers = get_context().custom.answers.read().unwrap();
        let guard_factory = get_context().custom.factories.read().unwrap();
        let mut runners = Vec::new();
        for (key, (path, answer)) in &*guard_answers {
            if path.len() < 2 {
                continue
            }
            if let Answer::Next(next) = my_factory.ask_path(key.clone(), 0) {
                let machine = guard_factory.get(&next).unwrap().create(key.clone(), 0, get_context().config.id.clone());

                // Important: if you spawn a thread, you must set an instance key explicitly.
                let instance_key = get_key();
                runners.push((
                    key.clone(),
                    thread::spawn(move || {
                        set_key(instance_key);
                        machine.run()
                    }),
                    answer.clone(),
                ));
            } else {
                panic!("Test illformed")
            }
        }

        while let Some((key, guess, answer)) = runners.pop() {
            assert_eq!(guess.join().unwrap(), answer);
            let mut avail = ctx.custom.schedule.read().unwrap().as_ref().unwrap().get();

            for (avail_sub_list, used_sub_list) in avail.iter_mut().zip(used_map_list.remove(&key).unwrap().into_iter())
            {
                for (avail_entry, used_entry) in avail_sub_list.iter_mut().zip(used_sub_list.iter()) {
                    *avail_entry += *used_entry;
                }
            }
            ctx.custom.schedule.read().unwrap().as_ref().unwrap().set(avail.clone());
        }
    }
    Vec::new()
}

fml_setup!(MyContext, Preset, Some(Box::new(initiate)));
