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

use crate::account_provider::Error as AccountProviderError;
use crate::consensus::EngineError;
use cdb::DatabaseError;
use cio::IoError;
use ckey::{Ed25519Public as Public, Error as KeyError};
use coordinator::types::CloseBlockError;
use cstate::StateError;
use ctypes::errors::{HistoryError, RuntimeError, SyntaxError};
use ctypes::util::unexpected::{Mismatch, OutOfBounds};
use ctypes::{BlockHash, BlockNumber};
use merkle_trie::TrieError;
use primitives::H256;
use rlp::DecoderError;
use std::fmt;
use std::io::Error as StdIoError;

#[derive(Debug, Clone, Copy, PartialEq)]
/// Import to the block queue result
pub enum ImportError {
    /// Already in the block chain.
    AlreadyInChain,
    /// Already in the block queue.
    AlreadyQueued,
    /// Already marked as bad from a previous import (could mean parent is bad).
    KnownBad,
}

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            ImportError::AlreadyInChain => "block already in chain",
            ImportError::AlreadyQueued => "block already in the block queue",
            ImportError::KnownBad => "block known to be bad",
        };

        f.write_fmt(format_args!("Block import error ({})", msg))
    }
}

/// Error dedicated to import block function
#[derive(Debug)]
pub enum BlockImportError {
    /// Import error
    Import(ImportError),
    /// Block error
    Block(BlockError),
    /// Other error
    Other(String),
}

impl From<Error> for BlockImportError {
    fn from(e: Error) -> Self {
        match e {
            Error::Block(block_error) => BlockImportError::Block(block_error),
            Error::Import(import_error) => BlockImportError::Import(import_error),
            _ => BlockImportError::Other(format!("other block import error: {:?}", e)),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
/// Errors concerning block processing.
pub enum BlockError {
    /// Extra data is of an invalid length.
    ExtraDataOutOfBounds(OutOfBounds<usize>),
    /// Seal is incorrect format.
    InvalidSealArity(Mismatch<usize>),
    /// State root header field is invalid.
    InvalidStateRoot(Mismatch<H256>),
    /// Transactions root header field is invalid.
    InvalidTransactionsRoot(Mismatch<H256>),
    /// Next validator set hash header field is invalid.
    InvalidNextValidatorSetHash(Mismatch<H256>),
    /// Some low-level aspect of the seal is incorrect.
    InvalidSeal,
    /// Timestamp header field is invalid.
    InvalidTimestamp(OutOfBounds<u64>),
    /// Timestamp header field is too far in future.
    TemporarilyInvalid(OutOfBounds<u64>),
    /// Parent hash field of header is invalid; this is an invalid error indicating a logic flaw in the codebase.
    /// TODO: remove and favour an assert!/panic!.
    InvalidParentHash(Mismatch<BlockHash>),
    /// Number field of header is invalid.
    InvalidNumber(Mismatch<BlockNumber>),
    /// Block number isn't sensible.
    RidiculousNumber(OutOfBounds<BlockNumber>),
    /// Too many transactions from a particular address.
    TooManyTransactions(Public),
    /// Parent given is unknown.
    UnknownParent(BlockHash),
    /// Body size limit is exceeded.
    BodySizeIsTooBig,
    /// prev_validator_set field in SyncHeader struct is invalid.
    InvalidValidatorSet,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SchemeError {
    InvalidState,
}

impl fmt::Display for SchemeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::SchemeError::*;
        let msg: String = match self {
            InvalidState => "Genesis state is not same with spec".into(),
        };
        f.write_fmt(format_args!("Scheme file error ({})", msg))
    }
}

impl fmt::Display for BlockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::BlockError::*;

        let msg: String = match self {
            ExtraDataOutOfBounds(oob) => format!("Extra block data too long. {}", oob),
            InvalidSealArity(mis) => format!("Block seal in incorrect format: {}", mis),
            InvalidStateRoot(mis) => format!("Invalid state root in header: {}", mis),
            InvalidTransactionsRoot(mis) => format!("Invalid transactions root in header: {}", mis),
            InvalidNextValidatorSetHash(mis) => format!("Invalid next validator set hash in header: {}", mis),
            InvalidSeal => "Block has invalid seal.".into(),
            InvalidTimestamp(oob) => format!("Invalid timestamp in header: {}", oob),
            TemporarilyInvalid(oob) => format!("Future timestamp in header: {}", oob),
            InvalidParentHash(mis) => format!("Invalid parent hash: {}", mis),
            InvalidNumber(mis) => format!("Invalid number in header: {}", mis),
            RidiculousNumber(oob) => format!("Implausible block number. {}", oob),
            UnknownParent(hash) => format!("Unknown parent: {}", hash),
            TooManyTransactions(pubkey) => format!("Too many transactions from: {:?}", pubkey),
            BodySizeIsTooBig => "Block's body size is too big".to_string(),
            InvalidValidatorSet => "Invalid prev_validator_set in SyncHeader".to_string(),
        };

        f.write_fmt(format_args!("Block error ({})", msg))
    }
}

#[derive(Debug)]
/// General error type which should be capable of representing all errors in codechain
pub enum Error {
    /// Error concerning block processing.
    Block(BlockError),
    /// Error concerning block import.
    Import(ImportError),
    /// Io crate error.
    Io(IoError),
    /// Consensus vote error.
    Engine(EngineError),
    /// Key error.
    Key(KeyError),
    Scheme(SchemeError),
    /// Account Provider error.
    AccountProvider(AccountProviderError),
    Trie(TrieError),
    Runtime(RuntimeError),
    History(HistoryError),
    Syntax(SyntaxError),
    /// Error concerning a database.
    Database(DatabaseError),
    Rlp(DecoderError),
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => err.fmt(f),
            Error::Block(err) => err.fmt(f),
            Error::Import(err) => err.fmt(f),
            Error::Engine(err) => err.fmt(f),
            Error::Key(err) => err.fmt(f),
            Error::Scheme(err) => err.fmt(f),
            Error::AccountProvider(err) => err.fmt(f),
            Error::Trie(err) => err.fmt(f),
            Error::Runtime(err) => err.fmt(f),
            Error::History(err) => err.fmt(f),
            Error::Syntax(err) => err.fmt(f),
            Error::Database(err) => err.fmt(f),
            Error::Rlp(err) => err.fmt(f),
            Error::Other(s) => write!(f, "{}", s),
        }
    }
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Error {
        Error::Io(err)
    }
}

impl From<StdIoError> for Error {
    fn from(err: StdIoError) -> Self {
        Self::Io(err.into())
    }
}

impl From<BlockError> for Error {
    fn from(err: BlockError) -> Error {
        Error::Block(err)
    }
}

impl From<SchemeError> for Error {
    fn from(err: SchemeError) -> Error {
        Error::Scheme(err)
    }
}

impl From<EngineError> for Error {
    fn from(err: EngineError) -> Error {
        Error::Engine(err)
    }
}

impl From<KeyError> for Error {
    fn from(err: KeyError) -> Error {
        Error::Key(err)
    }
}

impl From<ImportError> for Error {
    fn from(err: ImportError) -> Error {
        Error::Import(err)
    }
}

impl From<BlockImportError> for Error {
    fn from(err: BlockImportError) -> Error {
        match err {
            BlockImportError::Block(e) => Error::Block(e),
            BlockImportError::Import(e) => Error::Import(e),
            BlockImportError::Other(s) => Error::Other(s),
        }
    }
}

impl From<AccountProviderError> for Error {
    fn from(err: AccountProviderError) -> Error {
        Error::AccountProvider(err)
    }
}

impl From<TrieError> for Error {
    fn from(err: TrieError) -> Self {
        Error::Trie(err)
    }
}

impl From<StateError> for Error {
    fn from(err: StateError) -> Self {
        match err {
            StateError::Trie(err) => Error::Trie(err),
            StateError::Runtime(err) => Error::Runtime(err),
        }
    }
}

impl From<RuntimeError> for Error {
    fn from(err: RuntimeError) -> Self {
        Error::Runtime(err)
    }
}

impl From<HistoryError> for Error {
    fn from(err: HistoryError) -> Self {
        Error::History(err)
    }
}

impl From<SyntaxError> for Error {
    fn from(err: SyntaxError) -> Self {
        Error::Syntax(err)
    }
}

impl From<DatabaseError> for Error {
    fn from(err: DatabaseError) -> Error {
        Error::Database(err)
    }
}

impl From<DecoderError> for Error {
    fn from(err: DecoderError) -> Error {
        Error::Rlp(err)
    }
}

impl From<CloseBlockError> for Error {
    fn from(err: CloseBlockError) -> Error {
        Error::Other(err)
    }
}
