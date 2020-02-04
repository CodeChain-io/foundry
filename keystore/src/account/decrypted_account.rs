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

use ckey::{sign, Ed25519Private as Private, Ed25519Public as Public, Error as KeyError, Message, Secret, Signature};

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
        match Private::from_slice(&self.secret) {
            Some(private) => Ok(sign(&message, &private)),
            None => Err(KeyError::InvalidSecret),
        }
    }

    /// Derive public key.
    pub fn public(&self) -> Result<Public, KeyError> {
        match Private::from_slice(&self.secret) {
            Some(private) => Ok(private.public_key()),
            None => Err(KeyError::InvalidSecret),
        }
    }
}

impl Drop for DecryptedAccount {
    fn drop(&mut self) {
        self.secret = Secret::default();
    }
}
