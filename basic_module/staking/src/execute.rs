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

use crate::error::{Insufficient, Mismatch};
use crate::runtime_error::Error;
use crate::state::{
    get_stakes, Banned, Candidates, CurrentValidators, Delegation, Jail, Metadata, NextValidators, Params,
    StakeAccount, Stakeholders,
};
use crate::transactions::{AutoAction, UserAction, UserTransaction};
use crate::types::{Approval, ReleaseResult, StakeQuantity, Tiebreaker};
// use crate::{account_manager, account_viewer, substorage};
use crate::{account_manager, account_viewer};
use coordinator::types::TransactionOutcome;
use fkey::Ed25519Public as Public;
use primitives::Bytes;

fn check_before_fee_imposition(sender_public: &Public, fee: u64, seq: u64, min_fee: u64) -> Result<(), Error> {
    let account_sequence = account_viewer().get_sequence(sender_public);
    if account_sequence != seq {
        Err(Error::InvalidSeq(Mismatch {
            expected: seq,
            found: account_sequence,
        }))
    } else if fee < min_fee {
        Err(Error::InsufficientFee(Insufficient {
            required: min_fee,
            actual: fee,
        }))
    } else {
        Ok(())
    }
}

pub fn apply_internal(
    tx: UserTransaction,
    sender_public: &Public,
    tiebreaker: Tiebreaker,
) -> Result<TransactionOutcome, Error> {
    let UserTransaction {
        action,
        fee,
        seq,
        ..
    } = tx;

    let min_fee = action.min_fee();
    check_before_fee_imposition(sender_public, fee, seq, min_fee)?;

    // Does not impose fee and increase sequence for a failed transaction
    // let mut substorage = substorage();
    // substorage.create_checkpoint();

    let account_manager = account_manager();
    account_manager.sub_balance(sender_public, fee).map_err(|_err| {
        Error::InsufficientBalance(Insufficient {
            required: fee,
            actual: account_viewer().get_balance(sender_public),
        })
    })?;
    account_manager.increment_sequence(&sender_public);

    let result = execute_user_action(&sender_public, action, tiebreaker);
    // match result {
    //     Ok(_) => substorage.discard_checkpoint(),
    //     Err(_) => substorage.revert_to_the_checkpoint(),
    // };

    result
}

fn execute_user_action(
    sender_public: &Public,
    action: UserAction,
    tiebreaker: Tiebreaker,
) -> Result<TransactionOutcome, Error> {
    match action {
        UserAction::TransferCCS {
            receiver_public,
            quantity,
        } => transfer_ccs(sender_public, &receiver_public, quantity),
        UserAction::DelegateCCS {
            delegatee_public,
            quantity,
        } => delegate_ccs(sender_public, &delegatee_public, quantity),
        UserAction::Revoke {
            delegatee_public,
            quantity,
        } => revoke(sender_public, &delegatee_public, quantity),
        UserAction::Redelegate {
            prev_delegatee,
            next_delegatee,
            quantity,
        } => redelegate(sender_public, &prev_delegatee, &next_delegatee, quantity),
        UserAction::SelfNominate {
            deposit,
            metadata,
        } => self_nominate(sender_public, deposit, metadata, tiebreaker),
        UserAction::ChangeParams {
            metadata_seq,
            params,
            approvals,
        } => change_params(metadata_seq, params, approvals),
        UserAction::ReportDoubleVote {
            ..
        } => unimplemented!(),
    }
}

pub fn execute_auto_action(action: AutoAction, current_block_number: u64) -> Result<TransactionOutcome, Error> {
    match action {
        AutoAction::UpdateValidators {
            validators,
        } => update_validators(validators),
        AutoAction::CloseTerm {
            inactive_validators,
            next_validators,
            released_addresses,
            custody_until,
            kick_at,
        } => {
            close_term(next_validators, &inactive_validators)?;
            release_jailed_prisoners(&released_addresses)?;
            jail(&inactive_validators, custody_until, kick_at);
            increase_term_id(current_block_number);
            Ok(Default::default())
        }
        AutoAction::Elect => {
            NextValidators::elect().save();
            let mut metadata = Metadata::load();
            metadata.update_term_params();
            metadata.save();
            Ok(Default::default())
        }
        AutoAction::ChangeNextValidators {
            validators,
        } => {
            NextValidators::from(validators).save();
            Ok(Default::default())
        }
    }
}

fn transfer_ccs(from: &Public, to: &Public, quantity: StakeQuantity) -> Result<TransactionOutcome, Error> {
    let mut stakeholders = Stakeholders::load();
    let mut sender_account = StakeAccount::load(from);
    let mut receiver_account = StakeAccount::load(to);
    let sender_delegations = Delegation::load(from);

    sender_account.subtract_balance(quantity)?;
    receiver_account.add_balance(quantity)?;

    stakeholders.update_by_decreased_balance(&sender_account, &sender_delegations);
    stakeholders.update_by_increased_balance(&receiver_account);

    stakeholders.save();
    sender_account.save();
    receiver_account.save();

    Ok(Default::default())
}

fn delegate_ccs(delegator: &Public, delegatee: &Public, quantity: u64) -> Result<TransactionOutcome, Error> {
    let candidates = Candidates::load();
    if candidates.get_candidate(delegatee).is_none() {
        return Err(Error::DelegateeNotFoundInCandidates(*delegatee))
    }

    let banned = Banned::load();
    let jailed = Jail::load();
    assert!(!banned.is_banned(delegatee), "A candidate must not be banned");
    assert_eq!(None, jailed.get_prisoner(delegatee), "A candidate must not be jailed");

    let mut delegator_account = StakeAccount::load(delegator);
    let mut delegation = Delegation::load(delegator);

    delegator_account.subtract_balance(quantity)?;
    delegation.add_quantity(*delegatee, quantity)?;
    // delegation does not touch stakeholders

    delegation.save();
    delegator_account.save();

    Ok(Default::default())
}

fn revoke(delegator: &Public, delegatee: &Public, quantity: u64) -> Result<TransactionOutcome, Error> {
    let mut delegator_account = StakeAccount::load(delegator);
    let mut delegation = Delegation::load(delegator);

    delegator_account.add_balance(quantity)?;
    delegation.sub_quantity(*delegatee, quantity)?;
    // delegation does not touch stakeholders

    delegation.save();
    delegator_account.save();

    Ok(Default::default())
}

fn redelegate(
    delegator: &Public,
    prev_delegatee: &Public,
    next_delegatee: &Public,
    quantity: u64,
) -> Result<TransactionOutcome, Error> {
    let candidates = Candidates::load();
    if candidates.get_candidate(next_delegatee).is_none() {
        return Err(Error::DelegateeNotFoundInCandidates(*next_delegatee))
    }

    let banned = Banned::load();
    let jailed = Jail::load();
    assert!(!banned.is_banned(&next_delegatee), "A candidate must not be banned");
    assert_eq!(None, jailed.get_prisoner(next_delegatee), "A candidate must not be jailed");

    let delegator_account = StakeAccount::load(delegator);
    let mut delegation = Delegation::load(delegator);

    delegation.sub_quantity(*prev_delegatee, quantity)?;
    delegation.add_quantity(*next_delegatee, quantity)?;

    delegation.save();
    delegator_account.save();

    Ok(Default::default())
}

pub fn self_nominate(
    nominee_public: &Public,
    deposit: u64,
    metadata: Bytes,
    tiebreaker: Tiebreaker,
) -> Result<TransactionOutcome, Error> {
    let state_metadata = Metadata::load();
    let current_term = state_metadata.current_term_id;
    let nomination_ends_at = current_term + state_metadata.term_params.nomination_expiration;

    let blacklist = Banned::load();
    if blacklist.is_banned(nominee_public) {
        return Err(Error::BannedAccount(*nominee_public))
    }

    let mut jail = Jail::load();
    let total_deposit = match jail.try_release(nominee_public, current_term) {
        ReleaseResult::InCustody => return Err(Error::AccountInCustody(*nominee_public)),
        ReleaseResult::NotExists => deposit,
        ReleaseResult::Released(prisoner) => {
            assert_eq!(&prisoner.pubkey, nominee_public);
            prisoner.deposit + deposit
        }
    };

    let mut candidates = Candidates::load();
    // FIXME: Error handling is required
    account_manager().sub_balance(nominee_public, deposit).unwrap();
    candidates.add_deposit(nominee_public, total_deposit, nomination_ends_at, metadata, tiebreaker);

    jail.save();
    candidates.save();

    Ok(Default::default())
}

pub fn change_params(metadata_seq: u64, params: Params, approvals: Vec<Approval>) -> Result<TransactionOutcome, Error> {
    // Update state first because the signature validation is more expensive.
    let mut metadata = Metadata::load();
    metadata.update_params(metadata_seq, params)?;
    let stakes = get_stakes();
    // Approvals are verified
    let signed_stakes = approvals.iter().try_fold(0, |sum, approval| {
        let public = approval.signer_public;
        stakes.get(&public).map(|stake| sum + stake).ok_or_else(|| Error::SignatureOfInvalidAccount(public))
    })?;
    let total_stakes: u64 = stakes.values().sum();
    if total_stakes / 2 >= signed_stakes {
        return Err(Error::InsufficientStakes(Insufficient {
            required: total_stakes,
            actual: signed_stakes,
        }))
    }

    metadata.save();
    Ok(Default::default())
}

fn update_validators(validators: NextValidators) -> Result<TransactionOutcome, Error> {
    let next_validators_in_state = NextValidators::load();
    // NextValidators should be sorted by public key.
    if validators != next_validators_in_state {
        return Err(Error::InvalidValidators)
    }
    let mut current_validators = CurrentValidators::load();
    current_validators.update(validators.into());
    current_validators.save();
    Ok(Default::default())
}

fn close_term(next_validators: NextValidators, inactive_validators: &[Public]) -> Result<(), Error> {
    let metadata = Metadata::load();
    let current_term_id = metadata.current_term_id;
    let nomination_expiration = metadata.params.nomination_expiration;
    assert_ne!(0, nomination_expiration);

    update_candidates(current_term_id, nomination_expiration, &next_validators, inactive_validators)?;
    next_validators.save();
    Ok(())
}

fn update_candidates(
    current_term: u64,
    nomination_expiration: u64,
    next_validators: &NextValidators,
    inactive_validators: &[Public],
) -> Result<(), Error> {
    let banned = Banned::load();
    let mut candidates = Candidates::load();
    let nomination_ends_at = current_term + nomination_expiration;

    candidates.renew_candidates(next_validators, nomination_ends_at, inactive_validators, &banned);

    let expired = candidates.drain_expired_candidates(current_term);

    let account_manager = account_manager();
    for candidate in &expired {
        account_manager.add_balance(&candidate.pubkey, candidate.deposit);
    }
    candidates.save();
    let expired: Vec<_> = expired.into_iter().map(|c| c.pubkey).collect();
    revert_delegations(&expired)?;
    Ok(())
}

fn revert_delegations(reverted_delegatees: &[Public]) -> Result<(), Error> {
    let stakeholders = Stakeholders::load();
    for stakeholder in stakeholders.iter() {
        let mut delegator = StakeAccount::load(stakeholder);
        let mut delegation = Delegation::load(stakeholder);

        for delegatee in reverted_delegatees {
            let quantity = delegation.get_quantity(delegatee);
            if quantity > 0 {
                delegation.sub_quantity(*delegatee, quantity)?;
                delegator.add_balance(quantity)?;
            }
        }
        delegation.save();
        delegator.save();
    }
    Ok(())
}

fn release_jailed_prisoners(released: &[Public]) -> Result<(), Error> {
    if released.is_empty() {
        return Ok(())
    }

    let mut jailed = Jail::load();
    let account_manager = account_manager();
    for public in released {
        let prisoner = jailed.remove(public).unwrap();
        account_manager.add_balance(&public, prisoner.deposit);
    }
    jailed.save();
    revert_delegations(released)?;
    Ok(())
}

fn jail(publics: &[Public], custody_until: u64, kick_at: u64) {
    let mut candidates = Candidates::load();
    let mut jail = Jail::load();

    for public in publics {
        let candidate = candidates.remove(public).expect("There should be a candidate to jail");
        jail.add(candidate, custody_until, kick_at);
    }

    jail.save();
    candidates.save();
}

fn increase_term_id(last_term_finished_block_num: u64) {
    let mut metadata = Metadata::load();
    metadata.increase_term_id(last_term_finished_block_num);
    metadata.save();
}
