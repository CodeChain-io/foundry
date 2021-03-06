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

macro_rules! define_address_constructor {
    (TOP, $name:ident, $prefix:expr) => {
        fn from_transaction_hash(transaction_hash: ::primitives::H256, index: u64) -> Self {
            let mut hash: ::primitives::H256 = ::ccrypto::Blake::blake_with_key(
                &transaction_hash,
                ::primitives::h128_from_u128(index.into()).as_ref(),
            );
            hash.as_mut()[0] = $prefix;
            $name(hash)
        }
    };
    (MODULE, $name:ident, $prefix:expr) => {
        fn from_key_with_storage_id<T: AsRef<[u8]>>(input: T, storage_id: ::ctypes::StorageId) -> Self {
            let mut hash: ::primitives::H256 = ::ccrypto::Blake::blake(&input);
            hash[0..2].copy_from_slice(&[$prefix, 0]);

            debug_assert_eq!(::std::mem::size_of::<u16>(), ::std::mem::size_of::<::ctypes::StorageId>());
            let storage_id_bytes: [u8; 2] = storage_id.to_be_bytes();
            hash[2..4].copy_from_slice(&storage_id_bytes);

            $name(hash)
        }
    };
}

macro_rules! define_id_getter {
    (TOP) => {};
    (MODULE) => {
        pub fn storage_id(&self) -> ::ctypes::StorageId {
            debug_assert_eq!(::std::mem::size_of::<u16>(), ::std::mem::size_of::<StorageId>());
            let storage_id_bytes: [u8; 2] = [self.0[2], self.0[3]];
            ::ctypes::StorageId::from_be_bytes(storage_id_bytes)
        }
    };
}

macro_rules! impl_address {
    ($type:ident, $name:ident, $prefix:expr) => {
        impl $name {
            define_address_constructor!($type, $name, $prefix);

            define_id_getter!($type);

            pub fn from_hash(hash: ::primitives::H256) -> Option<Self> {
                if Self::is_valid_format(&hash) {
                    Some($name(hash))
                } else {
                    None
                }
            }

            pub fn is_valid_format(hash: &::primitives::H256) -> bool {
                hash[0] == $prefix
            }
        }

        impl From<$name> for ::primitives::H256 {
            fn from(a: $name) -> Self {
                a.0
            }
        }

        impl<'a> From<&'a $name> for &'a ::primitives::H256 {
            fn from(a: &'a $name) -> Self {
                &a.0
            }
        }

        impl ::std::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl AsRef<[u8]> for $name {
            fn as_ref(&self) -> &[u8] {
                self.0.as_ref()
            }
        }
    };
}
