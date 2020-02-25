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

/// This module representes all accessible interface
use crate::ibc;
use ibc::commitment_23::types::{create_membership_proof, create_non_membership_proof};
use ibc::commitment_23::{CommitmentPath, CommitmentState};
use primitives::Bytes;
use rlp::Decodable;

use crate::ibc::IdentifierSlice;

pub trait DebugName {
    fn debug_name() -> &'static str;
}

impl DebugName for ibc::connection_03::types::ConnectionEnd {
    fn debug_name() -> &'static str {
        "ConnectionEnd"
    }
}

impl DebugName for ibc::client_02::types::ClientState {
    fn debug_name() -> &'static str {
        "ClientState"
    }
}

impl DebugName for ibc::client_02::types::ConsensusState {
    fn debug_name() -> &'static str {
        "ConsensusState"
    }
}

/// Queries the path and returns the result in decoded struct
pub fn query<T>(ctx: &dyn ibc::Context, path: &CommitmentPath) -> Option<T>
where
    T: Decodable + DebugName, {
    let data = ctx.get_kv_store().get(&path.raw)?;
    // error means that state DB has stored an invalid data. (must never happen)
    Some(rlp::decode(&data).unwrap_or_else(|_| panic!(format!("Illformed {} stored in DB", T::debug_name()))))
}

/// Caller of this function should not care about the type of proof. Thus we return as Bytes
/// It may create both proof of presence and absence. Caller should be aware of which one would be.
pub fn make_proof(ctx: &dyn ibc::Context, path: &CommitmentPath) -> Bytes {
    if let Some(value) = ctx.get_kv_store().get(&path.raw) {
        let commitment_state = CommitmentState {
            kv_store: ctx.get_kv_store(),
        };
        let proof = create_membership_proof(&commitment_state, &path, &value);
        rlp::encode(&proof)
    } else {
        let commitment_state = CommitmentState {
            kv_store: ctx.get_kv_store(),
        };
        let proof = create_non_membership_proof(&commitment_state, &path);
        rlp::encode(&proof)
    }
}

pub fn path_client_state(id: IdentifierSlice) -> CommitmentPath {
    CommitmentPath {
        raw: ibc::client_02::path_client_state(id),
    }
}

pub fn path_consensus_state(id: IdentifierSlice, num: u64) -> CommitmentPath {
    CommitmentPath {
        raw: ibc::client_02::path_consensus_state(id, num),
    }
}

pub fn path_connection_end(id: IdentifierSlice) -> CommitmentPath {
    CommitmentPath {
        raw: ibc::connection_03::path(id),
    }
}
