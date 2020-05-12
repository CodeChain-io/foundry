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

mod backup;
mod chain_notify;
mod engine;
mod evidence_collector;
mod message;
mod network;
mod params;
pub mod types;
pub mod vote_collector;
mod vote_regression_checker;
mod worker;

use self::chain_notify::TendermintChainNotify;
pub use self::evidence_collector::Evidence;
pub use self::message::{ConsensusMessage, VoteOn, VoteStep};
pub use self::params::{TendermintParams, TimeGapParams, TimeoutParams};
pub use self::types::{Height, Step, View};
pub use super::ValidatorSet;
use crate::client::ConsensusClient;
use crate::consensus::DynamicValidator;
use crate::snapshot_notify::NotifySender as SnapshotNotifySender;
use crate::ChainNotify;
use crossbeam_channel as crossbeam;
use ctimer::TimerToken;
use parking_lot::RwLock;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Weak};
use std::thread::JoinHandle;

/// Timer token representing the consensus step timeouts.
const ENGINE_TIMEOUT_TOKEN_NONCE_BASE: TimerToken = 23;
/// Timer token for empty proposal blocks.
const ENGINE_TIMEOUT_EMPTY_PROPOSAL: TimerToken = 22;
/// Timer token for broadcasting step state.
const ENGINE_TIMEOUT_BROADCAST_STEP_STATE: TimerToken = 21;

/// Unit: second
const ENGINE_TIMEOUT_BROADCAT_STEP_STATE_INTERVAL: u64 = 1;

/// ConsensusEngine using `Tendermint` consensus algorithm
pub struct Tendermint {
    client: RwLock<Option<Weak<dyn ConsensusClient>>>,
    external_params_initializer: crossbeam::Sender<TimeGapParams>,
    extension_initializer: crossbeam::Sender<(crossbeam::Sender<network::Event>, Weak<dyn ConsensusClient>)>,
    snapshot_notify_sender_initializer: crossbeam::Sender<SnapshotNotifySender>,
    timeouts: TimeoutParams,
    join: Option<JoinHandle<()>>,
    quit_tendermint: crossbeam::Sender<()>,
    inner: crossbeam::Sender<worker::Event>,
    validators: Arc<dyn ValidatorSet>,
    /// Chain notify
    chain_notify: Arc<TendermintChainNotify>,
    has_signer: AtomicBool,
}

impl Drop for Tendermint {
    fn drop(&mut self) {
        self.quit_tendermint.send(()).unwrap();
        if let Some(handler) = self.join.take() {
            handler.join().unwrap();
        }
    }
}

impl Tendermint {
    /// Create a new instance of Tendermint engine
    pub fn new(our_params: TendermintParams) -> Arc<Self> {
        let validators = Arc::new(DynamicValidator::default());
        let timeouts = our_params.timeouts;

        let (
            join,
            external_params_initializer,
            extension_initializer,
            snapshot_notify_sender_initializer,
            inner,
            quit_tendermint,
        ) = worker::spawn(Arc::clone(&validators));
        let chain_notify = Arc::new(TendermintChainNotify::new(inner.clone()));

        Arc::new(Tendermint {
            client: Default::default(),
            external_params_initializer,
            extension_initializer,
            snapshot_notify_sender_initializer,
            timeouts,
            join: Some(join),
            quit_tendermint,
            inner,
            validators,
            chain_notify,
            has_signer: false.into(),
        })
    }

    fn client(&self) -> Option<Arc<dyn ConsensusClient>> {
        self.client.read().as_ref()?.upgrade()
    }
}

const SEAL_FIELDS: usize = 4;

#[cfg(test)]
mod tests {
    use ckey::{Ed25519Private as Private, Ed25519Public as Public};
    use ctypes::Header;

    use super::super::BitSet;
    use super::message::VoteStep;
    use crate::account_provider::AccountProvider;
    use crate::client::TestBlockChainClient;
    use crate::consensus::Seal;
    use crate::error::BlockError;
    use crate::error::Error;
    use crate::scheme::Scheme;

    use super::*;

    /// Accounts inserted with "0" and "1" are validators. First proposer is "0".
    fn setup() -> (Scheme, Arc<AccountProvider>, Arc<TestBlockChainClient>) {
        let tap = AccountProvider::transient_provider();
        let scheme = Scheme::new_test_tendermint();
        let test = TestBlockChainClient::new_with_scheme(Scheme::new_test_tendermint());

        let test_client: Arc<TestBlockChainClient> = Arc::new(test);
        let consensus_client = Arc::clone(&test_client) as Arc<dyn ConsensusClient>;
        scheme.engine.register_client(Arc::downgrade(&consensus_client));
        (scheme, tap, test_client)
    }

    fn insert_and_unlock(tap: &Arc<AccountProvider>, acc: &str) -> Public {
        let addr = tap.insert_account(Private::random(), &acc.into()).unwrap();
        tap.unlock_account_permanently(addr, acc.into()).unwrap();
        addr
    }

    #[test]
    #[ignore] // FIXME
    fn verification_fails_on_short_seal() {
        let engine = Scheme::new_test_tendermint().engine;
        let header = Header::default();

        let verify_result = engine.verify_header_basic(&header);

        match verify_result {
            Err(Error::Block(BlockError::InvalidSealArity(_))) => {}
            Err(err) => {
                panic!("should be block seal-arity mismatch error (got {:?})", err);
            }
            _ => {
                panic!("Should be error, got Ok");
            }
        }
    }

    #[test]
    #[ignore] // FIXME
    fn parent_block_existence_checking() {
        let (spec, tap, _c) = setup();
        let engine = spec.engine;

        let mut header = Header::default();
        header.set_number(4);
        let proposer = insert_and_unlock(&tap, "0");
        header.set_author(proposer);
        header.set_parent_hash(Default::default());

        let vote_on = VoteOn {
            step: VoteStep::new(3, 0, Step::Precommit),
            block_hash: Some(*header.parent_hash()),
        };
        let signature2 = tap.get_account(&proposer, None).unwrap().sign(&vote_on.hash()).unwrap();

        let seal = Seal::Tendermint {
            prev_view: 0,
            cur_view: 0,
            precommits: vec![signature2],
            precommit_bitset: BitSet::new_with_indices(&[2]),
        }
        .seal_fields()
        .unwrap();
        header.set_seal(seal);

        println!(".....");
        assert!(engine.verify_block_external(&header).is_err());
    }
}
