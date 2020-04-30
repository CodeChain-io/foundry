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

use super::version::Version;
use crate::{Ed25519Public as Public, NetworkId};
use std::cmp;
use std::vec::Vec;

fn encode_byte(input: u8) -> Option<u8> {
    static GEOHASH_ENCODING_TABLE: [u8; 32] = [
        b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'j',
        b'k', b'm', b'n', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'z',
    ];
    GEOHASH_ENCODING_TABLE.get(input as usize).cloned()
}

fn rearrange_bits(data: &[u8], from: usize, into: usize) -> Vec<u8> {
    let mut vec = Vec::with_capacity((data.len() * from + (into - 1)) / into);

    let mut group_index = 0;
    let mut group_required_bits = into;

    for val in data {
        let mut ungrouped_bits = from;

        while ungrouped_bits > 0 {
            let min = cmp::min(group_required_bits, ungrouped_bits);
            let min_mask = (1 << min) - 1;

            if group_required_bits == into {
                vec.push(0);
            }

            if ungrouped_bits >= group_required_bits {
                vec[group_index] |= (val >> (ungrouped_bits - group_required_bits)) & min_mask;
            } else {
                vec[group_index] |= (val & min_mask) << (group_required_bits - ungrouped_bits);
            }

            group_required_bits -= min;
            if group_required_bits == 0 {
                group_index += 1;
                group_required_bits = into;
            }
            ungrouped_bits -= min;
        }
    }
    vec
}

fn encode_bytes(bytes: &[u8]) -> Option<Vec<u8>> {
    bytes.iter().map(|byte| encode_byte(*byte)).collect::<Option<Vec<_>>>()
}

pub fn calculate(pubkey: &Public, network_id: NetworkId, version: Version) -> String {
    const OUTPUT_SIZE: usize = 5;
    let mut bytes = [0u8; OUTPUT_SIZE];
    for i in 0..(32 / OUTPUT_SIZE) {
        for (j, byte) in bytes.iter_mut().enumerate() {
            *byte ^= pubkey.as_ref()[i * OUTPUT_SIZE + j];
        }
    }
    bytes[3] ^= pubkey.as_ref()[30];
    bytes[4] ^= pubkey.as_ref()[31];

    bytes[0] ^= network_id[0];
    bytes[1] ^= network_id[1];

    bytes[2] ^= u8::from(version);

    let rearranged = rearrange_bits(&bytes, 8, 5);
    encode_bytes(&rearranged)
        .map(|bytes| bytes.into_iter().map(char::from).collect())
        .expect("The byte is smaller than 32.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fail_to_encode_32() {
        assert_eq!(None, encode_byte(32));
    }

    #[test]
    fn rearrange_bits_from_8_into_5() {
        let vec = vec![0b1110_1110, 0b1110_1110, 0b1110_1110, 0b1110_1110, 0b1110_1110];
        let rearranged = rearrange_bits(&vec, 8, 5);
        assert_eq!(rearranged, vec![0b11101, 0b11011, 0b10111, 0b01110, 0b11101, 0b11011, 0b10111, 0b01110]);
    }

    #[test]
    fn rearrange_bits_from_5_into_8() {
        let vec = vec![0b11101, 0b11011, 0b10111, 0b01110, 0b11101, 0b11011, 0b10111, 0b01110];
        let rearranged = rearrange_bits(&vec, 5, 8);
        assert_eq!(rearranged, vec![0b1110_1110, 0b1110_1110, 0b1110_1110, 0b1110_1110, 0b1110_1110]);
    }

    #[test]
    fn rearrange_bits_from_8_into_5_padded() {
        let vec = vec![0b1110_1110, 0b1110_1110, 0b1110_1110];
        let rearranged = rearrange_bits(&vec, 8, 5);
        assert_eq!(rearranged, vec![0b11101, 0b11011, 0b10111, 0b01110, 0b11100]);
    }

    #[test]
    fn rearrange_bits_from_5_into_8_padded() {
        let vec = vec![0b11101, 0b11011, 0b10111, 0b01110, 0b11101];
        let rearranged = rearrange_bits(&vec, 5, 8);
        assert_eq!(rearranged, vec![0b1110_1110, 0b1110_1110, 0b1110_1110, 0b1000_0000]);
    }
}
