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
use crate::ibc::context::TopLevelKVStore;
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
    let key = TopLevelKVStore::key(&path.raw);
    let unit = CryptoProofUnit {
        root: root.raw,
        key: key.to_vec(),
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
    let key = TopLevelKVStore::key(&path.raw);
    let unit = CryptoProofUnit {
        root: root.raw,
        key: key.to_vec(),
        value: None,
    };
    verify(&proof.raw, &unit)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ibc::commitment_23::types::create_membership_proof;
    use crate::ibc::context::{Context, TopLevelContext};
    use cstate::tests::helpers::get_temp_state;
    use cstate::{StateWithCache, TopState};

    #[test]
    fn test_verify() {
        let data = rlp::encode(&b"data".to_vec());
        let mut state = {
            let mut state = get_temp_state();

            state.update_ibc_data(&TopLevelKVStore::key("0"), data.clone()).unwrap();
            state.commit().unwrap();
            state
        };
        let state_root = state.root();

        let context = TopLevelContext::new(&mut state, 1);

        let commitment_state = CommitmentState {
            kv_store: context.get_kv_store(),
        };
        let membership_proof = create_membership_proof(
            &commitment_state,
            &CommitmentPath {
                raw: "0".to_owned(),
            },
            &data,
        );

        assert!(verify_membership(
            &CommitmentRoot {
                raw: state_root,
            },
            &membership_proof,
            CommitmentPath {
                raw: "0".to_owned(),
            },
            data,
        ));
    }
}
