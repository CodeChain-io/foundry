// Copyright 2019. Kodebox, Inc.
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

use ckey::{
    sign, sign_bls, BlsKeyPair, BlsPrivate, BlsPublic, BlsSignature, Error as KeyError, KeyPair, Message, Private,
    Public, Secret, Signature,
};

/// An opaque wrapper for secret.
#[derive(Clone)]
pub struct DecryptedAccount {
    secret: Secret,
}

impl DecryptedAccount {
    pub fn new(secret: Secret) -> DecryptedAccount {
        DecryptedAccount {
            secret,
        }
    }

    /// Sign a message.
    pub fn sign(&self, message: &Message) -> Result<Signature, KeyError> {
        sign(&Private::from(self.secret), message)
    }

    /// Sign a message with Schnorr scheme.
    pub fn sign_bls(&self, message: &Message) -> BlsSignature {
        sign_bls(&BlsPrivate::from(self.secret), message)
    }

    /// Derive public key.
    pub fn public(&self) -> Result<Public, KeyError> {
        Ok(*KeyPair::from_private(Private::from(self.secret))?.public())
    }

    /// Derive BLS public key for block signing.
    pub fn bls_public(&self) -> BlsPublic {
        *BlsKeyPair::from_secret(self.secret).public()
    }

    /// Signature of BLS public key signed by oneself.
    /// This is for proof of posession.
    pub fn pop_signature<B: AsRef<[u8]>>(&self, to_concat: B) -> BlsSignature {
        let public = self.bls_public();
        self.sign_bls(&public.hash_with_value(to_concat))
    }
}

impl Drop for DecryptedAccount {
    fn drop(&mut self) {
        self.secret = Secret::default();
    }
}
