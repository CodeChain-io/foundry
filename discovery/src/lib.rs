// Copyright 2018-2020 Kodebox, Inc.
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

#![allow(deprecated)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate codechain_logger as clogger;

mod config;
mod extension;
mod message;
mod node_id;

const K: u8 = 16;
const T_REFRESH: u32 = 60_000;

pub use crate::config::Config;
pub use crate::extension::Extension as Discovery;
