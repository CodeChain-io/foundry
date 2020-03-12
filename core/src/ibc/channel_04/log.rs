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

use super::types::Packet;
/// Nasty version of blockchain log event system.
/// In PoC, we consider only ORDERED channel.
/// Thus, it is enough to store only latest packet per channel.
/// Since we have no log system yet, we just store it in state DB.
///
use crate::ibc;
use ibc::IdentifierSlice;

pub fn set_packet<'a>(
    ctx: &'a mut dyn ibc::Context,
    port: IdentifierSlice,
    channel: IdentifierSlice,
    packet: &Packet,
    tag: &str,
) {
    let value = rlp::encode(packet);
    let path = format!("nastylogs/{}/{}/{}/latest", port, channel, tag);
    let result = ctx.get_kv_store_mut().insert(&path, &value);
    if result.is_some() {
        panic!("Packet already exists.");
    }
}

#[allow(dead_code)]
pub fn get_packet<'a>(
    ctx: &'a mut dyn ibc::Context,
    port: IdentifierSlice,
    channel: IdentifierSlice,
    tag: &str,
) -> Option<Packet> {
    let path = format!("nastylogs/{}/{}/{}/latest", port, channel, tag);
    Some(rlp::decode(&ctx.get_kv_store().get(&path)?).expect("Illformed Packet stored in state DB"))
}

pub fn remove_packet<'a>(ctx: &'a mut dyn ibc::Context, port: IdentifierSlice, channel: IdentifierSlice, tag: &str) {
    let path = format!("nastylogs/{}/{}/{}/latest", port, channel, tag);
    ctx.get_kv_store_mut().remove(&path).expect("Invalid call to remove_packet()");
}
