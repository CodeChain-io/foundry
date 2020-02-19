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

use super::{ClientState, Seal, UpdateHeader};
use ckey::verify_schnorr;
pub use ctypes::BlockNumber;
use ctypes::CompactValidatorSet;
use primitives::H256;

pub fn verify_signature(block_hash: H256, vset: &CompactValidatorSet, seal: &Seal) -> bool {
    let n = vset.len();

    for (index, sign) in &seal.0 {
        if *index >= n {
            return false
        }
        match verify_schnorr(&vset[*index].public_key, &sign, &block_hash) {
            Ok(true) => (),
            Ok(false) => return false,
            _ => return false,
        };
    }
    true
}

pub fn verify_quorum(vset: &CompactValidatorSet, seal: &Seal) -> bool {
    // Note that invalid index would already be rejcted in verify_signature()
    let total_delegation: u64 = vset.iter().map(|validator| validator.delegation).sum();
    let voted_delegation: u64 = seal.0.iter().map(|(index, _)| vset[*index].delegation).sum();
    if total_delegation * 2 >= voted_delegation * 3 {
        return false
    }
    true
}

pub fn verify_header(client_state: &ClientState, proposal: &UpdateHeader) -> bool {
    if client_state.number + 1 != proposal.number {
        return false
    }
    if client_state.next_validator_set_hash != proposal.validator_set.hash() {
        return false
    }
    if !verify_signature(proposal.hash, &proposal.validator_set, &proposal.seal) {
        return false
    }
    if !verify_quorum(&proposal.validator_set, &proposal.seal) {
        return false
    }
    true
}

#[cfg(test)]
mod tests {
    /*
    Note that tests for verify_state() & verify_transaction() are not needed,
    because all they do is just calling verify() function which had been already covered
    in unit tests in rust-merkle-trie
    */
    use super::*;
    use ccrypto::blake256;
    use ckey::sign_schnorr;
    use ckey::{Generator, Private, Public, Random};
    use ctypes::CompactValidatorEntry;
    use rand::{rngs::StdRng, Rng};

    #[test]
    fn verify_quorum() {
        let iteration = 100;
        let seed = [0 as u8; 32];
        let mut rng: StdRng = rand::SeedableRng::from_seed(seed);

        for _ in 0..iteration {
            let n: usize = rng.gen_range(4, 255 + 1);
            let hash = blake256(format!("{}", rng.gen::<u64>()));

            let mut users: Vec<(Public, Private)> = Vec::new();
            let mut vset = CompactValidatorSet::new(Vec::new());
            let mut seal = Seal(Vec::new());

            let mut del_total = 0 as u64;
            let mut del_signed = 0 as u64;
            for _ in 0..n {
                users.push({
                    let x = Random.generate().unwrap();
                    (*x.public(), *x.private())
                });
                let del = rng.gen_range(1, 100);
                vset.push(CompactValidatorEntry {
                    public_key: users.last().unwrap().0,
                    delegation: del,
                });
                del_total += del;
            }

            for (i, user) in users.iter().enumerate() {
                if rng.gen_range(0, 3) == 0 {
                    continue
                } else {
                    seal.0.push((i as usize, sign_schnorr(&user.1, &hash).unwrap()));
                    del_signed += vset[i].delegation;
                }
            }

            let client_state = ClientState {
                number: 10,
                next_validator_set_hash: vset.hash(),
            };

            let proposal = UpdateHeader {
                number: 11,
                hash,
                seal,
                validator_set: vset,
            };

            let result = verify_header(&client_state, &proposal);

            assert_eq!(result, 2 * del_total < 3 * del_signed);
        }
    }
}
