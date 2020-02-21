// Copyright 2019-2020 Kodebox, Inc.
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

use crate::ibc;
use ibc::kv_store::KVStore;
use merkle_trie::proof::CryptoProof;
use primitives::{Bytes, H256};
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

pub struct CommitmentState<'a> {
    pub kv_store: &'a dyn KVStore,
}

#[derive(RlpEncodable, RlpDecodable, Clone, PartialEq, Debug)]
pub struct CommitmentRoot {
    pub raw: H256,
}

#[derive(RlpEncodable, RlpDecodable, Clone, PartialEq, Debug)]
pub struct CommitmentPath {
    pub raw: String,
}

#[derive(RlpEncodable, RlpDecodable, Clone, PartialEq, Debug)]
pub struct CommitmentPrefix {
    pub raw: String,
}

#[derive(Debug)]
pub struct CommitmentProof {
    pub raw: CryptoProof,
}

// TODO: Instead just make CryptoProof RlpEn/Decodable in rust-merkle-trie
impl Encodable for CommitmentProof {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.append_single_value(&(self.raw).0);
    }
}
impl Decodable for CommitmentProof {
    fn decode(rlp: &Rlp<'_>) -> Result<Self, DecoderError> {
        let result: Vec<Bytes> = rlp.as_val()?;
        Ok(Self {
            raw: CryptoProof(result),
        })
    }
}

pub fn get_commiment_prefix() -> CommitmentPrefix {
    CommitmentPrefix {
        raw: "".to_owned(),
    }
}

pub fn calculate_root(commitment_state: &CommitmentState) -> CommitmentRoot {
    CommitmentRoot {
        raw: commitment_state.kv_store.root(),
    }
}

// You must be aware of whether there is a value for a given key before you call create_*_proof().
pub fn create_membership_proof(
    commitment_state: &CommitmentState,
    path: &CommitmentPath,
    value: &[u8],
) -> CommitmentProof {
    let (proof_unit, proof) = commitment_state.kv_store.make_proof(&path.raw);
    assert!(proof_unit.value.is_some());
    CommitmentProof {
        raw: proof,
    }
}

pub fn create_non_membership_proof(commitment_state: &CommitmentState, path: &CommitmentPath) -> CommitmentProof {
    let (proof_unit, proof) = commitment_state.kv_store.make_proof(&path.raw);
    assert!(proof_unit.value.is_none());
    CommitmentProof {
        raw: proof,
    }
}
