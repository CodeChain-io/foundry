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

use super::private::Private;
use super::public::Public;
use crate::KeyPairTrait;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
/// Ed25519 key pair
pub struct KeyPair {
    private: Private,
    public: Public,
}

impl fmt::Display for KeyPair {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        // Note: libsodium sign module does not support debug printing for secret values
        writeln!(f, "secret:  {:?}", self.private)?;
        writeln!(f, "public:  {:?}", self.public)?;
        Ok(())
    }
}

impl KeyPairTrait for KeyPair {
    type Private = Private;
    type Public = Public;

    /// Create a pair from secret key
    fn from_private(private: Private) -> Self {
        KeyPair {
            public: private.public_key(),
            private,
        }
    }

    fn from_keypair(private: Private, public: Public) -> Self {
        KeyPair {
            private,
            public,
        }
    }

    fn private(&self) -> &Private {
        &self.private
    }

    fn public(&self) -> &Public {
        &self.public
    }
}

impl KeyPair {
    pub fn get_private(self) -> Private {
        self.private
    }
}
