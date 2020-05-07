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

use ckey::{
    Ed25519KeyPair as KeyPair, Ed25519Private as Private, Ed25519Public as Public, Error as KeyError, Generator,
    Password, Random,
};
use ckeystore::accounts_dir::MemoryDirectory;
use ckeystore::{DecryptedAccount, Error as KeystoreError, KeyStore, SecretStore, SimpleSecretStore};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Type of unlock.
#[derive(Clone, PartialEq)]
enum Unlock {
    /// If account is unlocked temporarily, it should be locked after first usage.
    OneTime,
    /// Account unlocked permanently can always sign message.
    /// Use with caution.
    Perm,
    /// Account unlocked with a timeout
    Timed(Instant),
}

/// Data associated with account.
#[derive(Clone)]
struct UnlockedPassword {
    unlock: Unlock,
    password: Password,
}

/// Signing error
#[derive(Debug)]
pub enum Error {
    /// Account is not unlocked
    NotUnlocked,
    /// Account does not exist.
    NotFound,
    /// Key error.
    KeyError(KeyError),
    /// Keystore error.
    KeystoreError(KeystoreError),
}

impl From<KeyError> for Error {
    fn from(e: KeyError) -> Self {
        Error::KeyError(e)
    }
}

impl From<KeystoreError> for Error {
    fn from(e: KeystoreError) -> Self {
        Error::KeystoreError(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Error::NotUnlocked => write!(f, "Account is locked"),
            Error::NotFound => write!(f, "Account does not exist"),
            Error::KeyError(e) => write!(f, "{}", e),
            Error::KeystoreError(e) => write!(f, "{}", e),
        }
    }
}

pub struct AccountProvider {
    /// Unlocked account data.
    unlocked: RwLock<HashMap<Public, UnlockedPassword>>,
    keystore: KeyStore,
}

impl AccountProvider {
    pub fn new(keystore: KeyStore) -> Arc<Self> {
        Arc::new(Self {
            unlocked: RwLock::new(HashMap::new()),
            keystore,
        })
    }

    /// Creates not disk backed provider.
    pub fn transient_provider() -> Arc<Self> {
        Arc::new(Self {
            unlocked: RwLock::new(HashMap::new()),
            keystore: KeyStore::open(Box::new(MemoryDirectory::default())).unwrap(),
        })
    }

    pub fn new_account_and_public(&self, password: &Password) -> Result<Public, Error> {
        let acc: KeyPair = Random.generate().expect("ed25519 context has generation capabilities; qed");
        self.insert_account(acc.get_private(), password)
    }

    pub fn insert_account(&self, private: Private, password: &Password) -> Result<Public, Error> {
        let public = private.public_key();
        self.keystore.insert_account(private, password)?;
        Ok(public)
    }

    pub fn remove_account(&self, pubkey: Public) -> Result<(), Error> {
        self.keystore.remove_account(&pubkey)?;
        Ok(())
    }

    pub fn has_account(&self, pubkey: &Public) -> Result<bool, Error> {
        let has = self.keystore.has_account(pubkey)?;
        Ok(has)
    }

    pub fn get_list(&self) -> Result<Vec<Public>, Error> {
        let publics = self.keystore.accounts()?;
        Ok(publics)
    }

    pub fn import_wallet(&self, json: &[u8], password: &Password) -> Result<Public, Error> {
        Ok(self.keystore.import_wallet(json, password, false)?)
    }

    pub fn change_password(
        &self,
        pubkey: Public,
        old_password: &Password,
        new_password: &Password,
    ) -> Result<(), Error> {
        self.keystore.change_password(&pubkey, &old_password, &new_password)?;
        Ok(())
    }

    /// Unlocks account permanently.
    pub fn unlock_account_permanently(&self, account: Public, password: Password) -> Result<(), KeystoreError> {
        self.unlock_account(account, password, Unlock::Perm)
    }

    /// Unlocks account temporarily (for one signing).
    pub fn unlock_account_temporarily(&self, account: Public, password: Password) -> Result<(), KeystoreError> {
        self.unlock_account(account, password, Unlock::OneTime)
    }

    /// Unlocks account temporarily with a timeout.
    pub fn unlock_account_timed(
        &self,
        account: Public,
        password: Password,
        duration: Duration,
    ) -> Result<(), KeystoreError> {
        self.unlock_account(account, password, Unlock::Timed(Instant::now() + duration))
    }

    /// Helper method used for unlocking accounts.
    fn unlock_account(&self, pubkey: Public, password: Password, unlock: Unlock) -> Result<(), KeystoreError> {
        // check if account is already unlocked permanently, if it is, do nothing
        let mut unlocked = self.unlocked.write();
        if let Some(data) = unlocked.get(&pubkey) {
            if let Unlock::Perm = data.unlock {
                return Ok(())
            }
        }

        if !self.keystore.test_password(&pubkey, &password)? {
            return Err(KeystoreError::InvalidPassword)
        }

        let unlocked_account = UnlockedPassword {
            unlock,
            password,
        };

        unlocked.insert(pubkey, unlocked_account);
        Ok(())
    }

    pub fn get_unlocked_account(&self, pubkey: &Public) -> Result<ScopedAccount<'_>, Error> {
        let mut unlocked = self.unlocked.write();
        let data = unlocked.get(pubkey).ok_or(Error::NotUnlocked)?.clone();
        if let Unlock::OneTime = data.unlock {
            unlocked.remove(pubkey).expect("data exists: so key must exist: qed");
        }
        if let Unlock::Timed(ref end) = data.unlock {
            if Instant::now() > *end {
                unlocked.remove(pubkey).expect("data exists: so key must exist: qed");
                return Err(Error::NotUnlocked)
            }
        }

        let decrypted = self.decrypt_account(pubkey, &data.password)?;
        Ok(ScopedAccount::from(decrypted))
    }

    fn decrypt_account(&self, pubkey: &Public, password: &Password) -> Result<DecryptedAccount, KeystoreError> {
        self.keystore.decrypt_account(pubkey, password)
    }

    pub fn get_account(&self, pubkey: &Public, password: Option<&Password>) -> Result<ScopedAccount<'_>, Error> {
        match password {
            Some(password) => Ok(ScopedAccount::from(self.decrypt_account(pubkey, password)?)),
            None => self.get_unlocked_account(pubkey),
        }
    }
}

// UnlockedAccount should have limited lifetime
pub struct ScopedAccount<'a> {
    decrypted: DecryptedAccount,
    phantom: PhantomData<&'a ()>,
}

impl<'a> Deref for ScopedAccount<'a> {
    type Target = DecryptedAccount;

    fn deref(&self) -> &DecryptedAccount {
        &self.decrypted
    }
}

impl<'a> ScopedAccount<'a> {
    fn from(decrypted: DecryptedAccount) -> ScopedAccount<'a> {
        ScopedAccount {
            decrypted,
            phantom: PhantomData::default(),
        }
    }

    pub fn disclose(self) -> DecryptedAccount {
        self.decrypted
    }
}

#[cfg(test)]
mod tests {
    use ckey::{Ed25519KeyPair as KeyPair, Generator, KeyPairTrait, Random};

    use super::AccountProvider;

    #[test]
    fn unlock_account_temp() {
        let kp: KeyPair = Random.generate().unwrap();
        let ap = AccountProvider::transient_provider();
        assert!(ap.insert_account(kp.private().clone(), &"test".into()).is_ok());
        assert!(ap.unlock_account_temporarily(*kp.public(), "test1".into()).is_err());
        assert!(ap.unlock_account_temporarily(*kp.public(), "test".into()).is_ok());
        assert!(ap.get_account(&kp.public(), None).is_ok());
        assert!(ap.get_account(&kp.public(), None).is_err());
    }

    #[test]
    fn unlock_account_perm() {
        let kp: KeyPair = Random.generate().unwrap();
        let ap = AccountProvider::transient_provider();
        assert!(ap.insert_account(kp.private().clone(), &"test".into()).is_ok());
        assert!(ap.unlock_account_permanently(*kp.public(), "test1".into()).is_err());
        assert!(ap.unlock_account_permanently(*kp.public(), "test".into()).is_ok());
        assert!(ap.get_account(&kp.public(), None).is_ok());
        assert!(ap.get_account(&kp.public(), None).is_ok());
        assert!(ap.unlock_account_temporarily(*kp.public(), "test".into()).is_ok());
        assert!(ap.get_account(&kp.public(), None).is_ok());
        assert!(ap.get_account(&kp.public(), None).is_ok());
    }
}
