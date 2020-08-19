// Copyright 2018, 2020 Kodebox, Inc.
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

mod basic;

use ccore::Client;
use std::sync::Arc;

pub fn handle_test_command(cmd: &str, client: Arc<Client>) {
    match cmd {
        "check_block_nums" => basic::check_block_nums(client),
        _ => panic!(),
    }
}

#[test]
fn check_block_nums() {
    super::run_node(&clap::ArgMatches::new(), Some("check_block_nums")).unwrap()
}
