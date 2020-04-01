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

mod actions;

pub use self::actions::{
    ban, change_params, close_term, delegate_ccs, init_stake, jail, redelegate, release_jailed_prisoners,
    revert_delegations, revoke, self_nominate, transfer_ccs, update_candidates,
};
use super::TopStateView;
use crate::{StateResult, TopLevelState};
use ccrypto::blake256;
use ckey::Ed25519Public as Public;
use ctypes::errors::SyntaxError;
use primitives::H256;
use rlp::{Encodable, RlpStream};
use std::convert::From;

pub trait DoubleVoteHandler: Send + Sync {
    fn execute(&self, message1: &[u8], state: &mut TopLevelState, fee_payer: &Public) -> StateResult<()>;
    fn verify(&self, message1: &[u8], message2: &[u8]) -> Result<(), SyntaxError>;
}

pub fn query(key_fragment: &[u8], state: &TopLevelState) -> StateResult<Option<Vec<u8>>> {
    let key = StakeKeyBuilder::key_from_fragment(key_fragment);
    let some_action_data = state.action_data(&key)?.map(Vec::from);
    Ok(some_action_data)
}

pub struct StakeKeyBuilder {
    rlp: RlpStream,
}

impl StakeKeyBuilder {
    fn prepare() -> StakeKeyBuilder {
        let mut rlp = RlpStream::new_list(2);
        rlp.append(&"Stake");
        StakeKeyBuilder {
            rlp,
        }
    }

    pub fn key_from_fragment(key_fragment: &[u8]) -> H256 {
        let mut builder = Self::prepare();
        builder.rlp.append_raw(&key_fragment, 1);
        builder.into_key()
    }

    pub fn new(fragment_length: usize) -> StakeKeyBuilder {
        let mut builder = Self::prepare();
        builder.rlp.begin_list(fragment_length);
        builder
    }

    pub fn append<E>(mut self, e: &E) -> StakeKeyBuilder
    where
        E: Encodable, {
        self.rlp.append(e);
        self
    }

    pub fn into_key(self) -> H256 {
        blake256(self.rlp.as_raw())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::stake::{Candidate, Candidates, Delegation, Jail, StakeAccount};
    use crate::tests::helpers;
    use crate::{NextValidators, TopLevelState, TopState, TopStateView};
    use ckey::Ed25519Public as Public;
    use ctypes::{CommonParams, ConsensusParams, TransactionLocation};
    use std::collections::HashMap;

    #[test]
    fn action_data_key_builder_raw_fragment_and_list_are_same() {
        let key1 = StakeKeyBuilder::new(3).append(&"key").append(&"fragment").append(&"has trailing list").into_key();

        let mut rlp = RlpStream::new_list(3);
        rlp.append(&"key").append(&"fragment").append(&"has trailing list");
        let key2 = StakeKeyBuilder::key_from_fragment(rlp.as_raw());
        assert_eq!(key1, key2);
    }

    fn metadata_for_election() -> TopLevelState {
        let mut params = CommonParams::default_for_test();
        let mut state = helpers::get_temp_state_with_metadata(params, ConsensusParams::default_for_test());
        state.metadata().unwrap().unwrap().set_params(CommonParams::default_for_test());
        params.set_dynamic_validator_params_for_test(30, 10, 3, 20, 30, 4, 1000, 10000, 100);
        assert_eq!(Ok(()), state.update_params(0, params));
        state
    }

    fn increase_term_id_until(state: &mut TopLevelState, term_id: u64) {
        let mut block_num = state.metadata().unwrap().unwrap().last_term_finished_block_num() + 1;
        while state.metadata().unwrap().unwrap().current_term_id() != term_id {
            assert_eq!(Ok(()), state.increase_term_id(block_num));
            block_num += 1;
        }
    }

    #[test]
    fn self_nominate_returns_deposits_after_expiration() {
        let pubkey = Public::random();

        let mut state = metadata_for_election();
        increase_term_id_until(&mut state, 29);
        state.add_balance(&pubkey, 1000).unwrap();

        init_stake(&mut state, Default::default(), Default::default(), Default::default()).unwrap();

        let nomination_starts_at = TransactionLocation {
            block_number: 0,
            transaction_index: 0,
        };
        // TODO: change with stake::execute()
        self_nominate(&mut state, &pubkey, 200, 0, 30, nomination_starts_at, b"".to_vec()).unwrap();

        let next_validators = Vec::from(NextValidators::load_from_state(&state).unwrap());
        close_term(&mut state, &next_validators, &[]).unwrap();

        let current_term = state.metadata().unwrap().unwrap().current_term_id();
        let released_addresses = Jail::load_from_state(&state).unwrap().released_addresses(current_term);
        release_jailed_prisoners(&mut state, &released_addresses).unwrap();

        state.increase_term_id(pseudo_term_to_block_num_calculator(29)).unwrap();

        assert_eq!(state.balance(&pubkey).unwrap(), 800, "Should keep nomination before expiration");
        let candidates = Candidates::load_from_state(&state).unwrap();
        assert_eq!(
            candidates.get_candidate(&pubkey),
            Some(&Candidate {
                pubkey,
                deposit: 200,
                nomination_ends_at: 30,
                nomination_starts_at_block_number: 0,
                nomination_starts_at_transaction_index: 0,
                metadata: b"".to_vec(),
            }),
            "Keep deposit before expiration",
        );

        let next_validators = Vec::from(NextValidators::load_from_state(&state).unwrap());
        close_term(&mut state, &next_validators, &[]).unwrap();

        let current_term = state.metadata().unwrap().unwrap().current_term_id();
        let released_addresses = Jail::load_from_state(&state).unwrap().released_addresses(current_term);
        release_jailed_prisoners(&mut state, &released_addresses).unwrap();

        state.increase_term_id(pseudo_term_to_block_num_calculator(30)).unwrap();

        assert_eq!(state.balance(&pubkey).unwrap(), 1000, "Return deposit after expiration");
        let candidates = Candidates::load_from_state(&state).unwrap();
        assert_eq!(candidates.get_candidate(&pubkey), None, "Removed from candidates after expiration");
    }

    #[test]
    fn self_nominate_reverts_delegations_after_expiration() {
        let pubkey = Public::random();
        let delegator = Public::random();

        let mut state = metadata_for_election();
        increase_term_id_until(&mut state, 29);
        state.add_balance(&delegator, 1000).unwrap();

        let genesis_stakes = {
            let mut genesis_stakes = HashMap::new();
            genesis_stakes.insert(delegator, 100);
            genesis_stakes
        };
        init_stake(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();

        let nomination_starts_at = TransactionLocation {
            block_number: 0,
            transaction_index: 0,
        };
        // TODO: change with stake::execute()
        self_nominate(&mut state, &pubkey, 0, 0, 30, nomination_starts_at, b"".to_vec()).unwrap();

        let quantity = 40;
        delegate_ccs(&mut state, &delegator, &pubkey, quantity).unwrap();

        let next_validators = Vec::from(NextValidators::load_from_state(&state).unwrap());
        close_term(&mut state, &next_validators, &[]).unwrap();

        let current_term = state.metadata().unwrap().unwrap().current_term_id();
        let released_addresses = Jail::load_from_state(&state).unwrap().released_addresses(current_term);
        release_jailed_prisoners(&mut state, &released_addresses).unwrap();

        state.increase_term_id(pseudo_term_to_block_num_calculator(29)).unwrap();

        let account = StakeAccount::load_from_state(&state, &delegator).unwrap();
        assert_eq!(account.balance, 100 - 40);
        let delegation = Delegation::load_from_state(&state, &delegator).unwrap();
        assert_eq!(delegation.get_quantity(&pubkey), 40, "Should keep delegation before expiration");

        let next_validators = Vec::from(NextValidators::load_from_state(&state).unwrap());
        close_term(&mut state, &next_validators, &[]).unwrap();

        let current_term = state.metadata().unwrap().unwrap().current_term_id();
        let released_addresses = Jail::load_from_state(&state).unwrap().released_addresses(current_term);
        release_jailed_prisoners(&mut state, &released_addresses).unwrap();

        state.increase_term_id(pseudo_term_to_block_num_calculator(30)).unwrap();

        let account = StakeAccount::load_from_state(&state, &delegator).unwrap();
        assert_eq!(account.balance, 100);
        let delegation = Delegation::load_from_state(&state, &delegator).unwrap();
        assert_eq!(delegation.get_quantity(&pubkey), 0, "Should revert before expiration");
    }

    #[test]
    fn cannot_self_nominate_while_custody() {
        let pubkey = Public::random();

        let mut state = metadata_for_election();
        state.add_balance(&pubkey, 1000).unwrap();

        init_stake(&mut state, Default::default(), Default::default(), Default::default()).unwrap();

        // TODO: change with stake::execute()
        let deposit = 200;
        let nominate_expire = 5;
        let custody_until = 10;
        let released_at = 20;
        let nomination_starts_at = TransactionLocation {
            block_number: 0,
            transaction_index: 0,
        };
        self_nominate(&mut state, &pubkey, deposit, 0, nominate_expire, nomination_starts_at, b"".to_vec()).unwrap();
        jail(&mut state, &[pubkey], custody_until, released_at).unwrap();

        for current_term in 0..=custody_until {
            let nomination_starts_at = TransactionLocation {
                block_number: 0,
                transaction_index: 0,
            };
            let result = self_nominate(
                &mut state,
                &pubkey,
                0,
                current_term,
                current_term + nominate_expire,
                nomination_starts_at,
                b"".to_vec(),
            );
            assert!(
                result.is_err(),
                "Shouldn't nominate while current_term({}) <= custody_until({})",
                current_term,
                custody_until
            );
            let next_validators = Vec::from(NextValidators::load_from_state(&state).unwrap());
            close_term(&mut state, &next_validators, &[]).unwrap();

            let current_term_id = state.metadata().unwrap().unwrap().current_term_id();
            let released_addresses = Jail::load_from_state(&state).unwrap().released_addresses(current_term_id);
            release_jailed_prisoners(&mut state, &released_addresses).unwrap();

            state.increase_term_id(pseudo_term_to_block_num_calculator(current_term)).unwrap();
        }
    }

    #[test]
    fn can_self_nominate_after_custody() {
        let pubkey = Public::random();

        let mut state = metadata_for_election();
        state.add_balance(&pubkey, 1000).unwrap();

        init_stake(&mut state, Default::default(), Default::default(), Default::default()).unwrap();

        // TODO: change with stake::execute()
        let deposit = 200;
        let nominate_expire = 5;
        let custody_until = 10;
        let released_at = 20;
        let nomination_starts_at = TransactionLocation {
            block_number: 0,
            transaction_index: 0,
        };
        self_nominate(
            &mut state,
            &pubkey,
            deposit,
            0,
            nominate_expire,
            nomination_starts_at,
            b"metadata-before".to_vec(),
        )
        .unwrap();
        jail(&mut state, &[pubkey], custody_until, released_at).unwrap();
        for current_term in 0..=custody_until {
            let next_validators = Vec::from(NextValidators::load_from_state(&state).unwrap());
            close_term(&mut state, &next_validators, &[]).unwrap();

            let current_term_id = state.metadata().unwrap().unwrap().current_term_id();
            let released_addresses = Jail::load_from_state(&state).unwrap().released_addresses(current_term_id);
            release_jailed_prisoners(&mut state, &released_addresses).unwrap();

            state.increase_term_id(pseudo_term_to_block_num_calculator(current_term)).unwrap();
        }

        let current_term = custody_until + 1;
        let additional_deposit = 123;
        let result = self_nominate(
            &mut state,
            &pubkey,
            additional_deposit,
            current_term,
            current_term + nominate_expire,
            nomination_starts_at,
            b"metadata-after".to_vec(),
        );
        assert!(result.is_ok());

        let candidates = Candidates::load_from_state(&state).unwrap();
        assert_eq!(
            candidates.get_candidate(&pubkey),
            Some(&Candidate {
                deposit: deposit + additional_deposit,
                nomination_ends_at: current_term + nominate_expire,
                pubkey,
                nomination_starts_at_block_number: nomination_starts_at.block_number,
                nomination_starts_at_transaction_index: nomination_starts_at.transaction_index,
                metadata: "metadata-after".into()
            }),
            "The prisoner is become a candidate",
        );

        let jail = Jail::load_from_state(&state).unwrap();
        assert_eq!(jail.get_prisoner(&pubkey), None, "The prisoner is removed");

        assert_eq!(state.balance(&pubkey).unwrap(), 1000 - deposit - additional_deposit, "Deposit is accumulated");
    }

    #[test]
    fn jail_released_after() {
        let pubkey = Public::random();

        let mut state = metadata_for_election();
        state.add_balance(&pubkey, 1000).unwrap();

        init_stake(&mut state, Default::default(), Default::default(), Default::default()).unwrap();

        // TODO: change with stake::execute()
        let deposit = 200;
        let nominate_expire = 5;
        let custody_until = 10;
        let released_at = 20;
        let nomination_starts_at = TransactionLocation {
            block_number: 0,
            transaction_index: 0,
        };
        self_nominate(&mut state, &pubkey, deposit, 0, nominate_expire, nomination_starts_at, b"".to_vec()).unwrap();
        jail(&mut state, &[pubkey], custody_until, released_at).unwrap();

        for current_term in 0..released_at {
            let next_validators = Vec::from(NextValidators::load_from_state(&state).unwrap());
            close_term(&mut state, &next_validators, &[]).unwrap();

            let current_term_id = state.metadata().unwrap().unwrap().current_term_id();
            let released_addresses = Jail::load_from_state(&state).unwrap().released_addresses(current_term_id);
            release_jailed_prisoners(&mut state, &released_addresses).unwrap();

            state.increase_term_id(pseudo_term_to_block_num_calculator(current_term)).unwrap();

            let candidates = Candidates::load_from_state(&state).unwrap();
            assert_eq!(candidates.get_candidate(&pubkey), None);

            let jail = Jail::load_from_state(&state).unwrap();
            assert!(jail.get_prisoner(&pubkey).is_some());
        }

        let next_validators = Vec::from(NextValidators::load_from_state(&state).unwrap());
        close_term(&mut state, &next_validators, &[]).unwrap();

        let current_term = state.metadata().unwrap().unwrap().current_term_id();
        let released_addresses = Jail::load_from_state(&state).unwrap().released_addresses(current_term);
        release_jailed_prisoners(&mut state, &released_addresses).unwrap();

        state.increase_term_id(pseudo_term_to_block_num_calculator(released_at)).unwrap();

        let candidates = Candidates::load_from_state(&state).unwrap();
        assert_eq!(candidates.get_candidate(&pubkey), None, "A prisoner should not become a candidate");

        let jail = Jail::load_from_state(&state).unwrap();
        assert_eq!(jail.get_prisoner(&pubkey), None, "A prisoner should be released");

        assert_eq!(state.balance(&pubkey).unwrap(), 1000, "Balance should be restored after being released");
    }

    #[test]
    fn cannot_delegate_until_released() {
        let pubkey = Public::random();
        let delegator = Public::random();

        let mut state = metadata_for_election();
        state.add_balance(&pubkey, 1000).unwrap();

        let genesis_stakes = {
            let mut genesis_stakes = HashMap::new();
            genesis_stakes.insert(delegator, 100);
            genesis_stakes
        };
        init_stake(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();

        // TODO: change with stake::execute()
        let deposit = 200;
        let nominate_expire = 5;
        let custody_until = 10;
        let released_at = 20;
        let nomination_starts_at = TransactionLocation {
            block_number: 0,
            transaction_index: 0,
        };
        self_nominate(&mut state, &pubkey, deposit, 0, nominate_expire, nomination_starts_at, b"".to_vec()).unwrap();
        jail(&mut state, &[pubkey], custody_until, released_at).unwrap();

        for current_term in 0..=released_at {
            let quantity = 1;
            delegate_ccs(&mut state, &delegator, &pubkey, quantity).unwrap_err();

            let next_validators = Vec::from(NextValidators::load_from_state(&state).unwrap());
            close_term(&mut state, &next_validators, &[]).unwrap();

            let current_term_id = state.metadata().unwrap().unwrap().current_term_id();
            let released_addresses = Jail::load_from_state(&state).unwrap().released_addresses(current_term_id);
            release_jailed_prisoners(&mut state, &released_addresses).unwrap();

            state.increase_term_id(pseudo_term_to_block_num_calculator(current_term)).unwrap();
        }

        let quantity = 1;
        delegate_ccs(&mut state, &delegator, &pubkey, quantity).unwrap_err();
    }

    #[test]
    fn kick_reverts_delegations() {
        let pubkey = Public::random();
        let delegator = Public::random();

        let mut state = metadata_for_election();
        state.add_balance(&pubkey, 1000).unwrap();

        let genesis_stakes = {
            let mut genesis_stakes = HashMap::new();
            genesis_stakes.insert(delegator, 100);
            genesis_stakes
        };
        init_stake(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();

        // TODO: change with stake::execute()
        let deposit = 200;
        let nominate_expire = 5;
        let custody_until = 10;
        let released_at = 20;
        let nomination_starts_at = TransactionLocation {
            block_number: 0,
            transaction_index: 0,
        };
        self_nominate(&mut state, &pubkey, deposit, 0, nominate_expire, nomination_starts_at, b"".to_vec()).unwrap();

        let quantity = 40;
        delegate_ccs(&mut state, &delegator, &pubkey, quantity).unwrap();

        jail(&mut state, &[pubkey], custody_until, released_at).unwrap();

        for current_term in 0..=released_at {
            let next_validators = Vec::from(NextValidators::load_from_state(&state).unwrap());
            close_term(&mut state, &next_validators, &[]).unwrap();

            let current_term_id = state.metadata().unwrap().unwrap().current_term_id();
            let released_addresses = Jail::load_from_state(&state).unwrap().released_addresses(current_term_id);
            release_jailed_prisoners(&mut state, &released_addresses).unwrap();

            state.increase_term_id(pseudo_term_to_block_num_calculator(current_term)).unwrap();
        }

        let delegation = Delegation::load_from_state(&state, &delegator).unwrap();
        assert_eq!(delegation.get_quantity(&pubkey), 0, "Delegation should be reverted");

        let account = StakeAccount::load_from_state(&state, &delegator).unwrap();
        assert_eq!(account.balance, 100, "Delegation should be reverted");
    }

    #[test]
    fn self_nomination_before_kick_preserves_delegations() {
        let pubkey = Public::random();
        let delegator = Public::random();

        let mut state = metadata_for_election();
        state.add_balance(&pubkey, 1000).unwrap();

        let genesis_stakes = {
            let mut genesis_stakes = HashMap::new();
            genesis_stakes.insert(delegator, 100);
            genesis_stakes
        };
        init_stake(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();

        // TODO: change with stake::execute()
        let nominate_expire = 5;
        let custody_until = 10;
        let released_at = 20;
        let nomination_starts_at = TransactionLocation {
            block_number: 0,
            transaction_index: 0,
        };
        self_nominate(&mut state, &pubkey, 0, 0, nominate_expire, nomination_starts_at, b"".to_vec()).unwrap();

        let quantity = 40;
        delegate_ccs(&mut state, &delegator, &pubkey, quantity).unwrap();

        jail(&mut state, &[pubkey], custody_until, released_at).unwrap();

        for current_term in 0..custody_until {
            let next_validators = Vec::from(NextValidators::load_from_state(&state).unwrap());
            close_term(&mut state, &next_validators, &[]).unwrap();

            let current_term_id = state.metadata().unwrap().unwrap().current_term_id();
            let released_addresses = Jail::load_from_state(&state).unwrap().released_addresses(current_term_id);
            release_jailed_prisoners(&mut state, &released_addresses).unwrap();

            state.increase_term_id(pseudo_term_to_block_num_calculator(current_term)).unwrap();
        }

        let current_term = custody_until + 1;
        let nomination_starts_at = TransactionLocation {
            block_number: 0,
            transaction_index: 0,
        };
        let result = self_nominate(
            &mut state,
            &pubkey,
            0,
            current_term,
            current_term + nominate_expire,
            nomination_starts_at,
            b"".to_vec(),
        );
        assert!(result.is_ok());

        let delegation = Delegation::load_from_state(&state, &delegator).unwrap();
        assert_eq!(delegation.get_quantity(&pubkey), 40, "Delegation should be preserved");

        let account = StakeAccount::load_from_state(&state, &delegator).unwrap();
        assert_eq!(account.balance, 100 - 40, "Delegation should be preserved");
    }

    fn pseudo_term_to_block_num_calculator(term_id: u64) -> u64 {
        term_id * 10 + 1
    }
}
