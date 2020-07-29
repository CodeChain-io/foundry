// Copyright 2018, 2020 Kodebox, Inc.
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

mod checksum;
mod version;

use self::version::Version;
use crate::{Ed25519Public as Public, Error, NetworkId};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use serde::de::{Error as SerdeError, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash)]
pub struct PlatformAddress {
    /// The network id of the address.
    pub network_id: NetworkId,
    /// The version of the address.
    version: Version,
    /// Public key hash.
    pubkey: Public,
}

impl PlatformAddress {
    pub fn new_v0(network_id: NetworkId, pubkey: Public) -> Self {
        assert!(check_network_id(network_id));
        Self {
            network_id,
            version: Version::v0(),
            pubkey,
        }
    }

    pub fn pubkey(&self) -> &Public {
        self.try_pubkey().unwrap()
    }

    pub fn into_pubkey(self) -> Public {
        self.try_into_pubkey().unwrap()
    }

    pub fn try_pubkey(&self) -> Result<&Public, Error> {
        if !check_network_id(self.network_id) {
            return Err(Error::InvalidNetworkId(self.network_id))
        }
        Ok(&self.pubkey)
    }

    pub fn try_into_pubkey(self) -> Result<Public, Error> {
        if !check_network_id(self.network_id) {
            return Err(Error::InvalidNetworkId(self.network_id))
        }
        Ok(self.pubkey)
    }
}

impl fmt::Display for PlatformAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        assert!(check_network_id(self.network_id));
        let mut base64_encoded = base64::encode(self.pubkey);
        base64_encoded.pop(); // pad removed
        let base64url_encoded = base64_encoded.replace("+", &"-").replace("/", &"_");
        let checksum = checksum::calculate(&self.pubkey, self.network_id, self.version);
        write!(f, "{}{}{}{}", checksum, base64url_encoded, self.network_id, self.version)
    }
}

impl FromStr for PlatformAddress {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error>
    where
        Self: Sized, {
        let length = s.len();
        if length < 1 {
            return Err(Error::InvalidPlatformAddressFormat(s.to_string()))
        }

        let version = s[(length - 1)..].parse().map_err(|_| Error::InvalidPlatformAddressFormat(s.to_string()))?;

        if length < 8 + 43 + 2 + 1 {
            return Err(Error::InvalidPlatformAddressFormat(s.to_string()))
        }

        let network_id: NetworkId = s[(length - 2 - 1)..(length - 1)].parse().unwrap();
        if !check_network_id(network_id) {
            return Err(Error::InvalidNetworkId(network_id))
        }

        let base64url_encoded = s[8..(8 + 43)].to_string();
        let mut base64_encoded = base64url_encoded.replace("-", &"+").replace("_", &"/");
        base64_encoded.push('=');
        let decoded = base64::decode(&base64_encoded).map_err(|_| Error::InvalidPublic(s.to_string()))?;
        let pubkey = Public::from_slice(&decoded).ok_or_else(|| Error::InvalidPublic(s.to_string()))?;

        let calculated_checksum = checksum::calculate(&pubkey, network_id, version);
        let received_checksum = &s[0..8];
        if *received_checksum != calculated_checksum {
            return Err(Error::InvalidPlatformAddressFormat(s.to_string()))
        }

        Ok(Self {
            network_id,
            version,
            pubkey,
        })
    }
}

impl From<&'static str> for PlatformAddress {
    fn from(s: &'static str) -> Self {
        s.parse().unwrap_or_else(|_| panic!("invalid string literal for {}: '{}'", stringify!(Self), s))
    }
}

impl Serialize for PlatformAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer, {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

impl<'a> Deserialize<'a> for PlatformAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>, {
        deserializer.deserialize_any(PlatformAddressVisitor)
    }
}

struct PlatformAddressVisitor;

impl<'a> Visitor<'a> for PlatformAddressVisitor {
    type Value = PlatformAddress;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a base64 encoded string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: SerdeError, {
        PlatformAddress::from_str(value).map_err(|e| SerdeError::custom(format!("{}", e)))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: SerdeError, {
        PlatformAddress::from_str(value.as_ref()).map_err(|e| SerdeError::custom(format!("{}", e)))
    }
}

// FIXME: The below code can be simplified since Mutex::new is the const function.
//        Clean up this function when a const function becomes stable.
lazy_static! {
    static ref NETWORK_ID: Mutex<Option<NetworkId>> = Mutex::new(None);
}
fn check_network_id(network_id: NetworkId) -> bool {
    let mut saved_network_id = NETWORK_ID.lock();
    if saved_network_id.is_none() {
        *saved_network_id = Some(network_id);
    }
    *saved_network_id == Some(network_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialization() {
        let address = PlatformAddress::from_str("01sv1ngs16bSZoN8HFkTg7kNg1Boue1Y3TvOvW4oWRH1jkDOQTwtc0").unwrap();
        let serialized = serde_json::to_string(&address).unwrap();
        assert_eq!(serialized, r#""01sv1ngs16bSZoN8HFkTg7kNg1Boue1Y3TvOvW4oWRH1jkDOQTwtc0""#);
    }

    #[test]
    fn deserialization() {
        let addr1: Result<PlatformAddress, _> = serde_json::from_str(r#""""#);
        let addr2: Result<PlatformAddress, _> =
            serde_json::from_str(r#""4cnj73b1X2_K3XI_PJSXAQHZG0BD-VO4-TnhS3WEOy8Ym7UmT1Utc0""#);

        assert!(addr1.is_err());
        assert!(addr2.is_ok());
    }

    #[test]
    fn to_string() {
        let address = PlatformAddress {
            network_id: "tc".into(),
            version: Version::v0(),
            pubkey: Public::from_str("e83c0184ed9acc66868a7be2fbe901eecfe7c054450bbb8d24328e0116ea5e0c").unwrap(),
        };

        assert_eq!("whxw3vrh6DwBhO2azGaGinvi--kB7s_nwFRFC7uNJDKOARbqXgwtc0".to_string(), address.to_string());
    }
}
