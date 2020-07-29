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

use ctypes::BlockHash;

/// Represents what has to be handled by actor listening to chain events
pub trait ChainNotify: Send + Sync {
    /// fires when chain has new headers.
    fn new_headers(&self, _imported: Vec<BlockHash>, _enacted: Vec<BlockHash>, _new_best_proposal: Option<BlockHash>) {
        // does nothing by default
    }

    /// fires when chain has new blocks.
    fn new_blocks(&self, _imported: Vec<BlockHash>, _invalid: Vec<BlockHash>, _enacted: Vec<BlockHash>) {
        // does nothing by default
    }
}
