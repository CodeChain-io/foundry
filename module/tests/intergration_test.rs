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
extern crate foundry_process_sandbox as fproc_sndbx;

use cmodule::impls::process::*;
use cmodule::link::*;
use cmodule::sandbox::*;
use foundry_module_rt::UserModule;
use fproc_sndbx::execution::executor;
use fproc_sndbx::ipc::generate_random_name;
use remote_trait_object::{Context as RtoContext, Dispatch, HandleToExchange, Service, ToDispatcher};
use std::sync::Arc;

#[remote_trait_object_macro::service]
trait Hello: Service {
    fn hello(&self) -> i32;
    fn hi(&self) -> String;
}

struct SimpleHello {
    value: i32,
    greeting: String,
}
impl Service for SimpleHello {}
impl Hello for SimpleHello {
    fn hello(&self) -> i32 {
        self.value
    }

    fn hi(&self) -> String {
        self.greeting.clone()
    }
}

struct ModuleA {
    my_greeting: String,
    others_greeting: String,
    /// along with expected value from hello()
    hello_list: Vec<(Box<dyn Hello>, i32)>,
}

impl UserModule for ModuleA {
    fn new(arg: &[u8]) -> Self {
        let (my_greeting, others_greeting): (String, String) = serde_cbor::from_slice(arg).unwrap();
        Self {
            my_greeting,
            others_greeting,
            hello_list: Vec::new(),
        }
    }

    fn prepare_service_to_export(&mut self, ctor_name: &str, ctor_arg: &[u8]) -> Arc<dyn Dispatch> {
        assert_eq!(ctor_name, "Constructor");
        let value: i32 = serde_cbor::from_slice(ctor_arg).unwrap();
        (Box::new(SimpleHello {
            value,
            greeting: self.my_greeting.clone(),
        }) as Box<dyn Hello>)
            .to_dispatcher()
    }

    fn import_service(
        &mut self,
        rto_context: &RtoContext,
        _exporter_module: &str,
        name: &str,
        handle: HandleToExchange,
    ) {
        self.hello_list.push((remote_trait_object::import_service(rto_context, handle), name.parse().unwrap()))
    }

    fn debug(&mut self, _arg: &[u8]) -> Vec<u8> {
        for (hello, value) in &self.hello_list {
            assert_eq!(hello.hello(), *value);
            assert_eq!(hello.hi(), self.others_greeting);
        }
        Vec::new()
    }
}

fn execute_module<M: UserModule + 'static>(args: Vec<String>) {
    foundry_module_rt::start::<<SingleProcess as ExecutionScheme>::Ipc, M>(args);
}

#[test]
fn module_bootstrap1() {
    let sandboxer = ProcessSandboxer::<SingleProcess>::new();

    let name_a = generate_random_name();
    executor::add_function_pool(name_a.clone(), Arc::new(execute_module::<ModuleA>));
    let name_b = generate_random_name();
    executor::add_function_pool(name_b.clone(), Arc::new(execute_module::<ModuleA>));

    let name_a = std::path::PathBuf::from(name_a);
    let name_b = std::path::PathBuf::from(name_b);

    let n = 10;
    let exports: Vec<(String, i32)> = (0..n).map(|i| ("Constructor".to_owned(), i)).collect();
    let exports_ref: Vec<(&str, &dyn erased_serde::Serialize)> =
        exports.iter().map(|(name, i)| (name.as_str(), i as &dyn erased_serde::Serialize)).collect();

    let mut sandbox_a = sandboxer.load(&name_a, &("Annyeong", "Konnichiwa"), &exports_ref).unwrap();

    let mut sandbox_b = sandboxer.load(&name_b, &("Konnichiwa", "Annyeong"), &exports_ref).unwrap();

    let mut port_a = sandbox_a.new_port();
    let mut port_b = sandbox_b.new_port();

    let linker = ProcessLinker::<SingleProcess>::new();

    let zero_to_n: Vec<usize> = (0..n as usize).collect();
    let zero_to_n_in_string: Vec<String> = (0..n).map(|x| x.to_string()).collect();
    let zero_to_n_in_string_: Vec<&str> = zero_to_n_in_string.iter().map(|x| x.as_str()).collect();

    port_a.export(&zero_to_n);
    port_a.import(&zero_to_n_in_string_);

    port_b.export(&zero_to_n);
    port_b.import(&zero_to_n_in_string_);

    linker.link(&mut *port_a, &mut *port_b).unwrap();

    sandbox_a.debug(&[]);
    sandbox_b.debug(&[]);

    drop(port_a);
    drop(port_b);
}
