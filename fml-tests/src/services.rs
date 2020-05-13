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

// Common dependencis for services
extern crate codechain_fml as fml;
use fml::service_prelude::*;
pub struct TraitHolder<T: ?Sized>(std::marker::PhantomData<T>);

#[fml_macro::service]
pub trait HelloFactory: Service {
    fn create(&self, name: &String) -> Box<dyn HelloRobot>;
}

#[fml_macro::service]
pub trait HelloRobot: Service {
    fn hello(&self, flag: i32) -> String;
}

#[derive(PartialEq, serde::Serialize, serde::Deserialize, Debug)]
pub enum Answer {
    Next(String),
    End(String),
}

/// module index -> (caller module index -> availiable handlers)
pub type AvailiableMap = Vec<Vec<usize>>;

pub fn new_avail_map(size: usize, value: usize) -> AvailiableMap {
    let mut result = vec![Vec::new(); size];
    for (i, x) in result.iter_mut().enumerate() {
        for j in 0..size {
            if i == j {
                x.push(0);
            } else {
                x.push(value);
            }
        }
    }
    result
}

#[fml_macro::service]
pub trait Schedule: Service {
    /// Get the schedule. It is then locked.
    fn get(&self) -> AvailiableMap;

    /// Set the schedule. It is then unlocked.
    fn set(&self, s: AvailiableMap);
}

#[fml_macro::service]
pub trait RelayerFactory: Service {
    /// Make an invitation for a single visit toward itself
    fn create(&self, key: String, current: usize, destination: String) -> Box<dyn RelayerMachine>;

    /// Returns name of the next module to visit
    fn ask_path(&self, key: String, current: usize) -> Answer;
}

#[fml_macro::service]
pub trait RelayerMachine: Service {
    /// Recursively traverse all the path and query the answer for the destination
    fn run(&self) -> String;
}
