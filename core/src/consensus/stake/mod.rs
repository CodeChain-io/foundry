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

use crate::client::ConsensusClient;
use ckey::{public_to_address, Address};
use cstate::{
    ban, jail, release_jailed_prisoners, revert_delegations, update_candidates, DoubleVoteHandler, StateResult,
    TopLevelState, TopState, TopStateView,
};
use ctypes::errors::{RuntimeError, SyntaxError};
use parking_lot::RwLock;
use std::sync::{Arc, Weak};

use super::ValidatorSet;
use crate::consensus::ConsensusMessage;

#[derive(Default)]
pub struct Stake {
    client: RwLock<Option<Weak<dyn ConsensusClient>>>,
    validators: RwLock<Option<Weak<dyn ValidatorSet>>>,
}

impl Stake {
    pub fn register_resources(&self, client: Weak<dyn ConsensusClient>, validators: Weak<dyn ValidatorSet>) {
        *self.client.write() = Some(Weak::clone(&client));
        *self.validators.write() = Some(Weak::clone(&validators));
    }
}

impl DoubleVoteHandler for Stake {
    fn execute(&self, message1: &[u8], state: &mut TopLevelState, sender_address: &Address) -> StateResult<()> {
        let message1: ConsensusMessage =
            rlp::decode(message1).map_err(|err| RuntimeError::FailedToHandleCustomAction(err.to_string()))?;
        let validators =
            self.validators.read().as_ref().and_then(Weak::upgrade).expect("ValidatorSet must be initialized");
        let client = self.client.read().as_ref().and_then(Weak::upgrade).expect("Client must be initialized");

        execute_report_double_vote(message1, state, sender_address, &*client, &*validators)?;
        Ok(())
    }

    fn verify(&self, message1: &[u8], message2: &[u8]) -> Result<(), SyntaxError> {
        let client: Arc<dyn ConsensusClient> =
            self.client.read().as_ref().and_then(Weak::upgrade).expect("Client should be initialized");
        let validators: Arc<dyn ValidatorSet> =
            self.validators.read().as_ref().and_then(Weak::upgrade).expect("ValidatorSet should be initialized");

        let message1: ConsensusMessage =
            rlp::decode(message1).map_err(|err| SyntaxError::InvalidCustomAction(err.to_string()))?;
        let message2: ConsensusMessage =
            rlp::decode(message2).map_err(|err| SyntaxError::InvalidCustomAction(err.to_string()))?;

        verify_report_double_vote(message1, message2, &*client, &*validators)
    }
}

fn execute_report_double_vote(
    message1: ConsensusMessage,
    state: &mut TopLevelState,
    sender_address: &Address,
    client: &dyn ConsensusClient,
    validators: &dyn ValidatorSet,
) -> StateResult<()> {
    let parent_hash = client.block_header(&(message1.height() - 1).into()).expect("Parent header verified").hash();
    let malicious_user_public = validators.get(&parent_hash, message1.signer_index());

    ban(state, sender_address, public_to_address(&malicious_user_public))
}

pub fn verify_report_double_vote(
    message1: ConsensusMessage,
    message2: ConsensusMessage,
    client: &dyn ConsensusClient,
    validators: &dyn ValidatorSet,
) -> Result<(), SyntaxError> {
    if message1.round() != message2.round() {
        return Err(SyntaxError::InvalidCustomAction(String::from("The messages are from two different voting rounds")))
    }

    let signer_idx1 = message1.signer_index();
    let signer_idx2 = message2.signer_index();

    if signer_idx1 != signer_idx2 {
        return Err(SyntaxError::InvalidCustomAction(format!(
            "Two messages have different signer indexes: {}, {}",
            signer_idx1, signer_idx2
        )))
    }

    assert_eq!(
        message1.height(),
        message2.height(),
        "Heights of both messages must be same because message1.round() == message2.round()"
    );

    let signed_block_height = message1.height();
    if signed_block_height == 0 {
        return Err(SyntaxError::InvalidCustomAction(String::from(
            "Double vote on the genesis block does not make sense",
        )))
    }
    let parent_hash = client
        .block_header(&(signed_block_height - 1).into())
        .ok_or_else(|| {
            SyntaxError::InvalidCustomAction(format!("Cannot get header from the height {}", signed_block_height))
        })?
        .hash();
    let signer_idx1 = message1.signer_index();
    let signer = validators.get(&parent_hash, signer_idx1);
    if !message1.verify(&signer) || !message2.verify(&signer) {
        return Err(SyntaxError::InvalidCustomAction(String::from("Ed25519 signature verification fails")))
    }
    Ok(())
}

pub fn on_term_close(
    state: &mut TopLevelState,
    last_term_finished_block_num: u64,
    inactive_validators: &[Address],
) -> StateResult<()> {
    let metadata = state.metadata()?.expect("The metadata must exist");
    let current_term = metadata.current_term_id();
    ctrace!(ENGINE, "on_term_close. current_term: {}", current_term);

    let (nomination_expiration, custody_until, kick_at) = {
        let metadata = metadata.params();
        let nomination_expiration = metadata.nomination_expiration();
        assert_ne!(0, nomination_expiration);
        let custody_period = metadata.custody_period();
        assert_ne!(0, custody_period);
        let release_period = metadata.release_period();
        assert_ne!(0, release_period);
        (nomination_expiration, current_term + custody_period, current_term + release_period)
    };

    let expired = update_candidates(state, current_term, nomination_expiration, inactive_validators)?;
    let released = release_jailed_prisoners(state, current_term)?;

    let reverted: Vec<_> = expired.into_iter().chain(released).collect();
    revert_delegations(state, &reverted)?;

    jail(state, inactive_validators, custody_until, kick_at)?;

    state.increase_term_id(last_term_finished_block_num)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ckey::Ed25519Public as Public;
    use cstate::tests::helpers;
    use cstate::{
        execute_stake_action, init_stake, self_nominate, Candidate, Candidates, Delegation, Jail, StakeAccount,
        TopStateView,
    };
    use ctypes::transaction::StakeAction;
    use ctypes::CommonParams;
    use std::collections::HashMap;

    fn metadata_for_election() -> TopLevelState {
        let mut params = CommonParams::default_for_test();
        let mut state = helpers::get_temp_state_with_metadata(params);
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
        let address_pubkey = Public::random();
        let address = public_to_address(&address_pubkey);

        let mut state = metadata_for_election();
        increase_term_id_until(&mut state, 29);
        state.add_balance(&address, 1000).unwrap();

        init_stake(&mut state, Default::default(), Default::default(), Default::default()).unwrap();

        // TODO: change with stake::execute()
        self_nominate(&mut state, &address, &address_pubkey, 200, 0, 30, b"".to_vec()).unwrap();

        let result = on_term_close(&mut state, pseudo_term_to_block_num_calculator(29), &[]);
        assert_eq!(result, Ok(()));

        assert_eq!(state.balance(&address).unwrap(), 800, "Should keep nomination before expiration");
        let candidates = Candidates::load_from_state(&state).unwrap();
        assert_eq!(
            candidates.get_candidate(&address),
            Some(&Candidate {
                pubkey: address_pubkey,
                deposit: 200,
                nomination_ends_at: 30,
                metadata: b"".to_vec(),
            }),
            "Keep deposit before expiration",
        );

        let result = on_term_close(&mut state, pseudo_term_to_block_num_calculator(30), &[]);
        assert_eq!(result, Ok(()));

        assert_eq!(state.balance(&address).unwrap(), 1000, "Return deposit after expiration");
        let candidates = Candidates::load_from_state(&state).unwrap();
        assert_eq!(candidates.get_candidate(&address), None, "Removed from candidates after expiration");
    }

    #[test]
    fn self_nominate_reverts_delegations_after_expiration() {
        let address_pubkey = Public::random();
        let address = public_to_address(&address_pubkey);
        let delegator_pubkey = Public::random();
        let delegator = public_to_address(&address_pubkey);

        let mut state = metadata_for_election();
        increase_term_id_until(&mut state, 29);
        state.add_balance(&address, 1000).unwrap();

        let genesis_stakes = {
            let mut genesis_stakes = HashMap::new();
            genesis_stakes.insert(delegator, 100);
            genesis_stakes
        };
        init_stake(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();

        // TODO: change with stake::execute()
        self_nominate(&mut state, &address, &address_pubkey, 0, 0, 30, b"".to_vec()).unwrap();

        let action = StakeAction::DelegateCCS {
            address,
            quantity: 40,
        };
        execute_stake_action(action, &mut state, &delegator, &delegator_pubkey).unwrap();

        let result = on_term_close(&mut state, pseudo_term_to_block_num_calculator(29), &[]);
        assert_eq!(result, Ok(()));

        let account = StakeAccount::load_from_state(&state, &delegator).unwrap();
        assert_eq!(account.balance, 100 - 40);
        let delegation = Delegation::load_from_state(&state, &delegator).unwrap();
        assert_eq!(delegation.get_quantity(&address), 40, "Should keep delegation before expiration");

        let result = on_term_close(&mut state, pseudo_term_to_block_num_calculator(30), &[]);
        assert_eq!(result, Ok(()));

        let account = StakeAccount::load_from_state(&state, &delegator).unwrap();
        assert_eq!(account.balance, 100);
        let delegation = Delegation::load_from_state(&state, &delegator).unwrap();
        assert_eq!(delegation.get_quantity(&address), 0, "Should revert before expiration");
    }

    #[test]
    fn cannot_self_nominate_while_custody() {
        let address_pubkey = Public::random();
        let address = public_to_address(&address_pubkey);

        let mut state = metadata_for_election();
        state.add_balance(&address, 1000).unwrap();

        init_stake(&mut state, Default::default(), Default::default(), Default::default()).unwrap();

        // TODO: change with stake::execute()
        let deposit = 200;
        let nominate_expire = 5;
        let custody_until = 10;
        let released_at = 20;
        self_nominate(&mut state, &address, &address_pubkey, deposit, 0, nominate_expire, b"".to_vec()).unwrap();
        jail(&mut state, &[address], custody_until, released_at).unwrap();

        for current_term in 0..=custody_until {
            let result = self_nominate(
                &mut state,
                &address,
                &address_pubkey,
                0,
                current_term,
                current_term + nominate_expire,
                b"".to_vec(),
            );
            assert!(
                result.is_err(),
                "Shouldn't nominate while current_term({}) <= custody_until({})",
                current_term,
                custody_until
            );
            on_term_close(&mut state, pseudo_term_to_block_num_calculator(current_term), &[]).unwrap();
        }
    }

    #[test]
    fn can_self_nominate_after_custody() {
        let address_pubkey = Public::random();
        let address = public_to_address(&address_pubkey);

        let mut state = metadata_for_election();
        state.add_balance(&address, 1000).unwrap();

        init_stake(&mut state, Default::default(), Default::default(), Default::default()).unwrap();

        // TODO: change with stake::execute()
        let deposit = 200;
        let nominate_expire = 5;
        let custody_until = 10;
        let released_at = 20;
        self_nominate(&mut state, &address, &address_pubkey, deposit, 0, nominate_expire, b"metadata-before".to_vec())
            .unwrap();
        jail(&mut state, &[address], custody_until, released_at).unwrap();
        for current_term in 0..=custody_until {
            on_term_close(&mut state, pseudo_term_to_block_num_calculator(current_term), &[]).unwrap();
        }

        let current_term = custody_until + 1;
        let additional_deposit = 123;
        let result = self_nominate(
            &mut state,
            &address,
            &address_pubkey,
            additional_deposit,
            current_term,
            current_term + nominate_expire,
            b"metadata-after".to_vec(),
        );
        assert!(result.is_ok());

        let candidates = Candidates::load_from_state(&state).unwrap();
        assert_eq!(
            candidates.get_candidate(&address),
            Some(&Candidate {
                deposit: deposit + additional_deposit,
                nomination_ends_at: current_term + nominate_expire,
                pubkey: address_pubkey,
                metadata: "metadata-after".into()
            }),
            "The prisoner is become a candidate",
        );

        let jail = Jail::load_from_state(&state).unwrap();
        assert_eq!(jail.get_prisoner(&address), None, "The prisoner is removed");

        assert_eq!(state.balance(&address).unwrap(), 1000 - deposit - additional_deposit, "Deposit is accumulated");
    }

    #[test]
    fn jail_released_after() {
        let address_pubkey = Public::random();
        let address = public_to_address(&address_pubkey);

        let mut state = metadata_for_election();
        state.add_balance(&address, 1000).unwrap();

        init_stake(&mut state, Default::default(), Default::default(), Default::default()).unwrap();

        // TODO: change with stake::execute()
        let deposit = 200;
        let nominate_expire = 5;
        let custody_until = 10;
        let released_at = 20;
        self_nominate(&mut state, &address, &address_pubkey, deposit, 0, nominate_expire, b"".to_vec()).unwrap();
        jail(&mut state, &[address], custody_until, released_at).unwrap();

        for current_term in 0..released_at {
            on_term_close(&mut state, pseudo_term_to_block_num_calculator(current_term), &[]).unwrap();

            let candidates = Candidates::load_from_state(&state).unwrap();
            assert_eq!(candidates.get_candidate(&address), None);

            let jail = Jail::load_from_state(&state).unwrap();
            assert!(jail.get_prisoner(&address).is_some());
        }

        on_term_close(&mut state, pseudo_term_to_block_num_calculator(released_at), &[]).unwrap();

        let candidates = Candidates::load_from_state(&state).unwrap();
        assert_eq!(candidates.get_candidate(&address), None, "A prisoner should not become a candidate");

        let jail = Jail::load_from_state(&state).unwrap();
        assert_eq!(jail.get_prisoner(&address), None, "A prisoner should be released");

        assert_eq!(state.balance(&address).unwrap(), 1000, "Balance should be restored after being released");
    }

    #[test]
    fn cannot_delegate_until_released() {
        let address_pubkey = Public::random();
        let delegator_pubkey = Public::random();
        let address = public_to_address(&address_pubkey);
        let delegator = public_to_address(&delegator_pubkey);

        let mut state = metadata_for_election();
        state.add_balance(&address, 1000).unwrap();

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
        self_nominate(&mut state, &address, &address_pubkey, deposit, 0, nominate_expire, b"".to_vec()).unwrap();
        jail(&mut state, &[address], custody_until, released_at).unwrap();

        for current_term in 0..=released_at {
            let action = StakeAction::DelegateCCS {
                address,
                quantity: 1,
            };
            let result = execute_stake_action(action, &mut state, &delegator, &delegator_pubkey);
            assert_ne!(Ok(()), result);

            on_term_close(&mut state, pseudo_term_to_block_num_calculator(current_term), &[]).unwrap();
        }

        let action = StakeAction::DelegateCCS {
            address,
            quantity: 1,
        };
        let result = execute_stake_action(action, &mut state, &delegator, &delegator_pubkey);
        assert!(result.is_err());
    }

    #[test]
    fn kick_reverts_delegations() {
        let address_pubkey = Public::random();
        let delegator_pubkey = Public::random();
        let address = public_to_address(&address_pubkey);
        let delegator = public_to_address(&delegator_pubkey);

        let mut state = metadata_for_election();
        state.add_balance(&address, 1000).unwrap();

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
        self_nominate(&mut state, &address, &address_pubkey, deposit, 0, nominate_expire, b"".to_vec()).unwrap();

        let action = StakeAction::DelegateCCS {
            address,
            quantity: 40,
        };
        execute_stake_action(action, &mut state, &delegator, &delegator_pubkey).unwrap();

        jail(&mut state, &[address], custody_until, released_at).unwrap();

        for current_term in 0..=released_at {
            on_term_close(&mut state, pseudo_term_to_block_num_calculator(current_term), &[]).unwrap();
        }

        let delegation = Delegation::load_from_state(&state, &delegator).unwrap();
        assert_eq!(delegation.get_quantity(&address), 0, "Delegation should be reverted");

        let account = StakeAccount::load_from_state(&state, &delegator).unwrap();
        assert_eq!(account.balance, 100, "Delegation should be reverted");
    }

    #[test]
    fn self_nomination_before_kick_preserves_delegations() {
        let address_pubkey = Public::random();
        let delegator_pubkey = Public::random();
        let address = public_to_address(&address_pubkey);
        let delegator = public_to_address(&delegator_pubkey);

        let mut state = metadata_for_election();
        state.add_balance(&address, 1000).unwrap();

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
        self_nominate(&mut state, &address, &address_pubkey, 0, 0, nominate_expire, b"".to_vec()).unwrap();

        let action = StakeAction::DelegateCCS {
            address,
            quantity: 40,
        };
        execute_stake_action(action, &mut state, &delegator, &delegator_pubkey).unwrap();

        jail(&mut state, &[address], custody_until, released_at).unwrap();

        for current_term in 0..custody_until {
            on_term_close(&mut state, pseudo_term_to_block_num_calculator(current_term), &[]).unwrap();
        }

        let current_term = custody_until + 1;
        let result = self_nominate(
            &mut state,
            &address,
            &address_pubkey,
            0,
            current_term,
            current_term + nominate_expire,
            b"".to_vec(),
        );
        assert!(result.is_ok());

        let delegation = Delegation::load_from_state(&state, &delegator).unwrap();
        assert_eq!(delegation.get_quantity(&address), 40, "Delegation should be preserved");

        let account = StakeAccount::load_from_state(&state, &delegator).unwrap();
        assert_eq!(account.balance, 100 - 40, "Delegation should be preserved");
    }

    fn pseudo_term_to_block_num_calculator(term_id: u64) -> u64 {
        term_id * 10 + 1
    }
}
