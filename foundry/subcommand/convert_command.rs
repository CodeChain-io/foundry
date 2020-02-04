// Copyright 2019-2020 Kodebox, Inc.
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

use crate::config::ChainType;
use ckey::hex::ToHex;
use ckey::{
    public_to_address, Address, Ed25519KeyPair as KeyPair, Ed25519Private as Private, Ed25519Public as Public,
    NetworkId, PlatformAddress,
};
use clap::ArgMatches;
use primitives::remove_0x_prefix;
use std::io::stdin;
use std::ops::Deref;
use std::str::FromStr;

pub fn run_convert_command(matches: &ArgMatches<'_>) -> Result<(), String> {
    let from = matches.value_of("from").expect("Argument 'from' is required");
    let to = matches.value_of("to").expect("Argument 'to' is required");

    let mut input = String::new();
    stdin().read_line(&mut input).map_err(|e| e.to_string())?;
    let result = convert(from, to, &input.trim(), || get_network_id(matches))?;
    println!("{}", result);
    Ok(())
}

fn convert(
    from: &str,
    to: &str,
    input: &str,
    get_network_id: impl FnOnce() -> Result<NetworkId, String>,
) -> Result<String, String> {
    match (from, to) {
        ("private", "private") => {
            let private = get_private(input)?;
            Ok(private.as_ref().to_hex())
        }
        ("private", "public") => {
            let private = get_private(input)?;
            let public = private_to_public(private)?;
            Ok(public.as_ref().to_hex())
        }
        ("private", "address") => {
            let private = get_private(input)?;
            let public = private_to_public(private)?;
            let address = public_to_address(&public);
            Ok(format!("{:x}", address.deref()))
        }
        ("private", "accountId") => {
            let network_id = get_network_id()?;

            let private = get_private(input)?;
            let public = private_to_public(private)?;
            let address = public_to_address(&public);
            let account_id = PlatformAddress::new_v1(network_id, address);
            Ok(account_id.to_string())
        }
        ("public", "public") => {
            let public = get_public(input)?;
            Ok(public.as_ref().to_hex())
        }
        ("public", "address") => {
            let public = get_public(input)?;
            let address = public_to_address(&public);
            Ok(format!("{:x}", address.deref()))
        }
        ("public", "accountId") => {
            let network_id = get_network_id()?;

            let public = get_public(input)?;
            let address = public_to_address(&public);
            let account_id = PlatformAddress::new_v1(network_id, address);
            Ok(account_id.to_string())
        }
        ("address", "address") => {
            let address = get_address(input)?;
            Ok(format!("{:x}", address.deref()))
        }
        ("address", "accountId") => {
            let network_id = get_network_id()?;

            let address = get_address(input)?;
            let account_id = PlatformAddress::new_v1(network_id, address);
            Ok(account_id.to_string())
        }
        ("accountId", "accountId") => {
            let account_id = get_account_id(input)?;
            Ok(account_id.to_string())
        }
        ("accountId", "address") => {
            let account_id = get_account_id(input)?;
            let address = account_id.into_address();
            Ok(format!("{:x}", address.deref()))
        }
        (..) => Err(format!("Cannot convert from {} to {}", from, to)),
    }
}

fn get_public(input: &str) -> Result<Public, String> {
    Public::from_str(remove_0x_prefix(input)).map_err(|e| format!("Error on reading public key: {}", e))
}

fn get_private(input: &str) -> Result<Private, String> {
    Private::from_str(remove_0x_prefix(input)).map_err(|e| format!("Error on reading private key: {}", e))
}

fn get_address(input: &str) -> Result<Address, String> {
    Address::from_str(input).map_err(|e| format!("Error on reading address: {}", e))
}

fn get_account_id(input: &str) -> Result<PlatformAddress, String> {
    PlatformAddress::from_str(input).map_err(|e| format!("Error on reading accountId: {}", e))
}

fn private_to_public(private: Private) -> Result<Public, String> {
    let keypair =
        KeyPair::from_private(private).map_err(|e| format!("Error on converting private key to public key: {}", e))?;
    Ok(*keypair.public())
}

fn get_network_id(matches: &ArgMatches<'_>) -> Result<NetworkId, String> {
    let chain = matches.value_of("chain").unwrap_or_else(|| "solo");
    let chain_type: ChainType = chain.parse().unwrap();
    // XXX: What should we do if the network id has been changed
    let network_id: NetworkId = chain_type.scheme().map(|scheme| scheme.genesis_params().network_id())?;
    Ok(network_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PRIVATE_KEY: &str = "c5240ff5244a3bcd705998600cd40b1b6033c44337294da04c8a7545e1b87fedc8e8c897db2cb53dae2ed6114542057c1a164603f41ee6036d273303c9bad650";
    const PUBLIC_KEY: &str = "c8e8c897db2cb53dae2ed6114542057c1a164603f41ee6036d273303c9bad650";
    const ADDRESS: &str = "a6f4cb03c82244c6678de539e4a66c35a2e7ab78";
    const ACCOUNT_ID: &str = "tccqxn0fjcreq3yf3n83hjnne9xds669eat0qhh2fwv";

    fn get_test_network_id() -> Result<NetworkId, String> {
        Ok(NetworkId::from("tc"))
    }

    #[test]
    fn test_private_to_private() {
        let result = convert("private", "private", PRIVATE_KEY, get_test_network_id);
        assert_eq!(result, Ok(PRIVATE_KEY.to_string()));

        let prefixed = format!("0x{}", PRIVATE_KEY);
        let result = convert("private", "private", &prefixed, get_test_network_id);
        assert_eq!(result, Ok(PRIVATE_KEY.to_string()));
    }

    #[test]
    fn test_private_to_public() {
        let result = convert("private", "public", PRIVATE_KEY, get_test_network_id);
        assert_eq!(result, Ok(PUBLIC_KEY.to_string()));

        let prefixed = format!("0x{}", PRIVATE_KEY);
        let result = convert("private", "public", &prefixed, get_test_network_id);
        assert_eq!(result, Ok(PUBLIC_KEY.to_string()));
    }

    #[test]
    fn test_private_to_address() {
        let result = convert("private", "address", PRIVATE_KEY, get_test_network_id);
        assert_eq!(result, Ok(ADDRESS.to_string()));

        let prefixed = format!("0x{}", PRIVATE_KEY);
        let result = convert("private", "address", &prefixed, get_test_network_id);
        assert_eq!(result, Ok(ADDRESS.to_string()));
    }

    #[test]
    fn test_private_to_account_id() {
        let result = convert("private", "accountId", PRIVATE_KEY, get_test_network_id);
        assert_eq!(result, Ok(ACCOUNT_ID.to_string()));

        let prefixed = format!("0x{}", PRIVATE_KEY);
        let result = convert("private", "accountId", &prefixed, get_test_network_id);
        assert_eq!(result, Ok(ACCOUNT_ID.to_string()));
    }

    #[test]
    fn test_public_to_public() {
        let result = convert("public", "public", PUBLIC_KEY, get_test_network_id);
        assert_eq!(result, Ok(PUBLIC_KEY.to_string()));

        let prefixed = format!("0x{}", PUBLIC_KEY);
        let result = convert("public", "public", &prefixed, get_test_network_id);
        assert_eq!(result, Ok(PUBLIC_KEY.to_string()));
    }

    #[test]
    fn test_public_to_address() {
        let result = convert("public", "address", PUBLIC_KEY, get_test_network_id);
        assert_eq!(result, Ok(ADDRESS.to_string()));

        let prefixed = format!("0x{}", PUBLIC_KEY);
        let result = convert("public", "address", &prefixed, get_test_network_id);
        assert_eq!(result, Ok(ADDRESS.to_string()));
    }

    #[test]
    fn test_public_to_account_id() {
        let result = convert("public", "accountId", PUBLIC_KEY, get_test_network_id);
        assert_eq!(result, Ok(ACCOUNT_ID.to_string()));

        let prefixed = format!("0x{}", PUBLIC_KEY);
        let result = convert("public", "accountId", &prefixed, get_test_network_id);
        assert_eq!(result, Ok(ACCOUNT_ID.to_string()));
    }

    #[test]
    fn test_address_to_address() {
        let result = convert("address", "address", ADDRESS, get_test_network_id);
        assert_eq!(result, Ok(ADDRESS.to_string()));

        let prefixed = format!("0x{}", ADDRESS);
        let result = convert("address", "address", &prefixed, get_test_network_id);
        assert_eq!(result, Ok(ADDRESS.to_string()));
    }

    #[test]
    fn test_address_to_account_id() {
        let result = convert("address", "accountId", ADDRESS, get_test_network_id);
        assert_eq!(result, Ok(ACCOUNT_ID.to_string()));

        let prefixed = format!("0x{}", ADDRESS);
        let result = convert("address", "accountId", &prefixed, get_test_network_id);
        assert_eq!(result, Ok(ACCOUNT_ID.to_string()));
    }

    #[test]
    fn test_account_id_to_account_id() {
        let result = convert("accountId", "accountId", ACCOUNT_ID, get_test_network_id);
        assert_eq!(result, Ok(ACCOUNT_ID.to_string()));
    }

    #[test]
    fn test_account_id_to_address() {
        let result = convert("accountId", "address", ACCOUNT_ID, get_test_network_id);
        assert_eq!(result, Ok(ADDRESS.to_string()));
    }
}
