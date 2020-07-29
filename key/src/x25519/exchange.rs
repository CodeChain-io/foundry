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

use crate::{Error, SharedSecret, X25519Private, X25519Public};
use sodiumoxide::crypto::scalarmult::scalarmult;

pub fn exchange(other_public: &X25519Public, my_private: &X25519Private) -> Result<SharedSecret, Error> {
    let X25519Private(scalar) = my_private;
    let X25519Public(group_element) = other_public;

    let shared_secret = scalarmult(scalar, group_element).map_err(|_| Error::InvalidSecret)?;
    Ok(SharedSecret::from_slice(shared_secret.as_ref()))
}

#[cfg(test)]
mod tests {
    use super::super::keypair::KeyPair;
    use super::exchange;
    use crate::{Generator, KeyPairTrait, Random};

    #[test]
    fn exchange_makes_same_private_key() {
        let k1: KeyPair = Random.generate().unwrap();
        let k2 = {
            let mut k2: KeyPair = Random.generate().unwrap();
            while k1 == k2 {
                k2 = Random.generate().unwrap();
            }
            k2
        };
        assert_ne!(k1, k2);

        let s1 = exchange(k2.public(), k1.private()).unwrap();
        let s2 = exchange(k1.public(), k2.private()).unwrap();
        assert_eq!(s1, s2);
    }
}
