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

use crate::account_provider::{AccountProvider, Error as AccountProviderError};
use ckey::{Ed25519Public as Public, Signature};
use ckeystore::DecryptedAccount;
use primitives::H256;
use std::sync::Arc;

/// Everything that an Engine needs to sign messages.
pub struct EngineSigner {
    account_provider: Arc<AccountProvider>,
    signer: Option<Public>,
    decrypted_account: Option<DecryptedAccount>,
}

impl Default for EngineSigner {
    fn default() -> Self {
        EngineSigner {
            account_provider: AccountProvider::transient_provider(),
            signer: Default::default(),
            decrypted_account: Default::default(),
        }
    }
}

impl EngineSigner {
    // TODO: remove decrypted_account after some timeout
    pub fn set_to_keep_decrypted_account(&mut self, ap: Arc<AccountProvider>, pubkey: Public) {
        let account =
            ap.get_unlocked_account(&pubkey).expect("The pubkey must be registered in AccountProvider").disclose();

        self.account_provider = ap;
        self.signer = Some(pubkey);
        self.decrypted_account = Some(account);
        cinfo!(ENGINE, "Setting Engine signer to {:?} (retaining)", pubkey);
    }

    /// Sign a message hash with Ed25519.
    pub fn sign(&self, hash: H256) -> Result<Signature, AccountProviderError> {
        let pubkey = self.signer.unwrap_or_else(Default::default);
        let result = match &self.decrypted_account {
            Some(account) => account.sign(&hash)?,
            None => {
                let account = self.account_provider.get_unlocked_account(&pubkey)?;
                account.sign(&hash)?
            }
        };
        Ok(result)
    }

    /// Public Key of signer.
    pub fn public(&self) -> Option<&Public> {
        self.signer.as_ref()
    }

    /// Check if the given pubkey is the signing address.
    pub fn is_signer(&self, pubkey: &Public) -> bool {
        self.signer.map_or(false, |signer| *pubkey == signer)
    }
}
