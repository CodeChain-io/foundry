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

use crate::cmodule::link::{Linkable, Port};
use crate::cmodule::sandbox::Sandbox;
use crate::execution::executor;
use crate::ipc::domain_socket::DomainSocket;
use crate::ipc::Ipc;
use crate::link::BasePort;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone)]
struct BaseRelayerContext {
    socket: Arc<DomainSocket>,
    port: Arc<BasePort>,
}

/// Sandboxee
/// <- (IPC) ->
/// Sandbox
/// <- (BaseRelayer) ->
/// BasePort
/// <- (BaseLinker's scheme) -> Host [Ends in this case] OR
/// Another BasePort [Keeps going]
/// <- (BaseRelayer) ->
/// Another Sandbox
/// <- (IPC) ->
/// Another Sandboxee
struct BaseRelayer {
    context: BaseRelayerContext,
    receiver: Option<thread::JoinHandle<()>>,
    sender: Option<thread::JoinHandle<()>>,
}

impl BaseRelayer {
    fn receive(context: &BaseRelayerContext) {
        loop {
            let data = context.socket.recv();
            if data == b"#TERMINATE" {
                return
            }
            context.port.send(data);
        }
    }

    fn send(context: &BaseRelayerContext) {
        loop {
            let data = context.port.recv();
            if data == b"#TERMINATE" {
                return
            }
            context.socket.send(&data);
        }
    }
}

impl BaseRelayer {
    pub fn new(context: BaseRelayerContext) -> BaseRelayer {
        let context1 = context.clone();
        let context2 = context.clone();
        BaseRelayer {
            context,
            receiver: Some(thread::spawn(move || BaseRelayer::receive(&context1))),
            sender: Some(thread::spawn(move || BaseRelayer::send(&context2))),
        }
    }
}

impl Drop for BaseRelayer {
    fn drop(&mut self) {
        self.sender.take().unwrap().join().unwrap();
        self.receiver.take().unwrap().join().unwrap();
    }
}

pub struct BaseSandbox {
    boxer: String,
    relayer: BaseRelayer,
    process: executor::Context<DomainSocket>,
    baseport: Arc<BasePort>,
    ports: Vec<Arc<dyn Port>>,
}

impl BaseSandbox {
    pub fn new(path: String, id: String, boxer: String) -> Self {
        let process = executor::execute::<DomainSocket>(&path, &id).unwrap();
        let baseport = Arc::new(BasePort::new(64));
        let relayer = BaseRelayer::new(BaseRelayerContext {
            socket: process.ipc.clone(),
            port: baseport.clone(),
        });
        BaseSandbox {
            boxer,
            relayer,
            process,
            baseport,
            ports: Vec::new(),
        }
    }
}

impl Linkable for BaseSandbox {
    fn supported_linkers(&self) -> &'static [&'static str] {
        static L: [&'static str; 1] = ["Base"];
        &L
    }

    fn new_port(&mut self) -> Arc<dyn Port> {
        Arc::new(BasePort::new(64))
    }
}

impl Sandbox for BaseSandbox {
    fn sandboxer(&self) -> &str {
        &self.boxer
    }
}
