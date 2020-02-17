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

use crate::{Error, Message};
use bls_sigs_ref::{BLSSigCore as BlsSigCore, BLSSignatureBasic as BlsSignatureBasic};
use crypto::blake256;
use pairing_plus::bls12_381::{Fr, G1Compressed, G2Compressed, G1, G2};
use pairing_plus::{CurveAffine, CurveProjective, EncodedPoint};
use primitives::{H256, H384, H768};
use rand_core::OsRng;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};
use rustc_hex::{FromHex, ToHex};
use serde::de::Error as DeserializeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

pub const BLS_SIGNATURE_SIZE: usize = 48;
pub const BLS_PUBLIC_SIZE: usize = 96;

#[derive(Copy, Clone)]
pub struct BlsSignature(G1);

impl BlsSignature {
    pub fn random() -> Self {
        let mut rng = OsRng::default();
        BlsSignature(G1::random(&mut rng))
    }

    fn compressed(&self) -> G1Compressed {
        self.0.into_affine().into_compressed()
    }

    fn to_hex(&self) -> String {
        self.compressed().as_ref().to_hex()
    }

    fn from_bits(data: &H384) -> Result<Self, Error> {
        let g1 = match G1Compressed::from(data.0).into_affine() {
            Ok(g1_affine) => g1_affine.into_projective(),
            _ => return Err(Error::InvalidSignature),
        };
        Ok(BlsSignature(g1))
    }
}

impl PartialEq for BlsSignature {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for BlsSignature {}

impl fmt::Debug for BlsSignature {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "BlsSignature: {}", self.to_hex())
    }
}

impl fmt::Display for BlsSignature {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.to_hex())
    }
}

impl FromStr for BlsSignature {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.from_hex() {
            Ok(ref hex) if hex.len() == BLS_SIGNATURE_SIZE => {
                let mut data = [0; BLS_SIGNATURE_SIZE];
                data.copy_from_slice(&hex[0..BLS_SIGNATURE_SIZE]);
                let g1 = match G1Compressed::from(data).into_affine() {
                    Ok(g1) => g1.into_projective(),
                    _ => return Err(Error::InvalidSignature),
                };
                Ok(BlsSignature(g1))
            }
            _ => Err(Error::InvalidSignature),
        }
    }
}

impl Hash for BlsSignature {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let compressed = self.compressed();
        let data = compressed.as_ref();
        H384::from(data).hash(state);
    }
}

impl Default for BlsSignature {
    fn default() -> Self {
        BlsSignature(G1::zero())
    }
}
impl Encodable for BlsSignature {
    fn rlp_append(&self, s: &mut RlpStream) {
        let data: H384 = self.compressed().as_ref().into();
        data.rlp_append(s);
    }
}

impl Decodable for BlsSignature {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        let data = H384::decode(rlp)?;
        match BlsSignature::from_bits(&data) {
            Ok(signature) => Ok(signature),
            _ => Err(DecoderError::Custom("Invalid Signature")),
        }
    }
}

impl Serialize for BlsSignature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer, {
        let data: H384 = self.compressed().as_ref().into();
        data.serialize(serializer)
    }
}

impl<'a> Deserialize<'a> for BlsSignature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>, {
        let data = H384::deserialize(deserializer)?;
        match Self::from_bits(&data) {
            Ok(signature) => Ok(signature),
            _ => Err(D::Error::custom("Invalid signature")),
        }
    }
}

#[derive(Copy, Clone)]
pub struct BlsPublic(G2);

impl BlsPublic {
    pub fn random() -> Self {
        let mut rng = OsRng::default();
        BlsPublic(G2::random(&mut rng))
    }

    // Need to sign on BLSPublic for proof of posession
    pub fn hash_with_value<B: AsRef<[u8]>>(&self, value: B) -> Message {
        let mut data = self.compressed().as_ref().to_vec();
        data.extend(value.as_ref());
        blake256(data)
    }

    fn compressed(&self) -> G2Compressed {
        self.0.into_affine().into_compressed()
    }

    fn to_hex(&self) -> String {
        self.compressed().as_ref().to_hex()
    }

    fn from_bits(data: &H768) -> Result<Self, Error> {
        let g1 = match G2Compressed::from(data.0).into_affine() {
            Ok(g2_affine) => g2_affine.into_projective(),
            _ => return Err(Error::InvalidSignature),
        };
        Ok(BlsPublic(g1))
    }
}

impl PartialEq for BlsPublic {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for BlsPublic {}

impl Hash for BlsPublic {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.compressed().as_ref().hash(state);
    }
}

impl Ord for BlsPublic {
    fn cmp(&self, other: &Self) -> Ordering {
        self.compressed().as_ref().cmp(&other.compressed().as_ref())
    }
}

impl PartialOrd for BlsPublic {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Encodable for BlsPublic {
    fn rlp_append(&self, s: &mut RlpStream) {
        let data: H768 = self.compressed().as_ref().into();
        data.rlp_append(s);
    }
}

impl Decodable for BlsPublic {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        let data = H768::decode(rlp)?;
        match BlsPublic::from_bits(&data) {
            Ok(public) => Ok(public),
            _ => Err(DecoderError::Custom("Invalid Public")),
        }
    }
}

impl Serialize for BlsPublic {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer, {
        let data: H768 = self.compressed().as_ref().into();
        data.serialize(serializer)
    }
}

impl<'a> Deserialize<'a> for BlsPublic {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>, {
        let data = H768::deserialize(deserializer)?;
        match Self::from_bits(&data) {
            Ok(public) => Ok(public),
            _ => Err(D::Error::custom("Invalid Pubic")),
        }
    }
}

impl fmt::Debug for BlsPublic {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "BlsPublic: {}", self.to_hex())
    }
}

impl fmt::Display for BlsPublic {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.to_hex())
    }
}

impl FromStr for BlsPublic {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.from_hex() {
            Ok(ref hex) if hex.len() == BLS_PUBLIC_SIZE => {
                let mut data = [0; BLS_PUBLIC_SIZE];
                data.copy_from_slice(&hex[0..BLS_PUBLIC_SIZE]);
                let g2 = match G2Compressed::from(data).into_affine() {
                    Ok(g2) => g2.into_projective(),
                    _ => return Err(Error::InvalidPublic),
                };
                Ok(BlsPublic(g2))
            }
            _ => Err(Error::InvalidPublic),
        }
    }
}

#[derive(Copy, Clone)]
pub struct BlsPrivate(Fr);

pub struct BlsKeyPair {
    private: BlsPrivate,
    public: BlsPublic,
}

impl BlsKeyPair {
    pub fn from_secret<B: AsRef<[u8]>>(secret: B) -> Self {
        let (x_prime, pk) = G1::keygen(secret);
        BlsKeyPair {
            private: BlsPrivate(x_prime),
            public: BlsPublic(pk),
        }
    }

    pub fn private(&self) -> &BlsPrivate {
        &self.private
    }

    pub fn public(&self) -> &BlsPublic {
        &self.public
    }
}

impl From<H256> for BlsPrivate {
    fn from(msg: H256) -> Self {
        let (private, _public) = <G1 as BlsSigCore>::keygen(msg);
        BlsPrivate(private)
    }
}

pub fn sign_bls(private: &BlsPrivate, message: &Message) -> BlsSignature {
    let signature = <G1 as BlsSignatureBasic>::sign(private.0, message);
    BlsSignature(signature)
}

pub fn aggregate_signatures_bls(signatures: &[BlsSignature]) -> BlsSignature {
    let signatures_g1: Vec<_> = signatures.iter().map(|sig| sig.0).collect();
    let aggregated_signatures_g1 = <G1 as BlsSigCore>::aggregate(&signatures_g1);
    BlsSignature(aggregated_signatures_g1)
}

pub fn verify_aggregated_bls(
    publics: &[BlsPublic],
    aggregated_signature: &BlsSignature,
    message: &Message,
) -> Result<bool, Error> {
    let aggregated_public = aggregate_publics_bls(publics)?;
    verify_bls(&aggregated_public, &aggregated_signature, message)
}

fn aggregate_publics_bls(publics: &[BlsPublic]) -> Result<BlsPublic, Error> {
    let publics_g2: Vec<_> = publics.iter().map(|public| public.0).collect();
    let aggregated_publics_g2 = <G2 as BlsSigCore>::aggregate(&publics_g2);
    Ok(BlsPublic(aggregated_publics_g2))
}

pub fn verify_bls(public: &BlsPublic, signature: &BlsSignature, message: &Message) -> Result<bool, Error> {
    Ok(BlsSignatureBasic::verify(public.0, signature.0, message))
}
