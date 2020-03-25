// Copyright 2018-2020 Kodebox, Inc.
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

use crate::error::Error;
use ccrypto::blake256;
use ckey::{sign, verify, Ed25519Private as Private, Ed25519Public as Public, Error as KeyError, Signature};
use ctypes::errors::SyntaxError;
use ctypes::transaction::Transaction;
use ctypes::{BlockHash, BlockNumber, CommonParams, TxHash};
use rlp::{DecoderError, Encodable, Rlp, RlpStream};
use std::convert::{TryFrom, TryInto};

/// Signed transaction information without verified signature.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SignedTransaction {
    /// Plain Transaction.
    unsigned: Transaction,
    /// Signature.
    sig: Signature,
    /// Signer's public key
    signer_public: Public,
    /// Hash of the transaction
    hash: TxHash,
}

impl rlp::Encodable for SignedTransaction {
    fn rlp_append(&self, s: &mut RlpStream) {
        self.rlp_append_sealed_transaction(s)
    }
}

impl rlp::Decodable for SignedTransaction {
    fn decode(d: &Rlp<'_>) -> Result<Self, DecoderError> {
        let item_count = d.item_count()?;
        if item_count != 6 {
            return Err(DecoderError::RlpIncorrectListLen {
                expected: 6,
                got: item_count,
            })
        }
        let hash = blake256(d.as_raw()).into();
        Ok(SignedTransaction {
            unsigned: Transaction {
                seq: d.val_at(0)?,
                fee: d.val_at(1)?,
                network_id: d.val_at(2)?,
                action: d.val_at(3)?,
            },
            sig: d.val_at(4)?,
            signer_public: d.val_at(5)?,
            hash,
        })
    }
}

impl SignedTransaction {
    /// Append object with a signature into RLP stream
    fn rlp_append_sealed_transaction(&self, s: &mut RlpStream) {
        s.begin_list(6);
        s.append(&self.unsigned.seq);
        s.append(&self.unsigned.fee);
        s.append(&self.unsigned.network_id);
        s.append(&self.unsigned.action);
        s.append(&self.sig);
        s.append(&self.signer_public);
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VerifiedTransaction(SignedTransaction);

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UnverifiedTransaction(SignedTransaction);

impl rlp::Encodable for VerifiedTransaction {
    fn rlp_append(&self, s: &mut RlpStream) {
        self.0.rlp_append(s);
    }
}

impl From<VerifiedTransaction> for UnverifiedTransaction {
    fn from(tx: VerifiedTransaction) -> Self {
        UnverifiedTransaction(tx.0)
    }
}

impl VerifiedTransaction {
    /// Signs the transaction as coming from `signer`.
    pub fn new_with_sign(tx: Transaction, private: &Private) -> VerifiedTransaction {
        let sig = sign(&tx.hash(), private);
        UnverifiedTransaction::new(tx, sig, private.public_key())
            .try_into()
            .expect("The transaction's signature is invalid")
    }

    /// Returns a public key of the signer.
    pub fn signer_public(&self) -> Public {
        self.0.signer_public
    }

    /// Returns a public key of the signer.
    pub fn signature(&self) -> Signature {
        self.0.sig
    }

    pub fn transaction(&self) -> &Transaction {
        &self.0.unsigned
    }

    pub fn hash(&self) -> TxHash {
        self.0.hash
    }
}

impl rlp::Encodable for UnverifiedTransaction {
    fn rlp_append(&self, s: &mut RlpStream) {
        self.0.rlp_append(s);
    }
}

impl rlp::Decodable for UnverifiedTransaction {
    fn decode(d: &Rlp<'_>) -> Result<Self, DecoderError> {
        let transaction = d.as_val()?;
        Ok(Self(transaction))
    }
}

impl TryFrom<UnverifiedTransaction> for VerifiedTransaction {
    type Error = Error;
    fn try_from(tx: UnverifiedTransaction) -> Result<Self, Error> {
        if tx.verify_transaction() {
            Ok(VerifiedTransaction(tx.0))
        } else {
            Err(Error::Key(KeyError::InvalidSignature))
        }
    }
}

impl UnverifiedTransaction {
    pub fn new(unsigned: Transaction, sig: Signature, signer_public: Public) -> Self {
        UnverifiedTransaction(SignedTransaction {
            unsigned,
            sig,
            signer_public,
            hash: Default::default(),
        })
        .compute_hash()
    }

    /// Used to compute hash of created transactions
    fn compute_hash(mut self) -> UnverifiedTransaction {
        let hash = blake256(&*self.rlp_bytes()).into();
        self.0.hash = hash;
        self
    }

    /// Construct a signature object from the sig.
    pub fn signature(&self) -> Signature {
        self.0.sig
    }

    /// Get the hash of this header (blake256 of the RLP).
    pub fn hash(&self) -> TxHash {
        self.0.hash
    }

    pub fn signer_public(&self) -> Public {
        self.0.signer_public
    }

    /// Verify basic signature params. Does not attempt signer recovery.
    pub fn verify_basic(&self) -> Result<(), SyntaxError> {
        self.transaction().action.verify()
    }

    /// Verify transactiosn with the common params. Does not attempt signer recovery.
    pub fn verify_with_params(&self, params: &CommonParams) -> Result<(), SyntaxError> {
        if self.transaction().network_id != params.network_id() {
            return Err(SyntaxError::InvalidNetworkId(self.transaction().network_id))
        }
        let byte_size = rlp::encode(self).to_vec().len();
        if byte_size >= params.max_body_size() {
            return Err(SyntaxError::TransactionIsTooBig)
        }
        self.transaction().action.verify_with_params(params)
    }

    pub fn verify_transaction(&self) -> bool {
        verify(&self.0.sig, &self.0.unsigned.hash(), &self.0.signer_public)
    }

    pub fn transaction(&self) -> &Transaction {
        &self.0.unsigned
    }
}

pub struct PendingVerifiedTransactions {
    pub transactions: Vec<VerifiedTransaction>,
    pub last_timestamp: Option<u64>,
}

/// Signed Transaction that is a part of canon blockchain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalizedTransaction {
    /// Signed part.
    pub signed: UnverifiedTransaction,
    /// Block number.
    pub block_number: BlockNumber,
    /// Block hash.
    pub block_hash: BlockHash,
    /// Transaction index within block.
    pub transaction_index: usize,
    /// Cached public
    pub cached_signer_public: Option<Public>,
}

impl LocalizedTransaction {
    /// Returns transaction signer.
    /// Panics if `LocalizedTransaction` is constructed using invalid `UnverifiedTransaction`.
    pub fn signer(&mut self) -> Public {
        if let Some(public) = self.cached_signer_public {
            return public
        }
        let public = self.signed.signer_public();
        self.cached_signer_public = Some(public);
        public
    }

    pub fn unverified_tx(&self) -> &UnverifiedTransaction {
        &self.signed
    }
}

impl From<LocalizedTransaction> for Transaction {
    fn from(tx: LocalizedTransaction) -> Self {
        tx.signed.0.unsigned
    }
}

#[cfg(test)]
mod tests {
    use ckey::{Address, Signature};
    use ctypes::transaction::Action;
    use primitives::H256;
    use rlp::rlp_encode_and_decode_test;

    use super::*;

    #[test]
    fn encode_and_decode_pay_transaction() {
        rlp_encode_and_decode_test!(UnverifiedTransaction(SignedTransaction {
            unsigned: Transaction {
                seq: 30,
                fee: 40,
                network_id: "tc".into(),
                action: Action::Pay {
                    receiver: Address::random(),
                    quantity: 300,
                },
            },
            sig: Signature::default(),
            hash: H256::default().into(),
            signer_public: Public::random(),
        })
        .compute_hash());
    }
}
