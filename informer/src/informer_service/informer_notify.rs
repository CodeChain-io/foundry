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
use crate::Events;
use crossbeam::{bounded, unbounded, Receiver, Sender};
use crossbeam_channel as crossbeam;

pub fn create() -> (InformerEventSender, Receiver<Events>) {
    // FIXME: unbound would cause memory leak.
    // FIXME: The full queue should be handled.
    // This will be fixed soon.
    let (tx, rx) = unbounded();
    let event_sender = tx;
    (
        InformerEventSender {
            event_sender,
        },
        rx,
    )
}

#[derive(Clone)]
pub struct InformerEventSender {
    event_sender: Sender<Events>,
}

impl InformerEventSender {
    pub fn notify(&self, event: Events) {
        let guard = &self.event_sender;
        if let Some(event_sender) = Some(guard) {
            // TODO: Ignore the error. Receiver thread might be terminated or congested.
            let _ = event_sender.try_send(event);
        } else {
            // TODO: ReceiverCanceller would dropped.
        }
    }
    // FIXME: Handle error from try sender
    pub fn null_notifier() -> Self {
        let (sender, receiver) = bounded(0);
        std::mem::drop(receiver);
        Self {
            event_sender: sender,
        }
    }
}
