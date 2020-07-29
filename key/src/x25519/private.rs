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

use super::public::Public;
use sodiumoxide::crypto::scalarmult::{scalarmult_base, Scalar};

#[derive(Debug, Clone, PartialEq)]
// The inner type Scalar clears its memory when it is dropped
pub struct Private(pub(crate) Scalar);

impl Private {
    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        Scalar::from_slice(slice).map(Self)
    }

    pub fn public_key(&self) -> Public {
        let Private(scalar) = self;
        scalarmult_base(scalar).into()
    }
}

impl AsRef<[u8]> for Private {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<Scalar> for Private {
    fn from(k: Scalar) -> Self {
        Private(k)
    }
}
