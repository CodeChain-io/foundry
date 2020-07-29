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

use super::block_info::{BestBlockChanged, BestHeaderChanged};
use ctypes::BlockHash;

#[derive(Debug, PartialEq)]
pub struct ChainUpdateResult {
    // Some(updated_best_block_hash) if chain is updated
    // None if chain is not updated
    pub enacted: Option<BlockHash>,
}

impl ChainUpdateResult {
    pub fn new(best_block_changed: &BestBlockChanged) -> Self {
        match best_block_changed {
            BestBlockChanged::CanonChainAppended {
                ..
            } => {
                let enacted = Some(best_block_changed.new_best_hash().unwrap());
                ChainUpdateResult {
                    enacted,
                }
            }
            BestBlockChanged::None => ChainUpdateResult {
                enacted: None,
            },
        }
    }

    pub fn new_from_best_header_changed(best_header_changed: &BestHeaderChanged) -> Self {
        match best_header_changed {
            BestHeaderChanged::CanonChainAppended {
                ..
            } => {
                let enacted = Some(best_header_changed.new_best_hash().unwrap());
                ChainUpdateResult {
                    enacted,
                }
            }
            BestHeaderChanged::None => ChainUpdateResult {
                enacted: None,
            },
        }
    }

    pub fn none() -> Self {
        ChainUpdateResult {
            enacted: None,
        }
    }

    pub fn is_none(&self) -> bool {
        self.enacted.is_none()
    }
}
