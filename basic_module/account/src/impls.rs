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

use crate::check;
use crate::core::CheckTxHandler;
use crate::internal::get_sequence;
use crate::types::{Action, SignedTransaction};
pub use coordinator::types::ErrorCode;

#[allow(dead_code)]
pub struct Handler {}

impl CheckTxHandler for Handler {
    fn check_transaction(&self, signed_tx: &SignedTransaction) -> Result<(), ErrorCode> {
        check(signed_tx);

        let Action::Pay {
            sender,
            receiver: _,
            quantity: _,
        } = signed_tx.tx.action;
        if get_sequence(&sender) > signed_tx.tx.seq {
            return Err(0xFFFF_FFFF)
        }

        Ok(())
    }
}
