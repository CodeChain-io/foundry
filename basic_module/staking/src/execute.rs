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

use crate::core::TransactionExecutionOutcome;
use crate::error::{Insufficient, Mismatch};
use crate::runtime_error::Error;
use crate::state::*;
use crate::transactions::{Action, Transaction};
use crate::types::{Approval, Bytes, Public, ReleaseResult, ResultantFee, StakeQuantity};
use crate::{account_manager, account_viewer, substorage};

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
    tx: Transaction,
    sender_public: &Public,
    current_block_number: u64,
) -> Result<(TransactionExecutionOutcome, ResultantFee), Error> {
    let Transaction {
        action,
        fee,
        seq,
        ..
    } = tx;

    let min_fee = action.min_fee();
    check_before_fee_imposition(sender_public, fee, seq, min_fee)?;

    // Does not impose fee and increase sequence for a failed transaction
    let substorage = substorage();
    substorage.create_checkpoint();

    let account_manager = account_manager();
    account_manager.sub_balance(sender_public, fee).map_err(|_err| {
        Error::InsufficientBalance(Insufficient {
            required: fee,
            actual: account_viewer().get_balance(sender_public),
        })
    })?;
    account_manager.increment_sequence(&sender_public);

    let result = execute(&sender_public, action, current_block_number);
    match result {
        Ok(_) => substorage.discard_checkpoint(),
        Err(_) => substorage.revert_to_the_checkpoint(),
    };

    result.map(|outcome| {
        (outcome, ResultantFee {
            additional_fee: fee - min_fee,
            min_fee,
        })
    })
}

fn execute(
    sender_public: &Public,
    action: Action,
    current_block_number: u64,
) -> Result<TransactionExecutionOutcome, Error> {
    match action {
        Action::TransferCCS {
            receiver_public,
            quantity,
        } => transfer_ccs(sender_public, &receiver_public, quantity),
        Action::DelegateCCS {
            delegatee_public,
            quantity,
        } => delegate_ccs(sender_public, &delegatee_public, quantity),
        Action::Revoke {
            delegatee_public,
            quantity,
        } => revoke(sender_public, &delegatee_public, quantity),
        Action::Redelegate {
            prev_delegatee,
            next_delegatee,
            quantity,
        } => redelegate(sender_public, &prev_delegatee, &next_delegatee, quantity),
        Action::SelfNominate {
            deposit,
            metadata,
        } => self_nominate(sender_public, deposit, metadata),
        Action::ChangeParams {
            metadata_seq,
            params,
            approvals,
        } => change_params(metadata_seq, params, approvals),
        Action::ReportDoubleVote {
            ..
        } => unimplemented!(),
    }
}

fn transfer_ccs(from: &Public, to: &Public, quantity: StakeQuantity) -> Result<TransactionExecutionOutcome, Error> {
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

fn delegate_ccs(delegator: &Public, delegatee: &Public, quantity: u64) -> Result<TransactionExecutionOutcome, Error> {
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

fn revoke(delegator: &Public, delegatee: &Public, quantity: u64) -> Result<TransactionExecutionOutcome, Error> {
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
) -> Result<TransactionExecutionOutcome, Error> {
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
) -> Result<TransactionExecutionOutcome, Error> {
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
    candidates.add_deposit(nominee_public, total_deposit, nomination_ends_at, metadata);

    jail.save();
    candidates.save();

    Ok(Default::default())
}

pub fn change_params(
    metadata_seq: u64,
    params: Params,
    approvals: Vec<Approval>,
) -> Result<TransactionExecutionOutcome, Error> {
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
