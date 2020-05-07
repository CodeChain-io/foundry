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

#[macro_use]
mod address;

pub mod action_data;
pub mod metadata;
pub mod module;
pub mod module_datum;
pub mod stake;
pub mod validator_set;

#[derive(Clone, Copy)]
#[repr(u8)]
enum Prefix {
    Metadata = b'M',
    ModuleDatum = b'S',
    Module = b'U',
}
