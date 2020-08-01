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

use crate::link::{self, Linkable, Linker, Port, LINKERS};
use crate::sandbox::{self, Sandbox, Sandboxer};
use crossbeam::thread;
use foundry_module_rt::coordinator_interface::{FoundryModule, PartialRtoConfig};
use fproc_sndbx::execution::executor;
use fproc_sndbx::ipc::Ipc;
use linkme::distributed_slice;
use parking_lot::Mutex;
use remote_trait_object::raw_exchange::HandleToExchange;
use remote_trait_object::{Config as RtoConfig, Context as RtoContext, ServiceToImport};
use std::io::Cursor;
use std::marker::PhantomData;
use std::path::Path;
use std::sync::Arc;

#[distributed_slice(LINKERS)]
fn single_process_linker() -> (&'static str, Arc<dyn Linker>) {
    ("single-process", Arc::new(ProcessLinker::<SingleProcess>::new()))
}

#[distributed_slice(LINKERS)]
fn multi_process_linker() -> (&'static str, Arc<dyn Linker>) {
    ("multi-process", Arc::new(ProcessLinker::<MultiProcess>::new()))
}

/// ProcessSandboxer is really trivial, because there is nothing really to do
/// for the processes. It just creates ProcessSandbox by demand, and let it just be.
/// It doesn't even hold its sandboxes.
pub struct ProcessSandboxer<E: ExecutionScheme> {
    _p: PhantomData<E>,
}

impl<E: ExecutionScheme> Sandboxer for ProcessSandboxer<E> {
    fn id(&self) -> &'static str {
        unimplemented!()
    }

    fn supported_module_types(&self) -> &'static [&'static str] {
        unimplemented!()
    }

    fn load<'a>(
        &self,
        path: &'a dyn AsRef<Path>,
        init: &dyn erased_serde::Serialize,
        exports: &[(&str, &dyn erased_serde::Serialize)],
    ) -> Result<Box<dyn Sandbox>, sandbox::Error<'a>> {
        let mut init_buffer = Vec::<u8>::new();
        let cbor = &mut serde_cbor::Serializer::new(serde_cbor::ser::IoWrite::new(Cursor::new(&mut init_buffer)));
        init.erased_serialize(&mut erased_serde::Serializer::erase(cbor)).unwrap();

        let exports: Vec<(String, Vec<u8>)> = exports
            .iter()
            .map(|(name, data)| {
                let mut buffer = Vec::<u8>::new();
                let cbor = &mut serde_cbor::Serializer::new(serde_cbor::ser::IoWrite::new(Cursor::new(&mut buffer)));
                data.erased_serialize(&mut erased_serde::Serializer::erase(cbor)).unwrap();
                (name.to_string(), buffer)
            })
            .collect();

        Ok(Box::new(ProcessSandbox::<E>::new(path.as_ref(), &init_buffer, &exports)?))
    }
}

impl<E: ExecutionScheme> ProcessSandboxer<E> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            _p: PhantomData,
        }
    }
}

pub trait ExecutionScheme: Send + Sync + 'static {
    type Ipc: fproc_sndbx::ipc::Ipc + 'static;
    type Execution: executor::Executor;
    fn is_intra() -> bool;
}

pub struct MultiProcess;
impl ExecutionScheme for MultiProcess {
    type Ipc = fproc_sndbx::ipc::unix_socket::DomainSocket;
    type Execution = fproc_sndbx::execution::executor::Executable;
    fn is_intra() -> bool {
        false
    }
}

pub struct SingleProcess;
impl ExecutionScheme for SingleProcess {
    type Ipc = fproc_sndbx::ipc::intra::Intra;
    type Execution = fproc_sndbx::execution::executor::PlainThread;
    fn is_intra() -> bool {
        true
    }
}

pub struct ProcessSandbox<E: ExecutionScheme> {
    _process: Mutex<executor::Context<E::Ipc, E::Execution>>,
    /// module should be dropped first before rto_context
    module: Box<dyn foundry_module_rt::coordinator_interface::FoundryModule>,
    rto_context: remote_trait_object::Context,
}

impl<E: ExecutionScheme> ProcessSandbox<E> {
    fn new<'a>(path: &'a Path, init: &[u8], exports: &[(String, Vec<u8>)]) -> Result<Self, sandbox::Error<'a>> {
        let mut process =
            executor::execute::<E::Ipc, E::Execution>(path.to_str().ok_or_else(|| sandbox::Error::ModuleNotFound {
                path,
            })?)
            .map_err(|_| sandbox::Error::ModuleNotFound {
                path,
            })?;

        // TODO: parse init to get proper rto config
        let rto_config = RtoConfig::default_setup();
        let (transport_send, transport_recv) = process.ipc.take().unwrap().split();

        let (rto_context, module): (_, ServiceToImport<dyn FoundryModule>) =
            RtoContext::with_initial_service_import(rto_config, transport_send, transport_recv);
        let mut module: Box<dyn FoundryModule> = module.into_remote();
        module.initialize(init, exports);

        Ok(Self {
            _process: Mutex::new(process),
            rto_context,
            module,
        })
    }
}

impl<E: ExecutionScheme> Sandbox for ProcessSandbox<E> {
    fn sandboxer(&self) -> Arc<dyn Sandboxer> {
        unimplemented!()
    }

    fn debug(&mut self, arg: &[u8]) -> Vec<u8> {
        self.module.debug(arg)
    }
}

impl<E: ExecutionScheme> Linkable for ProcessSandbox<E> {
    fn supported_linkers(&self) -> &'static [&'static str] {
        &["single-process-linker", "multi-process-linker"]
    }

    fn new_port(&mut self) -> Box<dyn Port> {
        // TODO: use module name.
        // It MUST be unique anyway, for now.
        let random_name = fproc_sndbx::ipc::generate_random_name();
        Box::new(ProcessPort {
            module_side_port: self.module.create_port(&random_name).unwrap_import().into_remote(),
            ids: Vec::new(),
            slots: Vec::new(),
        })
    }

    fn seal(&mut self) {}
}

impl<E: ExecutionScheme> Drop for ProcessSandbox<E> {
    fn drop(&mut self) {
        self.rto_context.disable_garbage_collection();
        self.module.shutdown();
    }
}

pub struct ProcessPort {
    module_side_port: Box<dyn foundry_module_rt::coordinator_interface::Port>,
    ids: Vec<usize>,
    slots: Vec<String>,
}

impl Port for ProcessPort {
    fn export(&mut self, ids: &[usize]) {
        self.ids = ids.to_vec()
    }

    fn import(&mut self, slots: &[&str]) {
        self.slots = slots.iter().map(|x| x.to_string()).collect();
    }
}

impl ProcessPort {
    fn initialize(&mut self, rto_config: PartialRtoConfig, ipc_arg: Vec<u8>, intra: bool) {
        self.module_side_port.initialize(rto_config, ipc_arg, intra);
    }
}

pub struct ProcessLinker<E: ExecutionScheme> {
    _p: PhantomData<E>,
}

impl<E: ExecutionScheme> ProcessLinker<E> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            _p: PhantomData,
        }
    }
}

impl<E: ExecutionScheme> Linker for ProcessLinker<E> {
    fn link(&self, a: &mut dyn Port, b: &mut dyn Port) -> Result<(), link::Error> {
        let port_a: &mut ProcessPort = a.mut_any().downcast_mut().ok_or_else(|| link::Error::UnsupportedPortType {
            id: "Unknown",
        })?;
        let port_b: &mut ProcessPort = b.mut_any().downcast_mut().ok_or_else(|| link::Error::UnsupportedPortType {
            id: "Unknown",
        })?;

        let (ipc_arg_a, ipc_arg_b) = E::Ipc::arguments_for_both_ends();

        // TODO: get config from the module itself
        let rto_config_a = PartialRtoConfig::from_rto_config(RtoConfig::default_setup());
        let rto_config_b = PartialRtoConfig::from_rto_config(RtoConfig::default_setup());

        thread::scope(|s| {
            // two initialize()s must be called concurrently
            let j = s.spawn(|_| {
                port_a.initialize(rto_config_a, ipc_arg_a, E::is_intra());
            });
            port_b.initialize(rto_config_b, ipc_arg_b, E::is_intra());
            j.join().unwrap();
        })
        .unwrap();

        let handles_a_to_b = port_a.module_side_port.export(&port_a.ids);
        let handles_b_to_a = port_b.module_side_port.export(&port_b.ids);

        assert_eq!(handles_a_to_b.len(), port_b.slots.len());
        assert_eq!(handles_b_to_a.len(), port_a.slots.len());

        let handles_b_to_a: Vec<(String, HandleToExchange)> =
            port_a.slots.iter().map(|x| x.to_owned()).zip(handles_b_to_a.into_iter()).collect();
        let handles_a_to_b: Vec<(String, HandleToExchange)> =
            port_b.slots.iter().map(|x| x.to_owned()).zip(handles_a_to_b.into_iter()).collect();

        port_a.module_side_port.import(&handles_b_to_a);
        port_b.module_side_port.import(&handles_a_to_b);

        Ok(())
    }
}
