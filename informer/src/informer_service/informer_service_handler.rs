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

use crate::{informer_notify, Connection, EventTags, Events, InformerEventSender};
use crossbeam::Receiver;
use crossbeam_channel as crossbeam;
use std::thread::spawn;

pub struct InformerService {
    connections: Vec<Connection>,
    event_receiver: Receiver<Events>,
    connection_receiver: Receiver<Connection>,
}

impl InformerService {
    pub fn new(connection_receiver: Receiver<Connection>) -> (Self, InformerEventSender) {
        let (sender, event_receiver) = informer_notify::create();
        (
            Self {
                connections: Vec::new(),
                event_receiver,
                connection_receiver,
            },
            sender,
        )
    }

    pub fn run_service(mut self) {
        let event_rcv = self.event_receiver.clone();
        let connection_rcv = self.connection_receiver.clone();
        spawn(move || {
            cinfo!(INFORMER, "Informer Service has started");
            let mut select = crossbeam::Select::new();
            let event_index = select.recv(&event_rcv);
            let connection_index = select.recv(&connection_rcv);
            loop {
                match select.ready() {
                    index if index == event_index => {
                        if let Ok(event) = event_rcv.try_recv() {
                            cinfo!(INFORMER, "Event is sent to all clients");
                            self.notify_client(event);
                        }
                    }
                    index if index == connection_index => {
                        if let Ok(connection) = connection_rcv.recv() {
                            cinfo!(INFORMER, "A new connection is added");
                            self.add_new_connection(connection);
                        }
                    }
                    _ => {
                        cerror!(INFORMER, "is not an expected index of message queue");
                    }
                }
            }
        });
    }

    pub fn add_new_connection(&mut self, connection: Connection) {
        self.connections.push(connection);
    }

    fn compare_event_types(tag: &EventTags, event: &Events) -> bool {
        match (tag, event) {
            (EventTags::PeerAdded, Events::PeerAdded(..)) => true,
        }
    }

    pub fn notify_client(&self, popup_event: Events) {
        for connection in &self.connections {
            for interested_event in connection.interested_events.clone() {
                if InformerService::compare_event_types(&interested_event, &popup_event) {
                    connection.notify_client(&popup_event);
                }
            }
        }
    }
}
