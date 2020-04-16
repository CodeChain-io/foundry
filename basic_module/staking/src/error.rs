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

use crate::runtime_error::Error as RuntimeError;
use std::fmt::{Display, Formatter, Result as FormatResult};

#[derive(Debug)]
/// Error indicating an expected value was not found.
pub struct Mismatch<T> {
    /// Value expected.
    pub expected: T,
    /// Value found.
    pub found: T,
}

impl<T: Display> Display for Mismatch<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        write!(f, "Expected {}, found {}", self.expected, self.found)
    }
}

#[derive(Debug)]
pub struct Insufficient<T> {
    /// Value to have at least
    pub required: T,
    /// Value found
    pub actual: T,
}

impl<T: Display> Display for Insufficient<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        write!(f, "Required at least {}, found {}", self.required, self.actual)
    }
}

#[derive(Debug)]
pub enum Error {
    Runtime(RuntimeError),
}

impl From<RuntimeError> for Error {
    fn from(error: RuntimeError) -> Error {
        Error::Runtime(error)
    }
}
