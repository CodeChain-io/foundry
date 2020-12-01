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

mod keypair;
mod private;
mod public;

pub use keypair::KeyPair;
pub use private::Private;
pub use public::Public;
use sodiumoxide::crypto::sealedbox;

pub fn encrypt(message: &[u8], public: &Public) -> Vec<u8> {
    let Public(public) = public;
    sealedbox::seal(message, public)
}

pub fn decrypt(encrypted: &[u8], public: &Public, private: &Private) -> Result<Vec<u8>, ()> {
    let Private(private) = private;
    let Public(public) = public;
    sealedbox::open(encrypted, public, private)
}

#[cfg(test)]
mod tests {
    use super::keypair::KeyPair;
    use crate::{decrypt, encrypt, Generator, KeyPairTrait, Random};
    #[test]
    fn open_and_seal() {
        let secret_data = b"Dr. Crowe was dead";
        let keypair: KeyPair = Random.generate().unwrap();
        let encrypted = encrypt(secret_data, keypair.public());
        let decrypted = decrypt(&encrypted, keypair.public(), keypair.private()).unwrap();
        assert_eq!(secret_data[..], decrypted[..]);
    }

    #[test]
    fn same_data_different_result() {
        let secret_data = b"Dr. Crowe was dead";
        let keypair: KeyPair = Random.generate().unwrap();
        let encrypted1 = encrypt(secret_data, keypair.public());
        let decrypted1 = decrypt(&encrypted1, keypair.public(), keypair.private()).unwrap();
        let encrypted2 = encrypt(secret_data, keypair.public());
        let decrypted2 = decrypt(&encrypted2, keypair.public(), keypair.private()).unwrap();
        assert_eq!(secret_data[..], decrypted1[..]);
        assert_eq!(secret_data[..], decrypted2[..]);
        assert_ne!(encrypted1[..], encrypted2[..]);
    }
}
