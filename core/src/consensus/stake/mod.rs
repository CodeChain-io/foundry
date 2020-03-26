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

mod action_data;

use crate::client::ConsensusClient;
use ckey::{public_to_address, Address, Ed25519Public as Public};
use cstate::{StakeHandler, StateResult, TopLevelState, TopState, TopStateView};
use ctypes::errors::{RuntimeError, SyntaxError};
use ctypes::transaction::{Approval, StakeAction};
use ctypes::util::unexpected::Mismatch;
use ctypes::CommonParams;
use parking_lot::RwLock;
use primitives::Bytes;
use rlp::{Decodable, Rlp};
use std::collections::HashMap;
use std::sync::{Arc, Weak};

pub use self::action_data::{Banned, Candidates, CurrentValidators, Jail, NextValidators, Validator};
use self::action_data::{Delegation, ReleaseResult, StakeAccount, Stakeholders};
use super::tendermint::Deposit;
use super::ValidatorSet;
use crate::consensus::ConsensusMessage;

pub const CUSTOM_ACTION_HANDLER_ID: u64 = 2;

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

impl StakeHandler for Stake {
    fn execute(
        &self,
        bytes: &[u8],
        state: &mut TopLevelState,
        sender_address: &Address,
        sender_public: &Public,
    ) -> StateResult<()> {
        let action = StakeAction::decode(&Rlp::new(bytes)).expect("Verification passed");

        if let StakeAction::ReportDoubleVote {
            message1,
            ..
        } = &action
        {
            let message1: ConsensusMessage =
                rlp::decode(message1).map_err(|err| RuntimeError::FailedToHandleCustomAction(err.to_string()))?;
            let validators =
                self.validators.read().as_ref().and_then(Weak::upgrade).expect("ValidatorSet must be initialized");
            let client = self.client.read().as_ref().and_then(Weak::upgrade).expect("Client must be initialized");

            execute_report_double_vote(message1, state, sender_address, &*client, &*validators)?;
        }

        match action {
            StakeAction::TransferCCS {
                address,
                quantity,
            } => transfer_ccs(state, sender_address, &address, quantity),
            StakeAction::DelegateCCS {
                address,
                quantity,
            } => delegate_ccs(state, sender_address, &address, quantity),
            StakeAction::Revoke {
                address,
                quantity,
            } => revoke(state, sender_address, &address, quantity),
            StakeAction::Redelegate {
                prev_delegatee,
                next_delegatee,
                quantity,
            } => redelegate(state, sender_address, &prev_delegatee, &next_delegatee, quantity),
            StakeAction::SelfNominate {
                deposit,
                metadata,
            } => {
                let (current_term, nomination_ends_at) = {
                    let metadata = state.metadata()?.expect("Metadata must exist");
                    let current_term = metadata.current_term_id();
                    let expiration = metadata.params().nomination_expiration();
                    let nomination_ends_at = current_term + expiration;
                    (current_term, nomination_ends_at)
                };
                self_nominate(state, sender_address, sender_public, deposit, current_term, nomination_ends_at, metadata)
            }
            StakeAction::ChangeParams {
                metadata_seq,
                params,
                approvals,
            } => change_params(state, metadata_seq, *params, &approvals),
            StakeAction::ReportDoubleVote {
                ..
            } => Ok(()),
        }
    }

    fn verify(&self, bytes: &[u8], current_params: &CommonParams) -> Result<(), SyntaxError> {
        let action =
            StakeAction::decode(&Rlp::new(bytes)).map_err(|err| SyntaxError::InvalidCustomAction(err.to_string()))?;
        action.verify(current_params)?;
        if let StakeAction::ReportDoubleVote {
            message1,
            message2,
        } = action
        {
            let client: Arc<dyn ConsensusClient> =
                self.client.read().as_ref().and_then(Weak::upgrade).expect("Client should be initialized");
            let validators: Arc<dyn ValidatorSet> =
                self.validators.read().as_ref().and_then(Weak::upgrade).expect("ValidatorSet should be initialized");

            let message1: ConsensusMessage =
                rlp::decode(&message1).map_err(|err| SyntaxError::InvalidCustomAction(err.to_string()))?;
            let message2: ConsensusMessage =
                rlp::decode(&message2).map_err(|err| SyntaxError::InvalidCustomAction(err.to_string()))?;

            verify_report_double_vote(message1, message2, &*client, &*validators)?;
        }
        Ok(())
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

fn transfer_ccs(state: &mut TopLevelState, sender: &Address, receiver: &Address, quantity: u64) -> StateResult<()> {
    let mut stakeholders = Stakeholders::load_from_state(state)?;
    let mut sender_account = StakeAccount::load_from_state(state, sender)?;
    let mut receiver_account = StakeAccount::load_from_state(state, receiver)?;
    let sender_delegations = Delegation::load_from_state(state, sender)?;

    sender_account.subtract_balance(quantity)?;
    receiver_account.add_balance(quantity)?;

    stakeholders.update_by_decreased_balance(&sender_account, &sender_delegations);
    stakeholders.update_by_increased_balance(&receiver_account);

    stakeholders.save_to_state(state)?;
    sender_account.save_to_state(state)?;
    receiver_account.save_to_state(state)?;

    ctrace!(ENGINE, "Transferred CCS sender: {}, receiver: {}, quantity: {}", sender, receiver, quantity);
    Ok(())
}

fn delegate_ccs(state: &mut TopLevelState, delegator: &Address, delegatee: &Address, quantity: u64) -> StateResult<()> {
    let candidates = Candidates::load_from_state(state)?;
    if candidates.get_candidate(delegatee).is_none() {
        return Err(RuntimeError::FailedToHandleCustomAction("Can delegate to who is a candidate".into()).into())
    }

    let banned = Banned::load_from_state(state)?;
    let jailed = Jail::load_from_state(state)?;
    assert!(!banned.is_banned(&delegatee), "A candidate must not be banned");
    assert_eq!(None, jailed.get_prisoner(delegatee), "A candidate must not be jailed");

    let mut delegator_account = StakeAccount::load_from_state(state, delegator)?;
    let mut delegation = Delegation::load_from_state(state, &delegator)?;

    delegator_account.subtract_balance(quantity)?;
    delegation.add_quantity(*delegatee, quantity)?;
    // delegation does not touch stakeholders

    delegation.save_to_state(state)?;
    delegator_account.save_to_state(state)?;

    ctrace!(ENGINE, "Delegated CCS. delegator: {}, delegatee: {}, quantity: {}", delegator, delegatee, quantity);
    Ok(())
}

fn revoke(state: &mut TopLevelState, delegator: &Address, delegatee: &Address, quantity: u64) -> StateResult<()> {
    let mut delegator_account = StakeAccount::load_from_state(state, delegator)?;
    let mut delegation = Delegation::load_from_state(state, &delegator)?;

    delegator_account.add_balance(quantity)?;
    delegation.subtract_quantity(*delegatee, quantity)?;
    // delegation does not touch stakeholders

    delegation.save_to_state(state)?;
    delegator_account.save_to_state(state)?;

    ctrace!(ENGINE, "Revoked CCS. delegator: {}, delegatee: {}, quantity: {}", delegator, delegatee, quantity);
    Ok(())
}

fn redelegate(
    state: &mut TopLevelState,
    delegator: &Address,
    prev_delegatee: &Address,
    next_delegatee: &Address,
    quantity: u64,
) -> StateResult<()> {
    let candidates = Candidates::load_from_state(state)?;
    if candidates.get_candidate(next_delegatee).is_none() {
        return Err(RuntimeError::FailedToHandleCustomAction("Can delegate to who is a candidate".into()).into())
    }

    let banned = Banned::load_from_state(state)?;
    let jailed = Jail::load_from_state(state)?;
    assert!(!banned.is_banned(&next_delegatee), "A candidate must not be banned");
    assert_eq!(None, jailed.get_prisoner(next_delegatee), "A candidate must not be jailed");

    let delegator_account = StakeAccount::load_from_state(state, delegator)?;
    let mut delegation = Delegation::load_from_state(state, &delegator)?;

    delegation.subtract_quantity(*prev_delegatee, quantity)?;
    delegation.add_quantity(*next_delegatee, quantity)?;

    delegation.save_to_state(state)?;
    delegator_account.save_to_state(state)?;

    ctrace!(
        ENGINE,
        "Redelegated CCS. delegator: {}, prev_delegatee: {}, next_delegatee: {}, quantity: {}",
        delegator,
        prev_delegatee,
        next_delegatee,
        quantity
    );
    Ok(())
}

fn self_nominate(
    state: &mut TopLevelState,
    nominee: &Address,
    nominee_public: &Public,
    deposit: u64,
    current_term: u64,
    nomination_ends_at: u64,
    metadata: Bytes,
) -> StateResult<()> {
    let blacklist = Banned::load_from_state(state)?;
    if blacklist.is_banned(&nominee) {
        return Err(RuntimeError::FailedToHandleCustomAction("Account is blacklisted".to_string()).into())
    }

    let mut jail = Jail::load_from_state(&state)?;
    let total_deposit = match jail.try_release(nominee, current_term) {
        ReleaseResult::InCustody => {
            return Err(RuntimeError::FailedToHandleCustomAction("Account is still in custody".to_string()).into())
        }
        ReleaseResult::NotExists => deposit,
        ReleaseResult::Released(prisoner) => {
            assert_eq!(&prisoner.address, nominee);
            prisoner.deposit + deposit
        }
    };

    let mut candidates = Candidates::load_from_state(&state)?;
    state.sub_balance(nominee, deposit)?;
    candidates.add_deposit(nominee_public, total_deposit, nomination_ends_at, metadata);

    jail.save_to_state(state)?;
    candidates.save_to_state(state)?;

    ctrace!(
        ENGINE,
        "Self-nominated. nominee: {}, deposit: {}, current_term: {}, ends_at: {}",
        nominee,
        deposit,
        current_term,
        nomination_ends_at
    );
    Ok(())
}

pub fn get_stakes(state: &TopLevelState) -> StateResult<HashMap<Address, u64>> {
    let stakeholders = Stakeholders::load_from_state(state)?;
    let mut result = HashMap::new();
    for stakeholder in stakeholders.iter() {
        let account = StakeAccount::load_from_state(state, stakeholder)?;
        let delegation = Delegation::load_from_state(state, stakeholder)?;
        result.insert(*stakeholder, account.balance + delegation.sum());
    }
    Ok(result)
}

pub fn update_validator_weights(state: &mut TopLevelState, block_author: &Address) -> StateResult<()> {
    let mut validators = NextValidators::load_from_state(state)?;
    validators.update_weight(block_author);
    validators.save_to_state(state)
}

fn change_params(
    state: &mut TopLevelState,
    metadata_seq: u64,
    params: CommonParams,
    approvals: &[Approval],
) -> StateResult<()> {
    // Update state first because the signature validation is more expensive.
    state.update_params(metadata_seq, params)?;

    let stakes = get_stakes(state)?;
    // Approvals are verified
    let signed_stakes = approvals.iter().try_fold(0, |sum, approval| {
        let public = approval.signer_public();
        let address = public_to_address(public);
        stakes.get(&address).map(|stake| sum + stake).ok_or_else(|| RuntimeError::SignatureOfInvalidAccount(address))
    })?;
    let total_stakes: u64 = stakes.values().sum();
    if total_stakes / 2 >= signed_stakes {
        return Err(RuntimeError::InsufficientStakes(Mismatch {
            expected: total_stakes,
            found: signed_stakes,
        })
        .into())
    }

    ctrace!(ENGINE, "ChangeParams. params: {:?}", params);
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

fn update_candidates(
    state: &mut TopLevelState,
    current_term: u64,
    nomination_expiration: u64,
    inactive_validators: &[Address],
) -> StateResult<Vec<Address>> {
    let banned = Banned::load_from_state(state)?;

    let mut candidates = Candidates::load_from_state(state)?;
    let nomination_ends_at = current_term + nomination_expiration;

    let current_validators = NextValidators::load_from_state(state)?;
    candidates.renew_candidates(&current_validators, nomination_ends_at, &inactive_validators, &banned);

    let expired = candidates.drain_expired_candidates(current_term);
    for candidate in &expired {
        let address = public_to_address(&candidate.pubkey);
        state.add_balance(&address, candidate.deposit)?;
        ctrace!(ENGINE, "on_term_close::expired. candidate: {}, deposit: {}", address, candidate.deposit);
    }
    candidates.save_to_state(state)?;
    Ok(expired.into_iter().map(|c| public_to_address(&c.pubkey)).collect())
}

fn release_jailed_prisoners(state: &mut TopLevelState, current_term: u64) -> StateResult<Vec<Address>> {
    let mut jailed = Jail::load_from_state(&state)?;
    let released = jailed.drain_released_prisoners(current_term);
    for prisoner in &released {
        state.add_balance(&prisoner.address, prisoner.deposit)?;
        ctrace!(ENGINE, "on_term_close::released. prisoner: {}, deposit: {}", prisoner.address, prisoner.deposit);
    }
    jailed.save_to_state(state)?;
    Ok(released.into_iter().map(|p| p.address).collect())
}

pub fn jail(state: &mut TopLevelState, addresses: &[Address], custody_until: u64, kick_at: u64) -> StateResult<()> {
    if addresses.is_empty() {
        return Ok(())
    }
    let mut candidates = Candidates::load_from_state(state)?;
    let mut jail = Jail::load_from_state(state)?;

    for address in addresses {
        let candidate = candidates.remove(address).expect("There should be a candidate to jail");
        ctrace!(ENGINE, "on_term_close::jail. candidate: {}, deposit: {}", address, candidate.deposit);
        jail.add(candidate, custody_until, kick_at);
    }

    jail.save_to_state(state)?;
    candidates.save_to_state(state)?;
    Ok(())
}

pub fn ban(state: &mut TopLevelState, informant: &Address, criminal: Address) -> StateResult<()> {
    let mut banned = Banned::load_from_state(state)?;
    if banned.is_banned(&criminal) {
        return Err(RuntimeError::FailedToHandleCustomAction("Account is already banned".to_string()).into())
    }

    let mut candidates = Candidates::load_from_state(state)?;
    let mut jailed = Jail::load_from_state(state)?;
    let mut validators = NextValidators::load_from_state(state)?;

    let deposit = match (candidates.remove(&criminal), jailed.remove(&criminal)) {
        (Some(_), Some(_)) => unreachable!("A candidate that are jailed cannot exist"),
        (Some(candidate), _) => candidate.deposit,
        (_, Some(jailed)) => jailed.deposit,
        _ => 0,
    };
    // confiscate criminal's deposit and give the same deposit amount to the informant.
    state.add_balance(informant, deposit)?;

    jailed.remove(&criminal);
    banned.add(criminal);
    validators.remove(&criminal);

    jailed.save_to_state(state)?;
    banned.save_to_state(state)?;
    candidates.save_to_state(state)?;
    validators.save_to_state(state)?;

    // Revert delegations
    revert_delegations(state, &[criminal])?;

    Ok(())
}

fn revert_delegations(state: &mut TopLevelState, reverted_delegatees: &[Address]) -> StateResult<()> {
    // Stakeholders list isn't changed while reverting.

    let stakeholders = Stakeholders::load_from_state(state)?;
    for stakeholder in stakeholders.iter() {
        let mut delegator = StakeAccount::load_from_state(state, stakeholder)?;
        let mut delegation = Delegation::load_from_state(state, stakeholder)?;

        for delegatee in reverted_delegatees {
            let quantity = delegation.get_quantity(delegatee);
            if quantity > 0 {
                delegation.subtract_quantity(*delegatee, quantity)?;
                delegator.add_balance(quantity)?;
                ctrace!(
                    ENGINE,
                    "revert_delegation delegator: {}, delegatee: {}, quantity: {}",
                    stakeholder,
                    delegatee,
                    quantity
                );
            }
        }
        delegation.save_to_state(state)?;
        delegator.save_to_state(state)?;
    }
    Ok(())
}

pub(super) fn init(
    state: &mut TopLevelState,
    genesis_stakes: HashMap<Address, u64>,
    genesis_candidates: HashMap<Address, Deposit>,
    genesis_delegations: HashMap<Address, HashMap<Address, u64>>,
) -> StateResult<()> {
    let mut genesis_stakes = genesis_stakes;
    for (delegator, delegation) in &genesis_delegations {
        let stake = genesis_stakes.entry(*delegator).or_default();
        let total_delegation = delegation.values().sum();
        if *stake < total_delegation {
            cerror!(STATE, "{} has insufficient stakes to delegate", delegator);
            return Err(RuntimeError::InsufficientStakes(Mismatch {
                expected: total_delegation,
                found: *stake,
            })
            .into())
        }
        for delegatee in delegation.keys() {
            if !genesis_candidates.contains_key(delegatee) {
                return Err(RuntimeError::FailedToHandleCustomAction("Can delegate to who is a candidate".into()).into())
            }
        }
        *stake -= total_delegation;
    }

    let mut stakeholders = Stakeholders::load_from_state(state)?;
    for (address, amount) in &genesis_stakes {
        let account = StakeAccount {
            address,
            balance: *amount,
        };
        account.save_to_state(state)?;
        stakeholders.update_by_increased_balance(&account);
    }
    stakeholders.save_to_state(state)?;

    for (address, deposit) in &genesis_candidates {
        let balance = state.balance(address).unwrap_or_default();
        if balance < deposit.deposit {
            cerror!(STATE, "{} has insufficient balance to become the candidate", address);
            return Err(RuntimeError::InsufficientBalance {
                address: *address,
                balance,
                cost: deposit.deposit,
            }
            .into())
        }
        state.sub_balance(address, deposit.deposit).unwrap();
    }

    let mut candidates = Candidates::default();
    {
        let mut values: Vec<_> = genesis_candidates.values().collect();
        values.sort_unstable(); // The insertion order of candidates is important.

        for candidate in values {
            candidates.add_deposit(
                &candidate.pubkey,
                candidate.deposit,
                candidate.nomination_ends_at,
                candidate.metadata.clone(),
            );
        }
    }
    candidates.save_to_state(state)?;

    for (delegator, delegations) in &genesis_delegations {
        let mut delegation = Delegation::load_from_state(state, &delegator)?;
        for (delegatee, amount) in delegations {
            delegation.add_quantity(*delegatee, *amount)?;
        }
        delegation.save_to_state(state)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::action_data::get_account_key;
    use super::*;

    use crate::consensus::stake::action_data::{get_delegation_key, Candidate, Prisoner};
    use cstate::tests::helpers;
    use cstate::TopStateView;
    use rlp::Encodable;

    fn metadata_for_election() -> TopLevelState {
        let mut params = CommonParams::default_for_test();
        let mut state = helpers::get_temp_state_with_metadata(params);
        state.metadata().unwrap().unwrap().set_params(CommonParams::default_for_test());
        params.set_dynamic_validator_params_for_test(30, 10, 3, 20, 30, 4, 1000, 10000, 100);
        assert_eq!(Ok(()), state.update_params(0, params));
        state
    }

    #[test]
    fn genesis_stakes() {
        let address1 = Address::random();
        let address2 = Address::random();

        let mut state = helpers::get_temp_state();
        let genesis_stakes = {
            let mut genesis_stakes = HashMap::new();
            genesis_stakes.insert(address1, 100);
            genesis_stakes
        };
        super::init(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();

        let account1 = StakeAccount::load_from_state(&state, &address1).unwrap();
        assert_eq!(account1.balance, 100);

        let account2 = StakeAccount::load_from_state(&state, &address2).unwrap();
        assert_eq!(account2.balance, 0);

        let stakeholders = Stakeholders::load_from_state(&state).unwrap();
        assert_eq!(stakeholders.iter().len(), 1);
        assert!(stakeholders.contains(&address1));
        assert!(!stakeholders.contains(&address2));
    }

    #[test]
    fn balance_transfer_partial() {
        let address1 = Address::random();
        let address2 = Address::random();

        let mut state = helpers::get_temp_state();
        let genesis_stakes = {
            let mut genesis_stakes = HashMap::new();
            genesis_stakes.insert(address1, 100);
            genesis_stakes
        };
        super::init(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();

        let result = transfer_ccs(&mut state, &address1, &address2, 10);
        assert_eq!(result, Ok(()));

        let account1 = StakeAccount::load_from_state(&state, &address1).unwrap();
        assert_eq!(account1.balance, 90);

        let account2 = StakeAccount::load_from_state(&state, &address2).unwrap();
        assert_eq!(account2.balance, 10);

        let stakeholders = Stakeholders::load_from_state(&state).unwrap();
        assert_eq!(stakeholders.iter().len(), 2);
        assert!(stakeholders.contains(&address1));
        assert!(stakeholders.contains(&address2));
    }

    #[test]
    fn balance_transfer_all() {
        let address1 = Address::random();
        let address2 = Address::random();

        let mut state = helpers::get_temp_state();
        let genesis_stakes = {
            let mut genesis_stakes = HashMap::new();
            genesis_stakes.insert(address1, 100);
            genesis_stakes
        };
        super::init(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();

        let result = transfer_ccs(&mut state, &address1, &address2, 100);
        assert_eq!(result, Ok(()));

        let account1 = StakeAccount::load_from_state(&state, &address1).unwrap();
        assert_eq!(account1.balance, 0);
        assert_eq!(state.action_data(&get_account_key(&address1)).unwrap(), None, "Should clear state");

        let account2 = StakeAccount::load_from_state(&state, &address2).unwrap();
        assert_eq!(account2.balance, 100);

        let stakeholders = Stakeholders::load_from_state(&state).unwrap();
        assert_eq!(stakeholders.iter().len(), 1);
        assert!(!stakeholders.contains(&address1), "Not be a stakeholder anymore");
        assert!(stakeholders.contains(&address2));
    }

    #[test]
    fn delegate() {
        let delegatee_pubkey = Public::random();
        let delegator_pubkey = Public::random();
        let delegatee = public_to_address(&delegatee_pubkey);
        let delegator = public_to_address(&delegator_pubkey);

        let mut state = helpers::get_temp_state();
        let genesis_stakes = {
            let mut genesis_stakes = HashMap::new();
            genesis_stakes.insert(delegatee, 100);
            genesis_stakes.insert(delegator, 100);
            genesis_stakes
        };
        super::init(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();
        self_nominate(&mut state, &delegatee, &delegatee_pubkey, 0, 0, 10, b"".to_vec()).unwrap();

        let action = StakeAction::DelegateCCS {
            address: delegatee,
            quantity: 40,
        };
        let result = Stake::default().execute(&action.rlp_bytes(), &mut state, &delegator, &delegator_pubkey);
        assert_eq!(result, Ok(()));

        let delegator_account = StakeAccount::load_from_state(&state, &delegator).unwrap();
        assert_eq!(delegator_account.balance, 60);

        let delegatee_account = StakeAccount::load_from_state(&state, &delegatee).unwrap();
        assert_eq!(delegatee_account.balance, 100, "Shouldn't be touched");

        let delegation = Delegation::load_from_state(&state, &delegator).unwrap();
        assert_eq!(delegation.iter().count(), 1);
        assert_eq!(delegation.get_quantity(&delegatee), 40);

        let delegation_delegatee = Delegation::load_from_state(&state, &delegatee).unwrap();
        assert_eq!(delegation_delegatee.iter().count(), 0, "Shouldn't be touched");

        let stakeholders = Stakeholders::load_from_state(&state).unwrap();
        assert_eq!(stakeholders.iter().len(), 2);
        assert!(stakeholders.contains(&delegator));
        assert!(stakeholders.contains(&delegatee));
    }

    #[test]
    fn delegate_all() {
        let delegatee_pubkey = Public::random();
        let delegatee = public_to_address(&delegatee_pubkey);
        let delegator_pubkey = Public::random();
        let delegator = public_to_address(&delegator_pubkey);

        let mut state = helpers::get_temp_state();
        let genesis_stakes = {
            let mut genesis_stakes = HashMap::new();
            genesis_stakes.insert(delegatee, 100);
            genesis_stakes.insert(delegator, 100);
            genesis_stakes
        };
        super::init(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();
        self_nominate(&mut state, &delegatee, &delegatee_pubkey, 0, 0, 10, b"".to_vec()).unwrap();

        let action = StakeAction::DelegateCCS {
            address: delegatee,
            quantity: 100,
        };
        let result = Stake::default().execute(&action.rlp_bytes(), &mut state, &delegator, &delegator_pubkey);
        assert_eq!(result, Ok(()));

        let delegator_account = StakeAccount::load_from_state(&state, &delegator).unwrap();
        assert_eq!(delegator_account.balance, 0);
        assert_eq!(state.action_data(&get_account_key(&delegator)).unwrap(), None, "Should clear state");

        let delegatee_account = StakeAccount::load_from_state(&state, &delegatee).unwrap();
        assert_eq!(delegatee_account.balance, 100, "Shouldn't be touched");

        let delegation = Delegation::load_from_state(&state, &delegator).unwrap();
        assert_eq!(delegation.iter().count(), 1);
        assert_eq!(delegation.get_quantity(&delegatee), 100);

        let delegation_delegatee = Delegation::load_from_state(&state, &delegatee).unwrap();
        assert_eq!(delegation_delegatee.iter().count(), 0, "Shouldn't be touched");

        let stakeholders = Stakeholders::load_from_state(&state).unwrap();
        assert_eq!(stakeholders.iter().len(), 2);
        assert!(stakeholders.contains(&delegator), "Should still be a stakeholder after delegated all");
        assert!(stakeholders.contains(&delegatee));
    }

    #[test]
    fn delegate_only_to_candidate() {
        let delegatee_pubkey = Public::random();
        let delegatee = public_to_address(&delegatee_pubkey);
        let delegator_pubkey = Public::random();
        let delegator = public_to_address(&delegator_pubkey);

        let mut state = helpers::get_temp_state();
        let genesis_stakes = {
            let mut genesis_stakes = HashMap::new();
            genesis_stakes.insert(delegatee, 100);
            genesis_stakes.insert(delegator, 100);
            genesis_stakes
        };
        super::init(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();

        let action = StakeAction::DelegateCCS {
            address: delegatee,
            quantity: 40,
        };
        let result = Stake::default().execute(&action.rlp_bytes(), &mut state, &delegator, &delegator_pubkey);
        assert!(result.is_err());
    }

    #[test]
    fn delegate_too_much() {
        let delegatee_pubkey = Public::random();
        let delegatee = public_to_address(&delegatee_pubkey);
        let delegator_pubkey = Public::random();
        let delegator = public_to_address(&delegator_pubkey);

        let mut state = helpers::get_temp_state();
        let genesis_stakes = {
            let mut genesis_stakes = HashMap::new();
            genesis_stakes.insert(delegatee, 100);
            genesis_stakes.insert(delegator, 100);
            genesis_stakes
        };
        super::init(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();
        self_nominate(&mut state, &delegatee, &delegatee_pubkey, 0, 0, 10, b"".to_vec()).unwrap();

        let action = StakeAction::DelegateCCS {
            address: delegatee,
            quantity: 200,
        };
        let result = Stake::default().execute(&action.rlp_bytes(), &mut state, &delegator, &delegator_pubkey);
        assert!(result.is_err());
    }

    #[test]
    fn can_transfer_within_non_delegated_tokens() {
        let delegatee_pubkey = Public::random();
        let delegatee = public_to_address(&delegatee_pubkey);
        let delegator_pubkey = Public::random();
        let delegator = public_to_address(&delegator_pubkey);

        let mut state = helpers::get_temp_state();
        let genesis_stakes = {
            let mut genesis_stakes = HashMap::new();
            genesis_stakes.insert(delegatee, 100);
            genesis_stakes.insert(delegator, 100);
            genesis_stakes
        };
        super::init(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();
        self_nominate(&mut state, &delegatee, &delegatee_pubkey, 0, 0, 10, b"".to_vec()).unwrap();

        let action = StakeAction::DelegateCCS {
            address: delegatee,
            quantity: 50,
        };
        Stake::default().execute(&action.rlp_bytes(), &mut state, &delegator, &delegator_pubkey).unwrap();

        let action = StakeAction::TransferCCS {
            address: delegatee,
            quantity: 50,
        };
        let result = Stake::default().execute(&action.rlp_bytes(), &mut state, &delegator, &delegator_pubkey);
        assert!(result.is_ok());
    }

    #[test]
    fn cannot_transfer_over_non_delegated_tokens() {
        let delegatee_pubkey = Public::random();
        let delegatee = public_to_address(&delegatee_pubkey);
        let delegator_pubkey = Public::random();
        let delegator = public_to_address(&delegator_pubkey);

        let mut state = helpers::get_temp_state();
        let genesis_stakes = {
            let mut genesis_stakes = HashMap::new();
            genesis_stakes.insert(delegatee, 100);
            genesis_stakes.insert(delegator, 100);
            genesis_stakes
        };
        super::init(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();
        self_nominate(&mut state, &delegatee, &delegatee_pubkey, 0, 0, 10, b"".to_vec()).unwrap();

        let action = StakeAction::DelegateCCS {
            address: delegatee,
            quantity: 50,
        };
        Stake::default().execute(&action.rlp_bytes(), &mut state, &delegator, &delegator_pubkey).unwrap();

        let action = StakeAction::TransferCCS {
            address: delegatee,
            quantity: 100,
        };
        let result = Stake::default().execute(&action.rlp_bytes(), &mut state, &delegator, &delegator_pubkey);
        assert!(result.is_err());
    }

    #[test]
    fn can_revoke_delegated_tokens() {
        let delegatee_pubkey = Public::random();
        let delegatee = public_to_address(&delegatee_pubkey);
        let delegator_pubkey = Public::random();
        let delegator = public_to_address(&delegator_pubkey);

        let mut state = helpers::get_temp_state();
        let genesis_stakes = {
            let mut genesis_stakes = HashMap::new();
            genesis_stakes.insert(delegatee, 100);
            genesis_stakes.insert(delegator, 100);
            genesis_stakes
        };
        super::init(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();
        self_nominate(&mut state, &delegatee, &delegatee_pubkey, 0, 0, 10, b"".to_vec()).unwrap();

        let action = StakeAction::DelegateCCS {
            address: delegatee,
            quantity: 50,
        };
        let result = Stake::default().execute(&action.rlp_bytes(), &mut state, &delegator, &delegator_pubkey);
        assert!(result.is_ok());

        let action = StakeAction::Revoke {
            address: delegatee,
            quantity: 20,
        };
        let result = Stake::default().execute(&action.rlp_bytes(), &mut state, &delegator, &delegator_pubkey);
        assert_eq!(Ok(()), result);

        let delegator_account = StakeAccount::load_from_state(&state, &delegator).unwrap();
        let delegation = Delegation::load_from_state(&state, &delegator).unwrap();
        assert_eq!(delegator_account.balance, 100 - 50 + 20);
        assert_eq!(delegation.iter().count(), 1);
        assert_eq!(delegation.get_quantity(&delegatee), 50 - 20);
    }

    #[test]
    fn cannot_revoke_more_than_delegated_tokens() {
        let delegatee_pubkey = Public::random();
        let delegatee = public_to_address(&delegatee_pubkey);
        let delegator_pubkey = Public::random();
        let delegator = public_to_address(&delegator_pubkey);

        let mut state = helpers::get_temp_state();
        let genesis_stakes = {
            let mut genesis_stakes = HashMap::new();
            genesis_stakes.insert(delegatee, 100);
            genesis_stakes.insert(delegator, 100);
            genesis_stakes
        };
        super::init(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();
        self_nominate(&mut state, &delegatee, &delegatee_pubkey, 0, 0, 10, b"".to_vec()).unwrap();

        let action = StakeAction::DelegateCCS {
            address: delegatee,
            quantity: 50,
        };
        let result = Stake::default().execute(&action.rlp_bytes(), &mut state, &delegator, &delegator_pubkey);
        assert!(result.is_ok());

        let action = StakeAction::Revoke {
            address: delegatee,
            quantity: 70,
        };
        let result = Stake::default().execute(&action.rlp_bytes(), &mut state, &delegator, &delegator_pubkey);
        assert!(result.is_err());

        let delegator_account = StakeAccount::load_from_state(&state, &delegator).unwrap();
        let delegation = Delegation::load_from_state(&state, &delegator).unwrap();
        assert_eq!(delegator_account.balance, 100 - 50);
        assert_eq!(delegation.iter().count(), 1);
        assert_eq!(delegation.get_quantity(&delegatee), 50);
    }

    #[test]
    fn revoke_all_should_clear_state() {
        let delegatee_pubkey = Public::random();
        let delegatee = public_to_address(&delegatee_pubkey);
        let delegator_pubkey = Public::random();
        let delegator = public_to_address(&delegator_pubkey);

        let mut state = helpers::get_temp_state();
        let genesis_stakes = {
            let mut genesis_stakes = HashMap::new();
            genesis_stakes.insert(delegatee, 100);
            genesis_stakes.insert(delegator, 100);
            genesis_stakes
        };
        super::init(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();
        self_nominate(&mut state, &delegatee, &delegatee_pubkey, 0, 0, 10, b"".to_vec()).unwrap();

        let action = StakeAction::DelegateCCS {
            address: delegatee,
            quantity: 50,
        };
        let result = Stake::default().execute(&action.rlp_bytes(), &mut state, &delegator, &delegator_pubkey);
        assert!(result.is_ok());

        let action = StakeAction::Revoke {
            address: delegatee,
            quantity: 50,
        };
        let result = Stake::default().execute(&action.rlp_bytes(), &mut state, &delegator, &delegator_pubkey);
        assert_eq!(Ok(()), result);

        let delegator_account = StakeAccount::load_from_state(&state, &delegator).unwrap();
        assert_eq!(delegator_account.balance, 100);
        assert_eq!(state.action_data(&get_delegation_key(&delegator)).unwrap(), None);
    }

    #[test]
    fn can_redelegate_tokens() {
        let prev_delegatee_pubkey = Public::random();
        let prev_delegatee = public_to_address(&prev_delegatee_pubkey);
        let next_delegatee_pubkey = Public::random();
        let next_delegatee = public_to_address(&next_delegatee_pubkey);
        let delegator_pubkey = Public::random();
        let delegator = public_to_address(&delegator_pubkey);

        let mut state = helpers::get_temp_state();
        let genesis_stakes = {
            let mut genesis_stakes = HashMap::new();
            genesis_stakes.insert(delegator, 100);
            genesis_stakes
        };
        super::init(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();
        self_nominate(&mut state, &prev_delegatee, &prev_delegatee_pubkey, 0, 0, 10, b"".to_vec()).unwrap();
        self_nominate(&mut state, &next_delegatee, &next_delegatee_pubkey, 0, 0, 10, b"".to_vec()).unwrap();

        let action = StakeAction::DelegateCCS {
            address: prev_delegatee,
            quantity: 50,
        };
        let result = Stake::default().execute(&action.rlp_bytes(), &mut state, &delegator, &delegator_pubkey);
        assert!(result.is_ok());

        let action = StakeAction::Redelegate {
            prev_delegatee,
            next_delegatee,
            quantity: 20,
        };
        let result = Stake::default().execute(&action.rlp_bytes(), &mut state, &delegator, &delegator_pubkey);
        assert_eq!(Ok(()), result);

        let delegator_account = StakeAccount::load_from_state(&state, &delegator).unwrap();
        let delegation = Delegation::load_from_state(&state, &delegator).unwrap();
        assert_eq!(delegator_account.balance, 100 - 50);
        assert_eq!(delegation.iter().count(), 2);
        assert_eq!(delegation.get_quantity(&prev_delegatee), 50 - 20);
        assert_eq!(delegation.get_quantity(&next_delegatee), 20);
    }

    #[test]
    fn cannot_redelegate_more_than_delegated_tokens() {
        let prev_delegatee_pubkey = Public::random();
        let prev_delegatee = public_to_address(&prev_delegatee_pubkey);
        let next_delegatee_pubkey = Public::random();
        let next_delegatee = public_to_address(&next_delegatee_pubkey);
        let delegator_pubkey = Public::random();
        let delegator = public_to_address(&delegator_pubkey);

        let mut state = helpers::get_temp_state();
        let genesis_stakes = {
            let mut genesis_stakes = HashMap::new();
            genesis_stakes.insert(delegator, 100);
            genesis_stakes
        };
        super::init(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();
        self_nominate(&mut state, &prev_delegatee, &prev_delegatee_pubkey, 0, 0, 10, b"".to_vec()).unwrap();
        self_nominate(&mut state, &next_delegatee, &next_delegatee_pubkey, 0, 0, 10, b"".to_vec()).unwrap();

        let action = StakeAction::DelegateCCS {
            address: prev_delegatee,
            quantity: 50,
        };
        let result = Stake::default().execute(&action.rlp_bytes(), &mut state, &delegator, &delegator_pubkey);
        assert!(result.is_ok());

        let action = StakeAction::Redelegate {
            prev_delegatee,
            next_delegatee,
            quantity: 70,
        };
        let result = Stake::default().execute(&action.rlp_bytes(), &mut state, &delegator, &delegator_pubkey);
        assert!(result.is_err());

        let delegator_account = StakeAccount::load_from_state(&state, &delegator).unwrap();
        let delegation = Delegation::load_from_state(&state, &delegator).unwrap();
        assert_eq!(delegator_account.balance, 100 - 50);
        assert_eq!(delegation.iter().count(), 1);
        assert_eq!(delegation.get_quantity(&prev_delegatee), 50);
        assert_eq!(delegation.get_quantity(&next_delegatee), 0);
    }

    #[test]
    fn redelegate_all_should_clear_state() {
        let prev_delegatee_pubkey = Public::random();
        let prev_delegatee = public_to_address(&prev_delegatee_pubkey);
        let next_delegatee_pubkey = Public::random();
        let next_delegatee = public_to_address(&next_delegatee_pubkey);
        let delegator_pubkey = Public::random();
        let delegator = public_to_address(&delegator_pubkey);

        let mut state = helpers::get_temp_state();
        let genesis_stakes = {
            let mut genesis_stakes = HashMap::new();
            genesis_stakes.insert(delegator, 100);
            genesis_stakes
        };
        super::init(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();
        self_nominate(&mut state, &prev_delegatee, &prev_delegatee_pubkey, 0, 0, 10, b"".to_vec()).unwrap();
        self_nominate(&mut state, &next_delegatee, &next_delegatee_pubkey, 0, 0, 10, b"".to_vec()).unwrap();

        let action = StakeAction::DelegateCCS {
            address: prev_delegatee,
            quantity: 50,
        };
        let result = Stake::default().execute(&action.rlp_bytes(), &mut state, &delegator, &delegator_pubkey);
        assert!(result.is_ok());

        let action = StakeAction::Redelegate {
            prev_delegatee,
            next_delegatee,
            quantity: 50,
        };
        let result = Stake::default().execute(&action.rlp_bytes(), &mut state, &delegator, &delegator_pubkey);
        assert_eq!(Ok(()), result);

        let delegator_account = StakeAccount::load_from_state(&state, &delegator).unwrap();
        let delegation = Delegation::load_from_state(&state, &delegator).unwrap();
        assert_eq!(delegator_account.balance, 50);
        assert_eq!(delegation.iter().count(), 1);
        assert_eq!(delegation.get_quantity(&prev_delegatee), 0);
        assert_eq!(delegation.get_quantity(&next_delegatee), 50);
    }

    #[test]
    fn redelegate_only_to_candidate() {
        let prev_delegatee_pubkey = Public::random();
        let prev_delegatee = public_to_address(&prev_delegatee_pubkey);
        let next_delegatee_pubkey = Public::random();
        let next_delegatee = public_to_address(&next_delegatee_pubkey);
        let delegator_pubkey = Public::random();
        let delegator = public_to_address(&delegator_pubkey);

        let mut state = helpers::get_temp_state();
        let genesis_stakes = {
            let mut genesis_stakes = HashMap::new();
            genesis_stakes.insert(delegator, 100);
            genesis_stakes
        };
        super::init(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();

        self_nominate(&mut state, &prev_delegatee, &prev_delegatee_pubkey, 0, 0, 10, b"".to_vec()).unwrap();

        let action = StakeAction::DelegateCCS {
            address: prev_delegatee,
            quantity: 40,
        };
        let result = Stake::default().execute(&action.rlp_bytes(), &mut state, &delegator, &delegator_pubkey);
        assert!(result.is_ok());

        let action = StakeAction::Redelegate {
            prev_delegatee,
            next_delegatee,
            quantity: 50,
        };
        let result = Stake::default().execute(&action.rlp_bytes(), &mut state, &delegator, &delegator_pubkey);
        assert!(result.is_err());
    }

    #[test]
    fn cannot_redelegate_to_banned_account() {
        let informant_pubkey = Public::random();
        let criminal_pubkey = Public::random();
        let delegator_pubkey = Public::random();
        let informant = public_to_address(&informant_pubkey);
        let criminal = public_to_address(&criminal_pubkey);
        let delegator = public_to_address(&delegator_pubkey);
        let prev_delegatee_pubkey = Public::random();
        let prev_delegatee = public_to_address(&prev_delegatee_pubkey);

        let mut state = helpers::get_temp_state();
        state.add_balance(&criminal, 1000).unwrap();

        let genesis_stakes = {
            let mut genesis_stakes = HashMap::new();
            genesis_stakes.insert(delegator, 100);
            genesis_stakes
        };
        super::init(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();
        self_nominate(&mut state, &prev_delegatee, &prev_delegatee_pubkey, 0, 0, 10, b"".to_vec()).unwrap();
        self_nominate(&mut state, &criminal, &criminal_pubkey, 100, 0, 10, b"".to_vec()).unwrap();

        let action = StakeAction::DelegateCCS {
            address: criminal,
            quantity: 40,
        };
        Stake::default().execute(&action.rlp_bytes(), &mut state, &delegator, &delegator_pubkey).unwrap();
        let action = StakeAction::DelegateCCS {
            address: prev_delegatee,
            quantity: 40,
        };
        Stake::default().execute(&action.rlp_bytes(), &mut state, &delegator, &delegator_pubkey).unwrap();

        let candidates = Candidates::load_from_state(&state).unwrap();
        assert_eq!(candidates.len(), 2);

        assert_eq!(Ok(()), ban(&mut state, &informant, criminal));

        let banned = Banned::load_from_state(&state).unwrap();
        assert!(banned.is_banned(&criminal));

        let candidates = Candidates::load_from_state(&state).unwrap();
        assert_eq!(candidates.len(), 1);

        let action = StakeAction::Redelegate {
            prev_delegatee,
            next_delegatee: criminal,
            quantity: 40,
        };
        let result = Stake::default().execute(&action.rlp_bytes(), &mut state, &delegator, &delegator_pubkey);
        assert!(result.is_err());
    }

    #[test]
    fn cannot_redelegate_to_jailed_account() {
        let jail_pubkey = Public::random();
        let jail_address = public_to_address(&jail_pubkey);
        let prev_delegatee_pubkey = Public::random();
        let prev_delegatee = public_to_address(&prev_delegatee_pubkey);
        let delegator_pubkey = Public::random();
        let delegator = public_to_address(&delegator_pubkey);

        let mut state = helpers::get_temp_state();
        state.add_balance(&jail_address, 1000).unwrap();

        let genesis_stakes = {
            let mut genesis_stakes = HashMap::new();
            genesis_stakes.insert(delegator, 100);
            genesis_stakes
        };
        super::init(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();
        self_nominate(&mut state, &prev_delegatee, &prev_delegatee_pubkey, 0, 0, 10, b"".to_vec()).unwrap();

        let deposit = 200;
        self_nominate(&mut state, &jail_address, &jail_pubkey, deposit, 0, 5, b"".to_vec()).unwrap();

        let action = StakeAction::DelegateCCS {
            address: prev_delegatee,
            quantity: 40,
        };
        Stake::default().execute(&action.rlp_bytes(), &mut state, &delegator, &delegator_pubkey).unwrap();

        let candidates = Candidates::load_from_state(&state).unwrap();
        assert_eq!(candidates.len(), 2);

        let custody_until = 10;
        let released_at = 20;
        let result = jail(&mut state, &[jail_address], custody_until, released_at);
        assert!(result.is_ok());

        let candidates = Candidates::load_from_state(&state).unwrap();
        assert_eq!(candidates.len(), 1);

        let action = StakeAction::Redelegate {
            prev_delegatee,
            next_delegatee: jail_address,
            quantity: 40,
        };
        let result = Stake::default().execute(&action.rlp_bytes(), &mut state, &delegator, &delegator_pubkey);
        assert!(result.is_err());
    }

    #[test]
    fn self_nominate_deposit_test() {
        let address_pubkey = Public::random();
        let address = public_to_address(&address_pubkey);

        let mut state = helpers::get_temp_state();
        state.add_balance(&address, 1000).unwrap();

        super::init(&mut state, Default::default(), Default::default(), Default::default()).unwrap();

        // TODO: change with stake::execute()
        let result = self_nominate(&mut state, &address, &address_pubkey, 0, 0, 5, b"metadata1".to_vec());
        assert_eq!(result, Ok(()));

        assert_eq!(state.balance(&address).unwrap(), 1000);
        let candidates = Candidates::load_from_state(&state).unwrap();
        assert_eq!(
            candidates.get_candidate(&address),
            Some(&Candidate {
                pubkey: address_pubkey,
                deposit: 0,
                nomination_ends_at: 5,
                metadata: b"metadata1".to_vec(),
            }),
            "nomination_ends_at should be updated even if candidate deposits 0"
        );

        let result = self_nominate(&mut state, &address, &address_pubkey, 200, 0, 10, b"metadata2".to_vec());
        assert_eq!(result, Ok(()));

        assert_eq!(state.balance(&address).unwrap(), 800);
        let candidates = Candidates::load_from_state(&state).unwrap();
        assert_eq!(
            candidates.get_candidate(&address),
            Some(&Candidate {
                pubkey: address_pubkey,
                deposit: 200,
                nomination_ends_at: 10,
                metadata: b"metadata2".to_vec(),
            })
        );

        let result = self_nominate(&mut state, &address, &address_pubkey, 0, 0, 15, b"metadata3".to_vec());
        assert_eq!(result, Ok(()));

        assert_eq!(state.balance(&address).unwrap(), 800);
        let candidates = Candidates::load_from_state(&state).unwrap();
        assert_eq!(
            candidates.get_candidate(&address),
            Some(&Candidate {
                pubkey: address_pubkey,
                deposit: 200,
                nomination_ends_at: 15,
                metadata: b"metadata3".to_vec(),
            }),
            "nomination_ends_at should be updated even if candidate deposits 0"
        );
    }

    #[test]
    fn self_nominate_fail_with_insufficient_balance() {
        let address_pubkey = Public::random();
        let address = public_to_address(&address_pubkey);

        let mut state = helpers::get_temp_state();
        state.add_balance(&address, 1000).unwrap();

        super::init(&mut state, Default::default(), Default::default(), Default::default()).unwrap();

        // TODO: change with stake::execute()
        let result = self_nominate(&mut state, &address, &address_pubkey, 2000, 0, 5, b"".to_vec());
        assert!(result.is_err(), "Cannot self-nominate without a sufficient balance");
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

        super::init(&mut state, Default::default(), Default::default(), Default::default()).unwrap();

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
        super::init(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();

        // TODO: change with stake::execute()
        self_nominate(&mut state, &address, &address_pubkey, 0, 0, 30, b"".to_vec()).unwrap();

        let action = StakeAction::DelegateCCS {
            address,
            quantity: 40,
        };
        Stake::default().execute(&action.rlp_bytes(), &mut state, &delegator, &delegator_pubkey).unwrap();

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
    fn jail_candidate() {
        let address_pubkey = Public::random();
        let address = public_to_address(&address_pubkey);

        let mut state = helpers::get_temp_state();
        state.add_balance(&address, 1000).unwrap();

        super::init(&mut state, Default::default(), Default::default(), Default::default()).unwrap();

        // TODO: change with stake::execute()
        let deposit = 200;
        self_nominate(&mut state, &address, &address_pubkey, deposit, 0, 5, b"".to_vec()).unwrap();

        let custody_until = 10;
        let released_at = 20;
        let result = jail(&mut state, &[address], custody_until, released_at);
        assert!(result.is_ok());

        let candidates = Candidates::load_from_state(&state).unwrap();
        assert_eq!(candidates.get_candidate(&address), None, "The candidate is removed");

        let jail = Jail::load_from_state(&state).unwrap();
        assert_eq!(
            jail.get_prisoner(&address),
            Some(&Prisoner {
                address,
                deposit,
                custody_until,
                released_at,
            }),
            "The candidate become a prisoner"
        );

        assert_eq!(state.balance(&address).unwrap(), 1000 - deposit, "Deposited ccs is temporarily unavailable");
    }

    #[test]
    fn cannot_self_nominate_while_custody() {
        let address_pubkey = Public::random();
        let address = public_to_address(&address_pubkey);

        let mut state = metadata_for_election();
        state.add_balance(&address, 1000).unwrap();

        super::init(&mut state, Default::default(), Default::default(), Default::default()).unwrap();

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

        super::init(&mut state, Default::default(), Default::default(), Default::default()).unwrap();

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

        super::init(&mut state, Default::default(), Default::default(), Default::default()).unwrap();

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
        super::init(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();

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
            let result = Stake::default().execute(&action.rlp_bytes(), &mut state, &delegator, &delegator_pubkey);
            assert_ne!(Ok(()), result);

            on_term_close(&mut state, pseudo_term_to_block_num_calculator(current_term), &[]).unwrap();
        }

        let action = StakeAction::DelegateCCS {
            address,
            quantity: 1,
        };
        let result = Stake::default().execute(&action.rlp_bytes(), &mut state, &delegator, &delegator_pubkey);
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
        super::init(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();

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
        Stake::default().execute(&action.rlp_bytes(), &mut state, &delegator, &delegator_pubkey).unwrap();

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
        super::init(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();

        // TODO: change with stake::execute()
        let nominate_expire = 5;
        let custody_until = 10;
        let released_at = 20;
        self_nominate(&mut state, &address, &address_pubkey, 0, 0, nominate_expire, b"".to_vec()).unwrap();

        let action = StakeAction::DelegateCCS {
            address,
            quantity: 40,
        };
        Stake::default().execute(&action.rlp_bytes(), &mut state, &delegator, &delegator_pubkey).unwrap();

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

    #[test]
    fn test_ban() {
        let informant_pubkey = Public::random();
        let criminal_pubkey = Public::random();
        let delegator_pubkey = Public::random();
        let informant = public_to_address(&informant_pubkey);
        let criminal = public_to_address(&criminal_pubkey);
        let delegator = public_to_address(&delegator_pubkey);

        let mut state = helpers::get_temp_state();
        state.add_balance(&criminal, 1000).unwrap();

        let genesis_stakes = {
            let mut genesis_stakes = HashMap::new();
            genesis_stakes.insert(delegator, 100);
            genesis_stakes
        };
        super::init(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();

        let deposit = 100;
        self_nominate(&mut state, &criminal, &criminal_pubkey, deposit, 0, 10, b"".to_vec()).unwrap();
        let action = StakeAction::DelegateCCS {
            address: criminal,
            quantity: 40,
        };
        Stake::default().execute(&action.rlp_bytes(), &mut state, &delegator, &delegator_pubkey).unwrap();

        assert_eq!(Ok(()), ban(&mut state, &informant, criminal));

        let banned = Banned::load_from_state(&state).unwrap();
        assert!(banned.is_banned(&criminal));

        let candidates = Candidates::load_from_state(&state).unwrap();
        assert_eq!(candidates.len(), 0);

        assert_eq!(state.balance(&criminal).unwrap(), 900, "Should lose deposit");

        let delegation = Delegation::load_from_state(&state, &delegator).unwrap();
        assert_eq!(delegation.get_quantity(&criminal), 0, "Delegation should be reverted");

        let account_delegator = StakeAccount::load_from_state(&state, &delegator).unwrap();
        assert_eq!(account_delegator.balance, 100, "Delegation should be reverted");
    }

    #[test]
    fn ban_should_remove_prisoner_from_jail() {
        let informant_pubkey = Public::random();
        let criminal_pubkey = Public::random();
        let informant = public_to_address(&informant_pubkey);
        let criminal = public_to_address(&criminal_pubkey);

        let mut state = helpers::get_temp_state();
        super::init(&mut state, Default::default(), Default::default(), Default::default()).unwrap();
        assert_eq!(Ok(()), state.add_balance(&criminal, 100));

        let deposit = 10;
        self_nominate(&mut state, &criminal, &criminal_pubkey, deposit, 0, 10, b"".to_vec()).unwrap();
        let custody_until = 10;
        let released_at = 20;
        jail(&mut state, &[criminal], custody_until, released_at).unwrap();

        assert_eq!(Ok(()), ban(&mut state, &informant, criminal));

        let jail = Jail::load_from_state(&state).unwrap();
        assert_eq!(jail.get_prisoner(&criminal), None, "Should be removed from the jail");
    }

    fn pseudo_term_to_block_num_calculator(term_id: u64) -> u64 {
        term_id * 10 + 1
    }
}

#[cfg(test)]
mod tests_double_vote {
    use super::*;
    use crate::client::{ConsensusClient, TestBlockChainClient};
    use crate::consensus::{DynamicValidator, Step, VoteOn, VoteStep};
    use ckey::sign;
    use ctypes::BlockHash;
    use primitives::H256;
    use rlp::Encodable;

    struct ConsensusMessageInfo {
        pub height: u64,
        pub view: u64,
        pub step: Step,
        pub block_hash: Option<BlockHash>,
        pub signer_index: usize,
    }

    fn create_consensus_message<F, G>(
        info: ConsensusMessageInfo,
        client: &TestBlockChainClient,
        vote_step_twister: &F,
        block_hash_twister: &G,
    ) -> ConsensusMessage
    where
        F: Fn(VoteStep) -> VoteStep,
        G: Fn(Option<BlockHash>) -> Option<BlockHash>, {
        let ConsensusMessageInfo {
            height,
            view,
            step,
            block_hash,
            signer_index,
        } = info;
        let vote_step = VoteStep::new(height, view, step);
        let on = VoteOn {
            step: vote_step,
            block_hash,
        };
        let twisted = VoteOn {
            step: vote_step_twister(vote_step),
            block_hash: block_hash_twister(block_hash),
        };
        let reversed_idx = client.get_validators().len() - 1 - signer_index;
        let pubkey = *client.get_validators().get(reversed_idx).unwrap().pubkey();
        let validator_keys = client.validator_keys.read();
        let privkey = validator_keys.get(&pubkey).unwrap();
        let signature = sign(&twisted.hash(), privkey);

        ConsensusMessage {
            signature,
            signer_index,
            on,
        }
    }

    fn double_vote_verification_result<F, G>(
        message_info1: ConsensusMessageInfo,
        message_info2: ConsensusMessageInfo,
        vote_step_twister: &F,
        block_hash_twister: &G,
    ) -> Result<(), SyntaxError>
    where
        F: Fn(VoteStep) -> VoteStep,
        G: Fn(Option<BlockHash>) -> Option<BlockHash>, {
        let mut test_client = TestBlockChainClient::default();
        test_client.add_blocks(10, 1);
        test_client.set_random_validators(10);
        let validator_set = DynamicValidator::default();

        let consensus_message1 =
            create_consensus_message(message_info1, &test_client, vote_step_twister, block_hash_twister);
        let consensus_message2 =
            create_consensus_message(message_info2, &test_client, vote_step_twister, block_hash_twister);
        let action = StakeAction::ReportDoubleVote {
            message1: consensus_message1.rlp_bytes(),
            message2: consensus_message2.rlp_bytes(),
        };
        let arced_client: Arc<dyn ConsensusClient> = Arc::new(test_client);
        validator_set.register_client(Arc::downgrade(&arced_client));
        action.verify(&CommonParams::default_for_test())?;
        verify_report_double_vote(consensus_message1, consensus_message2, &*arced_client, &validator_set)
    }

    #[test]
    fn double_vote_verify_desirable_report() {
        let result = double_vote_verification_result(
            ConsensusMessageInfo {
                height: 2,
                view: 0,
                step: Step::Precommit,
                block_hash: None,
                signer_index: 0,
            },
            ConsensusMessageInfo {
                height: 2,
                view: 0,
                step: Step::Precommit,
                block_hash: Some(H256::random().into()),
                signer_index: 0,
            },
            &|v| v,
            &|v| v,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn double_vote_verify_same_message() {
        let block_hash = Some(H256::random().into());
        let result = double_vote_verification_result(
            ConsensusMessageInfo {
                height: 3,
                view: 1,
                step: Step::Precommit,
                block_hash,
                signer_index: 2,
            },
            ConsensusMessageInfo {
                height: 3,
                view: 1,
                step: Step::Precommit,
                block_hash,
                signer_index: 2,
            },
            &|v| v,
            &|v| v,
        );
        let expected_err = Err(SyntaxError::InvalidCustomAction(String::from("Messages are duplicated")));
        assert_eq!(result, expected_err);
    }

    #[test]
    fn double_vote_verify_different_height() {
        let block_hash = Some(H256::random().into());
        let result = double_vote_verification_result(
            ConsensusMessageInfo {
                height: 3,
                view: 1,
                step: Step::Precommit,
                block_hash,
                signer_index: 2,
            },
            ConsensusMessageInfo {
                height: 2,
                view: 1,
                step: Step::Precommit,
                block_hash,
                signer_index: 2,
            },
            &|v| v,
            &|v| v,
        );
        let expected_err =
            Err(SyntaxError::InvalidCustomAction(String::from("The messages are from two different voting rounds")));
        assert_eq!(result, expected_err);
    }

    #[test]
    fn double_vote_verify_different_signer() {
        let result = double_vote_verification_result(
            ConsensusMessageInfo {
                height: 2,
                view: 0,
                step: Step::Precommit,
                block_hash: None,
                signer_index: 1,
            },
            ConsensusMessageInfo {
                height: 2,
                view: 0,
                step: Step::Precommit,
                block_hash: Some(H256::random().into()),
                signer_index: 0,
            },
            &|v| v,
            &|v| v,
        );
        match result {
            Err(SyntaxError::InvalidCustomAction(ref s))
                if s.contains("Two messages have different signer indexes") => {}
            _ => panic!(),
        }
    }

    #[test]
    fn double_vote_verify_different_message_and_signer() {
        let hash1 = Some(H256::random().into());
        let mut hash2 = Some(H256::random().into());
        while hash1 == hash2 {
            hash2 = Some(H256::random().into());
        }
        let result = double_vote_verification_result(
            ConsensusMessageInfo {
                height: 2,
                view: 0,
                step: Step::Precommit,
                block_hash: hash1,
                signer_index: 1,
            },
            ConsensusMessageInfo {
                height: 2,
                view: 0,
                step: Step::Precommit,
                block_hash: hash2,
                signer_index: 0,
            },
            &|v| v,
            &|v| v,
        );
        match result {
            Err(SyntaxError::InvalidCustomAction(ref s))
                if s.contains("Two messages have different signer indexes") => {}
            _ => panic!(),
        }
    }

    #[test]
    fn double_vote_verify_strange_sig1() {
        let vote_step_twister = |original: VoteStep| VoteStep {
            height: original.height + 1,
            view: original.height + 1,
            step: original.step,
        };
        let result = double_vote_verification_result(
            ConsensusMessageInfo {
                height: 2,
                view: 0,
                step: Step::Precommit,
                block_hash: None,
                signer_index: 0,
            },
            ConsensusMessageInfo {
                height: 2,
                view: 0,
                step: Step::Precommit,
                block_hash: Some(H256::random().into()),
                signer_index: 0,
            },
            &vote_step_twister,
            &|v| v,
        );
        let expected_err = Err(SyntaxError::InvalidCustomAction(String::from("Ed25519 signature verification fails")));
        assert_eq!(result, expected_err);
    }

    #[test]
    fn double_vote_verify_strange_sig2() {
        let block_hash_twister = |original: Option<BlockHash>| {
            original.map(|hash| {
                let mut twisted = H256::random();
                while twisted == *hash {
                    twisted = H256::random();
                }
                BlockHash::from(twisted)
            })
        };
        let result = double_vote_verification_result(
            ConsensusMessageInfo {
                height: 2,
                view: 0,
                step: Step::Precommit,
                block_hash: None,
                signer_index: 0,
            },
            ConsensusMessageInfo {
                height: 2,
                view: 0,
                step: Step::Precommit,
                block_hash: Some(H256::random().into()),
                signer_index: 0,
            },
            &|v| v,
            &block_hash_twister,
        );
        let expected_err = Err(SyntaxError::InvalidCustomAction(String::from("Ed25519 signature verification fails")));
        assert_eq!(result, expected_err);
    }
}
