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

use crate::{
    Banned, Candidates, Delegation, Jail, NextValidators, ReleaseResult, StakeAccount, Stakeholders, StateResult,
    TopLevelState, TopState,
};
use ckey::{public_to_address, Address, Ed25519Public as Public};
use ctypes::errors::RuntimeError;
use ctypes::transaction::Approval;
use ctypes::util::unexpected::Mismatch;
use ctypes::{CommonParams, Deposit};
use primitives::Bytes;
use std::collections::HashMap;

#[allow(clippy::implicit_hasher)] // XXX: Fix this clippy warning if it becomes a real problem.
pub fn init_stake(
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
        // This balance was an element of `TopLevelState`, but the concept of `Account` was moved
        // to a module level, and the element was removed from `TopLevelState`. Therefore, this balance
        // was newly defiend for build, and its value is temporarily Default::default().

        let balance: u64 = Default::default();
        if balance < deposit.deposit {
            cerror!(STATE, "{} has insufficient balance to become the candidate", address);
            return Err(RuntimeError::InsufficientBalance {
                address: *address,
                balance,
                cost: deposit.deposit,
            }
            .into())
        }
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

pub fn transfer_ccs(state: &mut TopLevelState, sender: &Address, receiver: &Address, quantity: u64) -> StateResult<()> {
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

pub fn delegate_ccs(
    state: &mut TopLevelState,
    delegator: &Address,
    delegatee: &Address,
    quantity: u64,
) -> StateResult<()> {
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

pub fn revoke(state: &mut TopLevelState, delegator: &Address, delegatee: &Address, quantity: u64) -> StateResult<()> {
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

pub fn redelegate(
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

pub fn self_nominate(
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

pub fn change_params(
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

fn get_stakes(state: &TopLevelState) -> StateResult<HashMap<Address, u64>> {
    let stakeholders = Stakeholders::load_from_state(state)?;
    let mut result = HashMap::new();
    for stakeholder in stakeholders.iter() {
        let account = StakeAccount::load_from_state(state, stakeholder)?;
        let delegation = Delegation::load_from_state(state, stakeholder)?;
        result.insert(*stakeholder, account.balance + delegation.sum());
    }
    Ok(result)
}

pub fn release_jailed_prisoners(state: &mut TopLevelState, current_term: u64) -> StateResult<Vec<Address>> {
    let mut jailed = Jail::load_from_state(&state)?;
    let released = jailed.drain_released_prisoners(current_term);
    for prisoner in &released {
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

pub fn ban(state: &mut TopLevelState, _informant: &Address, criminal: Address) -> StateResult<()> {
    let mut banned = Banned::load_from_state(state)?;
    if banned.is_banned(&criminal) {
        return Err(RuntimeError::FailedToHandleCustomAction("Account is already banned".to_string()).into())
    }

    let mut candidates = Candidates::load_from_state(state)?;
    let mut jailed = Jail::load_from_state(state)?;
    let mut validators = NextValidators::load_from_state(state)?;

    let _deposit = match (candidates.remove(&criminal), jailed.remove(&criminal)) {
        (Some(_), Some(_)) => unreachable!("A candidate that are jailed cannot exist"),
        (Some(candidate), _) => candidate.deposit,
        (_, Some(jailed)) => jailed.deposit,
        _ => 0,
    };
    // confiscate criminal's deposit and give the same deposit amount to the informant.

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

pub fn revert_delegations(state: &mut TopLevelState, reverted_delegatees: &[Address]) -> StateResult<()> {
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

pub fn update_validator_weights(state: &mut TopLevelState, block_author: &Address) -> StateResult<()> {
    let mut validators = NextValidators::load_from_state(state)?;
    validators.update_weight(block_author);
    validators.save_to_state(state)
}

pub fn update_candidates(
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
        ctrace!(ENGINE, "on_term_close::expired. candidate: {}, deposit: {}", address, candidate.deposit);
    }
    candidates.save_to_state(state)?;
    Ok(expired.into_iter().map(|c| public_to_address(&c.pubkey)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::helpers;
    use crate::{
        get_delegation_key, get_stake_account_key, init_stake, Banned, Candidate, Candidates, Delegation, Jail,
        Prisoner, StakeAccount, Stakeholders, TopStateView,
    };
    use std::collections::HashMap;

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
        init_stake(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();

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
        init_stake(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();

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
        init_stake(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();

        let result = transfer_ccs(&mut state, &address1, &address2, 100);
        assert_eq!(result, Ok(()));

        let account1 = StakeAccount::load_from_state(&state, &address1).unwrap();
        assert_eq!(account1.balance, 0);
        assert_eq!(state.action_data(&get_stake_account_key(&address1)).unwrap(), None, "Should clear state");

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
        init_stake(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();
        self_nominate(&mut state, &delegatee, &delegatee_pubkey, 0, 0, 10, b"".to_vec()).unwrap();

        let quantity = 40;
        delegate_ccs(&mut state, &delegator, &delegatee, quantity).unwrap();

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
        init_stake(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();
        self_nominate(&mut state, &delegatee, &delegatee_pubkey, 0, 0, 10, b"".to_vec()).unwrap();

        let quantity = 100;
        delegate_ccs(&mut state, &delegator, &delegatee, quantity).unwrap();

        let delegator_account = StakeAccount::load_from_state(&state, &delegator).unwrap();
        assert_eq!(delegator_account.balance, 0);
        assert_eq!(state.action_data(&get_stake_account_key(&delegator)).unwrap(), None, "Should clear state");

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
        init_stake(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();

        let quantity = 40;
        delegate_ccs(&mut state, &delegator, &delegatee, quantity).unwrap_err();
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
        init_stake(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();
        self_nominate(&mut state, &delegatee, &delegatee_pubkey, 0, 0, 10, b"".to_vec()).unwrap();

        let quantity = 200;
        delegate_ccs(&mut state, &delegator, &delegatee, quantity).unwrap_err();
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
        init_stake(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();
        self_nominate(&mut state, &delegatee, &delegatee_pubkey, 0, 0, 10, b"".to_vec()).unwrap();

        let quantity = 50;
        delegate_ccs(&mut state, &delegator, &delegatee, quantity).unwrap();

        let quantity = 50;
        transfer_ccs(&mut state, &delegator, &delegatee, quantity).unwrap();
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
        init_stake(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();
        self_nominate(&mut state, &delegatee, &delegatee_pubkey, 0, 0, 10, b"".to_vec()).unwrap();

        let quantity = 50;
        delegate_ccs(&mut state, &delegator, &delegatee, quantity).unwrap();

        let quantity = 100;
        transfer_ccs(&mut state, &delegator, &delegatee, quantity).unwrap_err();
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
        init_stake(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();
        self_nominate(&mut state, &delegatee, &delegatee_pubkey, 0, 0, 10, b"".to_vec()).unwrap();

        let quantity = 50;
        delegate_ccs(&mut state, &delegator, &delegatee, quantity).unwrap();

        let quantity = 20;
        revoke(&mut state, &delegator, &delegatee, quantity).unwrap();

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
        init_stake(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();
        self_nominate(&mut state, &delegatee, &delegatee_pubkey, 0, 0, 10, b"".to_vec()).unwrap();

        let quantity = 50;
        delegate_ccs(&mut state, &delegator, &delegatee, quantity).unwrap();

        let quantity = 70;
        revoke(&mut state, &delegator, &delegatee, quantity).unwrap_err();

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
        init_stake(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();
        self_nominate(&mut state, &delegatee, &delegatee_pubkey, 0, 0, 10, b"".to_vec()).unwrap();

        let quantity = 50;
        delegate_ccs(&mut state, &delegator, &delegatee, quantity).unwrap();

        let quantity = 50;
        revoke(&mut state, &delegator, &delegatee, quantity).unwrap();

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
        init_stake(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();
        self_nominate(&mut state, &prev_delegatee, &prev_delegatee_pubkey, 0, 0, 10, b"".to_vec()).unwrap();
        self_nominate(&mut state, &next_delegatee, &next_delegatee_pubkey, 0, 0, 10, b"".to_vec()).unwrap();

        let quantity = 50;
        delegate_ccs(&mut state, &delegator, &prev_delegatee, quantity).unwrap();

        let quantity = 20;
        redelegate(&mut state, &delegator, &prev_delegatee, &next_delegatee, quantity).unwrap();

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
        init_stake(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();
        self_nominate(&mut state, &prev_delegatee, &prev_delegatee_pubkey, 0, 0, 10, b"".to_vec()).unwrap();
        self_nominate(&mut state, &next_delegatee, &next_delegatee_pubkey, 0, 0, 10, b"".to_vec()).unwrap();

        let quantity = 50;
        delegate_ccs(&mut state, &delegator, &prev_delegatee, quantity).unwrap();

        let quantity = 70;
        redelegate(&mut state, &delegator, &prev_delegatee, &next_delegatee, quantity).unwrap_err();

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
        init_stake(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();
        self_nominate(&mut state, &prev_delegatee, &prev_delegatee_pubkey, 0, 0, 10, b"".to_vec()).unwrap();
        self_nominate(&mut state, &next_delegatee, &next_delegatee_pubkey, 0, 0, 10, b"".to_vec()).unwrap();

        let quantity = 50;
        delegate_ccs(&mut state, &delegator, &prev_delegatee, quantity).unwrap();

        let quantity = 50;
        redelegate(&mut state, &delegator, &prev_delegatee, &next_delegatee, quantity).unwrap();

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
        init_stake(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();

        self_nominate(&mut state, &prev_delegatee, &prev_delegatee_pubkey, 0, 0, 10, b"".to_vec()).unwrap();

        let quantity = 40;
        delegate_ccs(&mut state, &delegator, &prev_delegatee, quantity).unwrap();

        let quantity = 50;
        redelegate(&mut state, &delegator, &prev_delegatee, &next_delegatee, quantity).unwrap_err();
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

        let genesis_stakes = {
            let mut genesis_stakes = HashMap::new();
            genesis_stakes.insert(delegator, 100);
            genesis_stakes
        };
        init_stake(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();
        self_nominate(&mut state, &prev_delegatee, &prev_delegatee_pubkey, 0, 0, 10, b"".to_vec()).unwrap();
        self_nominate(&mut state, &criminal, &criminal_pubkey, 100, 0, 10, b"".to_vec()).unwrap();

        let quantity = 40;
        delegate_ccs(&mut state, &delegator, &criminal, quantity).unwrap();
        let quantity = 40;
        delegate_ccs(&mut state, &delegator, &prev_delegatee, quantity).unwrap();

        let candidates = Candidates::load_from_state(&state).unwrap();
        assert_eq!(candidates.len(), 2);

        assert_eq!(Ok(()), ban(&mut state, &informant, criminal));

        let banned = Banned::load_from_state(&state).unwrap();
        assert!(banned.is_banned(&criminal));

        let candidates = Candidates::load_from_state(&state).unwrap();
        assert_eq!(candidates.len(), 1);

        let quantity = 40;
        redelegate(&mut state, &delegator, &prev_delegatee, &criminal, quantity).unwrap_err();
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

        let genesis_stakes = {
            let mut genesis_stakes = HashMap::new();
            genesis_stakes.insert(delegator, 100);
            genesis_stakes
        };
        init_stake(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();
        self_nominate(&mut state, &prev_delegatee, &prev_delegatee_pubkey, 0, 0, 10, b"".to_vec()).unwrap();

        let deposit = 200;
        self_nominate(&mut state, &jail_address, &jail_pubkey, deposit, 0, 5, b"".to_vec()).unwrap();

        let quantity = 40;
        delegate_ccs(&mut state, &delegator, &prev_delegatee, quantity).unwrap();

        let candidates = Candidates::load_from_state(&state).unwrap();
        assert_eq!(candidates.len(), 2);

        let custody_until = 10;
        let released_at = 20;
        let result = jail(&mut state, &[jail_address], custody_until, released_at);
        assert!(result.is_ok());

        let candidates = Candidates::load_from_state(&state).unwrap();
        assert_eq!(candidates.len(), 1);

        let quantity = 40;
        redelegate(&mut state, &delegator, &prev_delegatee, &jail_address, quantity).unwrap_err();
    }

    #[test]
    fn self_nominate_deposit_test() {
        let address_pubkey = Public::random();
        let address = public_to_address(&address_pubkey);

        let mut state = helpers::get_temp_state();

        init_stake(&mut state, Default::default(), Default::default(), Default::default()).unwrap();

        // TODO: change with stake::execute()
        let result = self_nominate(&mut state, &address, &address_pubkey, 0, 0, 5, b"metadata1".to_vec());
        assert_eq!(result, Ok(()));
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

    #[allow(dead_code)]
    fn self_nominate_fail_with_insufficient_balance() {
        let address_pubkey = Public::random();
        let address = public_to_address(&address_pubkey);

        let mut state = helpers::get_temp_state();

        init_stake(&mut state, Default::default(), Default::default(), Default::default()).unwrap();

        // TODO: change with stake::execute()
        let result = self_nominate(&mut state, &address, &address_pubkey, 2000, 0, 5, b"".to_vec());
        assert!(result.is_err(), "Cannot self-nominate without a sufficient balance");
    }

    #[test]
    fn jail_candidate() {
        let address_pubkey = Public::random();
        let address = public_to_address(&address_pubkey);

        let mut state = helpers::get_temp_state();
        init_stake(&mut state, Default::default(), Default::default(), Default::default()).unwrap();

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

        let genesis_stakes = {
            let mut genesis_stakes = HashMap::new();
            genesis_stakes.insert(delegator, 100);
            genesis_stakes
        };
        init_stake(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();

        let deposit = 100;
        self_nominate(&mut state, &criminal, &criminal_pubkey, deposit, 0, 10, b"".to_vec()).unwrap();

        let quantity = 40;
        delegate_ccs(&mut state, &delegator, &criminal, quantity).unwrap();

        assert_eq!(Ok(()), ban(&mut state, &informant, criminal));

        let banned = Banned::load_from_state(&state).unwrap();
        assert!(banned.is_banned(&criminal));

        let candidates = Candidates::load_from_state(&state).unwrap();
        assert_eq!(candidates.len(), 0);

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
        init_stake(&mut state, Default::default(), Default::default(), Default::default()).unwrap();

        let deposit = 10;
        self_nominate(&mut state, &criminal, &criminal_pubkey, deposit, 0, 10, b"".to_vec()).unwrap();
        let custody_until = 10;
        let released_at = 20;
        jail(&mut state, &[criminal], custody_until, released_at).unwrap();

        assert_eq!(Ok(()), ban(&mut state, &informant, criminal));

        let jail = Jail::load_from_state(&state).unwrap();
        assert_eq!(jail.get_prisoner(&criminal), None, "Should be removed from the jail");
    }
}
