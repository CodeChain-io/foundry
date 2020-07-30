// Copyright 2020 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::{
    informer_notify, EventTags, Events, InformerEventSender, RateLimiter, Registration, Subscription, SubscriptionId,
};
use ccore::{BlockChainTrait, Client, EngineInfo};
use crossbeam::Receiver;
use crossbeam_channel as crossbeam;
use crpc::v1::Block as RPCBlock;
use ctypes::BlockId;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::task;

#[derive(Serialize)]
pub enum ColdEvents {
    BlockGeneration(Box<RPCBlock>),
}

pub struct InformerService {
    subscriptions: Vec<Arc<Subscription>>,
    client: Arc<Client>,
    event_receiver: Receiver<Events>,
    subscription_receiver: Receiver<Registration>,
}

impl InformerService {
    pub fn new(subscription_receiver: Receiver<Registration>, client: Arc<Client>) -> (Self, InformerEventSender) {
        let (sender, event_receiver) = informer_notify::create();
        (
            Self {
                subscriptions: Vec::new(),
                client,
                event_receiver,
                subscription_receiver,
            },
            sender,
        )
    }

    pub fn run_service(mut self) {
        cinfo!(INFORMER, "Informer service is started");
        let event_rcv = self.event_receiver.clone();
        let subscriptions_rcv = self.subscription_receiver.clone();
        thread::spawn(move || {
            let mut select = crossbeam::Select::new();
            let event_index = select.recv(&event_rcv);
            let subscription_index = select.recv(&subscriptions_rcv);
            let rt = Runtime::new().unwrap();
            loop {
                match select.ready() {
                    index if index == event_index => {
                        if let Ok(event) = event_rcv.try_recv() {
                            cinfo!(INFORMER, "Event is sent to all clients");
                            self.notify_client(event);
                        }
                    }
                    index if index == subscription_index => {
                        if let Ok(subscription) = subscriptions_rcv.recv() {
                            match subscription {
                                Registration::Register(subscribe) => {
                                    cinfo!(INFORMER, "A new subscription is added");
                                    let new_subscription = Arc::new(subscribe);
                                    self.add_new_subscription(Arc::clone(&new_subscription));
                                    let client = Arc::clone(&self.client);
                                    rt.spawn(async move {
                                        for interested_events in &new_subscription.interested_events {
                                            if let EventTags::ColdBlockGenerationNumerical(value) = interested_events {
                                                let cold_generator =
                                                    BlockCreatedEventGenerator::new(Arc::clone(&client));
                                                cold_generator.run(Arc::clone(&new_subscription), *value);
                                            }
                                        }
                                    });
                                }
                                Registration::Deregister(sub_id) => {
                                    if self
                                        .subscriptions
                                        .iter()
                                        .any(|subscription| subscription.subscription_id == sub_id.clone())
                                    {
                                        self.remove_subscription(sub_id);
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        cerror!(INFORMER, "is not an expected index of message queue");
                    }
                }
            }
        });
    }

    pub fn add_new_subscription(&mut self, subscription: Arc<Subscription>) {
        self.subscriptions.push(subscription);
    }

    pub fn remove_subscription(&mut self, sub_id: SubscriptionId) {
        let index = self
            .subscriptions
            .iter()
            .position(|subscription| subscription.subscription_id == sub_id)
            .expect("The index optionality is already checked while receiving the subscription ID ");
        let removed_subscription = self.subscriptions.remove(index);
        removed_subscription.is_subscribing.store(false, SeqCst);
        let subscription_id = &removed_subscription.subscription_id;
        if let SubscriptionId::Number(value) = subscription_id {
            cinfo!(INFORMER, "The Subscription {} would no longer supported", value);
        }
    }

    fn compare_event_types(tag: &EventTags, event: &Events) -> bool {
        match (tag, event) {
            (EventTags::PeerAdded, Events::PeerAdded(..)) => true,
            _ => false,
        }
    }

    pub fn notify_client(&self, popup_event: Events) {
        for subscription in &self.subscriptions {
            for interested_event in subscription.interested_events.clone() {
                if InformerService::compare_event_types(&interested_event, &popup_event) {
                    subscription.notify_client(&popup_event);
                }
            }
        }
    }
}

pub struct BlockCreatedEventGenerator {
    client: Arc<Client>,
    rate_limiter: RateLimiter,
}

impl BlockCreatedEventGenerator {
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            rate_limiter: RateLimiter::new(100),
        }
    }

    pub fn run(mut self, subscription: Arc<Subscription>, from_block_number: u64) -> tokio::task::JoinHandle<()> {
        let mut current_block_number = from_block_number;
        task::spawn(async move {
            loop {
                if !subscription.is_subscribing.load(SeqCst) {
                    cinfo!(INFORMER, "Cold event supports is Stopped");
                    break
                }
                let best_block_number = self.client.best_block_header().number();
                if best_block_number >= current_block_number {
                    let event = self.gen(current_block_number);
                    subscription.cold_notify(&event);
                    self.rate_limiter.acquire_ticket().await;
                    current_block_number += 1;
                } else {
                    tokio::time::delay_for(Duration::from_millis(500)).await;
                }
            }
        })
    }

    fn gen(&mut self, favorite_block_number: u64) -> ColdEvents {
        let current_id = BlockId::Number(favorite_block_number);
        let block = self.client.block(&current_id).map(|block| {
            let block = block.decode();
            RPCBlock::from_core(block, self.client.network_id())
        });
        let current_block = block.expect("The block number is already checked by the run function.");
        ColdEvents::BlockGeneration(Box::new(current_block))
    }
}
