use ccrypto::blake256;
use ckey::{Ed25519Public as Public, NetworkId, Signature};
use primitives::{Bytes, H256};

#[allow(dead_code)]
pub type ErroneousTransactions = Vec<SignedTransaction>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Account {
    pub balance: u64,
    pub sequence: u64,
}

impl From<Bytes> for Account {
    fn from(bytes: Bytes) -> Account {
        serde_cbor::from_slice(&bytes).unwrap()
    }
}

impl Default for Account {
    fn default() -> Self {
        Account {
            balance: 0,
            sequence: 0,
        }
    }
}

#[allow(dead_code)]
impl Account {
    pub fn new(balance: u64, sequence: u64) -> Account {
        Account {
            balance,
            sequence,
        }
    }

    pub fn to_bytes(&self) -> Bytes {
        serde_cbor::to_vec(&self).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Transaction {
    pub seq: u64,
    pub fee: u64,
    pub network_id: NetworkId,
    pub action: Action,
}

impl Transaction {
    pub fn hash(&self) -> H256 {
        let serialized = serde_cbor::to_vec(&self).unwrap();
        blake256(serialized)
    }
}

#[derive(Clone)]
pub struct SignedTransaction {
    pub signature: Signature,
    pub signer_public: Public,
    pub tx: Transaction,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Action {
    Pay {
        sender: Public,
        receiver: Public,
        quantity: u64,
    },
}

#[allow(dead_code)]
impl Action {
    pub fn min_fee(&self) -> u64 {
        // Where can we initialize the min fee
        // We need both consensus-defined minimum fee and machine-defined minimum fee
        unimplemented!()
    }
}
