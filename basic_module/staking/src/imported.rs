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

use fkey::Ed25519Public as Public;

pub trait AccountManager {
    fn add_balance(&self, public: &Public, val: u64);
    fn sub_balance(&self, public: &Public, val: u64) -> Result<(), String>;
    fn set_balance(&self, public: &Public, val: u64);
    fn increment_sequence(&self, public: &Public);
}

pub trait AccountView {
    fn is_active(&self, public: &Public) -> bool;
    fn get_balance(&self, public: &Public) -> u64;
    fn get_sequence(&self, public: &Public) -> u64;
}
