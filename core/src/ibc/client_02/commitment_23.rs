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

use crate::ibc::commitment_23::*;
use merkle_trie::proof::{verify, CryptoProofUnit};
use primitives::Bytes;

/// Counterparty's commitment is purely client's interest.
/// This trivial re-declaration iss due to our special IBC context: Foundry-Foundry
/// If the client is actually for the foregin chain, here must be definitions for
/// specific types which are dedicated to verify_membership() and verify_non_membership()
pub(in crate::ibc::client_02) type CommitmentPathCounter = CommitmentPath;
#[allow(dead_code)]
pub(in crate::ibc::client_02) type CommitmentPrefixCounter = CommitmentPrefix;
pub(in crate::ibc::client_02) type CommitmentProofCounter = CommitmentProof;
pub(in crate::ibc::client_02) type CommitmentRootCounter = CommitmentRoot;

pub(in crate::ibc::client_02) fn verify_membership(
    root: &CommitmentRootCounter,
    proof: &CommitmentProofCounter,
    path: CommitmentPathCounter,
    value: Bytes,
) -> bool {
    let unit = CryptoProofUnit {
        root: root.raw,
        key: path.raw.into_bytes(),
        value: Some(value),
    };
    verify(&proof.raw, &unit)
}

#[allow(dead_code)]
pub(in crate::ibc::client_02) fn verify_non_membership(
    root: &CommitmentRootCounter,
    proof: &CommitmentProofCounter,
    path: CommitmentPathCounter,
) -> bool {
    let unit = CryptoProofUnit {
        root: root.raw,
        key: path.raw.into_bytes(),
        value: None,
    };
    verify(&proof.raw, &unit)
}
