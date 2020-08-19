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

use ccore::BlockChainClient;
use ccore::Client;
use ctypes::BlockId;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

pub fn check_block_nums(client: Arc<Client>) {
    let last_num = client.block_number(&BlockId::Latest).unwrap();

    for _ in 0..10000 {
        let num = client.block_number(&BlockId::Latest).unwrap();
        if num >= last_num + 5 {
            return
        }
        sleep(Duration::from_millis(10));
    }
    panic!("Chain is not growing")
}
