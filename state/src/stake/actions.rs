// Copyright 2020 Kodebox, Inc.
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

use crate::item::stake::{Delegation, StakeAccount, Stakeholders};
use crate::{Banned, Candidates, Jail, NextValidators, StateResult, TopLevelState};
use ckey::Ed25519Public as Public;
use ctypes::errors::RuntimeError;
use ctypes::util::unexpected::Mismatch;
use ctypes::{Deposit, TransactionIndex, TransactionLocation};
use std::collections::HashMap;

#[allow(clippy::implicit_hasher)] // XXX: Fix this clippy warning if it becomes a real problem.
pub fn init_stake(
    state: &mut TopLevelState,
    genesis_stakes: HashMap<Public, u64>,
    genesis_candidates: HashMap<Public, Deposit>,
    genesis_delegations: HashMap<Public, HashMap<Public, u64>>,
) -> StateResult<()> {
    let mut genesis_stakes = genesis_stakes;
    for (delegator, delegation) in &genesis_delegations {
        let stake = genesis_stakes.entry(*delegator).or_default();
        let total_delegation = delegation.values().sum();
        if *stake < total_delegation {
            cerror!(STATE, "{:?} has insufficient stakes to delegate", delegator);
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
    for (pubkey, amount) in &genesis_stakes {
        let account = StakeAccount {
            pubkey,
            balance: *amount,
        };
        account.save_to_state(state)?;
        stakeholders.update_by_increased_balance(&account);
    }
    stakeholders.save_to_state(state)?;

    for (pubkey, deposit) in &genesis_candidates {
        // This balance was an element of `TopLevelState`, but the concept of `Account` was moved
        // to a module level, and the element was removed from `TopLevelState`. Therefore, this balance
        // was newly defiend for build, and its value is temporarily Default::default().

        let balance: u64 = Default::default();
        if balance < deposit.deposit {
            cerror!(STATE, "{:?} has insufficient balance to become the candidate", pubkey);
            return Err(RuntimeError::InsufficientBalance {
                pubkey: *pubkey,
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

        for (index, candidate) in values.iter().enumerate() {
            let block_number = 0;
            // `index` is not a real transaction index.
            // Since Candidate struct requires transaction index to order candidates, let's add a fake one.
            let transaction_index = index as TransactionIndex;
            candidates.add_deposit(
                &candidate.pubkey,
                candidate.deposit,
                candidate.nomination_ends_at,
                TransactionLocation {
                    block_number,
                    transaction_index,
                },
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

pub fn ban(state: &mut TopLevelState, _informant: &Public, criminal: Public) -> StateResult<()> {
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

pub fn revert_delegations(state: &mut TopLevelState, reverted_delegatees: &[Public]) -> StateResult<()> {
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
                    "revert_delegation delegator: {:?}, delegatee: {:?}, quantity: {}",
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init_stake;
    use crate::item::stake::{StakeAccount, Stakeholders};
    use crate::tests::helpers;
    use std::collections::HashMap;

    #[test]
    fn genesis_stakes() {
        let pubkey1 = Public::random();
        let pubkey2 = Public::random();

        let mut state = helpers::get_temp_state();
        let genesis_stakes = {
            let mut genesis_stakes = HashMap::new();
            genesis_stakes.insert(pubkey1, 100);
            genesis_stakes
        };
        init_stake(&mut state, genesis_stakes, Default::default(), Default::default()).unwrap();

        let account1 = StakeAccount::load_from_state(&state, &pubkey1).unwrap();
        assert_eq!(account1.balance, 100);

        let account2 = StakeAccount::load_from_state(&state, &pubkey2).unwrap();
        assert_eq!(account2.balance, 0);

        let stakeholders = Stakeholders::load_from_state(&state).unwrap();
        assert_eq!(stakeholders.iter().len(), 1);
        assert!(stakeholders.contains(&pubkey1));
        assert!(!stakeholders.contains(&pubkey2));
    }
}
