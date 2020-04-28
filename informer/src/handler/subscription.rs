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

use crate::{ColdEvents, EventTags, Events, Params, Sink, SubscriptionId};
use jsonrpc_core::futures::Future;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[derive(Clone)]
enum ConnectionState {
    Connected,
}

#[derive(Clone)]
pub struct Subscription {
    status: ConnectionState,
    pub subscription_id: SubscriptionId,
    pub interested_events: Vec<EventTags>,
    sink: Sink,
    pub is_subscribing: Arc<AtomicBool>,
}

impl Subscription {
    pub fn new(sink: Sink, sub_id: SubscriptionId) -> Self {
        Self {
            status: ConnectionState::Connected,
            subscription_id: sub_id,
            interested_events: Vec::new(),
            sink,
            is_subscribing: Arc::new(AtomicBool::new(true)),
        }
    }
    pub fn add_events(&mut self, params: Vec<String>) {
        match params[0].as_str() {
            "PeerAdded" => {
                let event = EventTags::PeerAdded;
                cinfo!(INFORMER, "The event is successfully added to user's interested events");
                self.interested_events.push(event);
            }
            "BlockGeneration_by_number" => {
                let cold_event = EventTags::ColdBlockGenerationNumerical(
                    // FIXME: Handle Unvalid block number
                    params[1].as_str().parse().unwrap(),
                );
                cinfo!(INFORMER, "The event is successfully added to user's interested events");
                self.interested_events.push(cold_event);
            }
            "BlockGeneration_by_hash" => {
                let cold_event = EventTags::ColdBlockGenerationHash(params[1].clone());
                cinfo!(INFORMER, "The event is successfully added to user's interested events");
                self.interested_events.push(cold_event);
            }
            _ => {
                cinfo!(INFORMER, "invalid Event: the event is not supported");
            }
        }
    }

    pub fn cold_notify(&self, event: &ColdEvents) {
        let json_object = serde_json::to_value(event).expect("event has no non-string key").as_object_mut().cloned();
        let params = Params::Map(json_object.expect("Event is serialized as object"));
        match self.status {
            // FIXME : We should use `.await` instead of wait. The standard Future is not supported by the current paritytech/jsonrpc crate.
            ConnectionState::Connected => match self.sink.notify(params).wait() {
                Ok(_) => {}
                Err(_) => {
                    cinfo!(INFORMER, "Subscription has ended, finishing.");
                }
            },
        }
    }

    pub fn notify_client(&self, event: &Events) {
        let json_object = serde_json::to_value(event).expect("json format is not valid").as_object_mut().cloned();
        let params = Params::Map(json_object.expect("Event is serialized as object"));
        match self.status {
            // FIXME : We should use `.await` instead of wait. The standard Future is not supported by the current paritytech/jsonrpc crate.
            ConnectionState::Connected => match self.sink.notify(params).wait() {
                Ok(_) => {}
                Err(_) => {
                    cinfo!(INFORMER, "Subscription has ended, finishing.");
                }
            },
        }
    }
}
