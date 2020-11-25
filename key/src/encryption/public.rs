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

use sodiumoxide::crypto::box_::PublicKey;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Public(pub(crate) PublicKey);

impl Public {
    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        PublicKey::from_slice(slice).map(Self)
    }
}

impl AsRef<[u8]> for Public {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<PublicKey> for Public {
    fn from(k: PublicKey) -> Self {
        Public(k)
    }
}
