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

use super::private::Private;
use super::public::Public;
use crate::KeyPairTrait;

#[derive(Debug, Clone, PartialEq)]
pub struct KeyPair {
    private: Private,
    public: Public,
}

impl KeyPairTrait for KeyPair {
    type Private = Private;
    type Public = Public;

    fn from_private(private: Self::Private) -> Self {
        KeyPair {
            public: private.public_key(),
            private,
        }
    }

    fn from_keypair(private: Self::Private, public: Self::Public) -> Self {
        KeyPair {
            private,
            public,
        }
    }

    fn private(&self) -> &Self::Private {
        &self.private
    }

    fn public(&self) -> &Self::Public {
        &self.public
    }
}
