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

use ckey::{Ed25519Public as Public, Signature};

#[derive(Clone, Debug, Eq, PartialEq, RlpEncodable, RlpDecodable, Serialize, Deserialize)]
pub struct Approval {
    signature: Signature,
    signer_public: Public,
}

impl Approval {
    pub fn new(signature: Signature, signer_public: Public) -> Self {
        Self {
            signature,
            signer_public,
        }
    }
    pub fn signature(&self) -> &Signature {
        &self.signature
    }

    pub fn signer_public(&self) -> &Public {
        &self.signer_public
    }
}
