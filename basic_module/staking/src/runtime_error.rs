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

use crate::error::Insufficient;
use crate::types::StakeQuantity;
use fkey::Ed25519Public as Public;
use std::fmt::{Display, Formatter, Result as FormatResult};

#[derive(Debug)]
pub enum Error {
    InsufficientStakes(Insufficient<StakeQuantity>),
    DelegateeNotFoundInCandidates(Public),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        match self {
            Error::InsufficientStakes(insufficient) => write!(f, "Insufficient stakes: {}", insufficient),
            Error::DelegateeNotFoundInCandidates(delegatee) => {
                write!(f, "Delegatee {:?} is not in Candidates", delegatee)
            }
        }
    }
}
