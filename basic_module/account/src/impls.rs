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

use super::core::*;
use crate::account::{
    add_balance as add_balance_internalliy, get_account, get_balance as get_balance_internally,
    get_sequence as get_sequence_internally, sub_balance as sub_balance_internalliy,
};
use crate::import::{fee_manager, signature_manager};
use crate::{check, get_context, Action, SignedTransaction};
use coordinator::context::SubStorageAccess;
pub use coordinator::types::ErrorCode;
use ckey::{sign as sign_ed25519, verify as verify_ed25519};

#[allow(dead_code)]
pub struct Handler {}

impl CheckTxHandler for Handler {
    fn check_transaction(&self, tx: &Transaction) -> Result<(), ErrorCode> {
        if get_balance_internally(&tx.action.sender) < tx.fee + tx.action.quantity || get_sequence_internally(&tx.action.sender) <= tx.seq {
            return Err(-1)
        }
        Ok(())
    }
}
