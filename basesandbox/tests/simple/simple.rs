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

extern crate codechain_basesandbox as cbsb;
use cbsb::execution::executee;
use cbsb::ipc::{IpcRecv, IpcSend};
use std::time::Duration;

type IpcScheme = cbsb::ipc::servo_channel::ServoChannel;

#[cfg(all(unix, target_arch = "x86_64"))]
fn main() -> Result<(), String> {
    let args = std::env::args().collect();
    let ctx = executee::start::<IpcScheme>(args);
    let r = ctx.ipc.as_ref().unwrap().recv(Some(Duration::from_millis(100))).unwrap();
    assert_eq!(r, b"Hello?\0");
    ctx.ipc.as_ref().unwrap().send(b"I'm here!\0");
    ctx.terminate();
    Ok(())
}
