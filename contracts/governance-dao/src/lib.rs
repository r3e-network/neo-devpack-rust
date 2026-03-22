// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use neo_devpack::prelude::*;

mod config;
mod events;
mod proposals;
mod storage;
mod types;
mod utils;
mod voting;

use config::{load_config, load_stake, store_config, store_stake};
use proposals::{execute_proposal, load_proposal, next_proposal_id, store_proposal};
use storage::{serialize_value, storage_context};
use types::{DaoConfig, Proposal};
use utils::{ensure_witness, read_address, read_bytes, read_string};
use voting::{call_transfer, has_voted, record_vote};

neo_manifest_overlay!(
    r#"{
    "name": "NeoGovernanceDAO"
}"#
);

// Events
#[neo_event]
pub struct ProposalCreatedEvt {
    pub proposal_id: NeoInteger,
    pub proposer: NeoByteString,
    pub title: NeoString,
}

#[neo_event]
pub struct VoteCastEvt {
    pub proposal_id: NeoInteger,
    pub voter: NeoByteString,
    pub support: NeoBoolean,
    pub weight: NeoInteger,
}

#[neo_event]
pub struct ProposalExecutedEvt {
    pub proposal_id: NeoInteger,
}

#[neo_event]
pub struct StakeIncreasedEvt {
    pub staker: NeoByteString,
    pub amount: NeoInteger,
    pub new_total: NeoInteger,
}

#[neo_event]
pub struct StakeDecreasedEvt {
    pub staker: NeoByteString,
    pub amount: NeoInteger,
    pub new_total: NeoInteger,
}

#[neo_contract]
pub struct NeoGovernanceDaoContract;

#[neo_contract]
impl NeoGovernanceDaoContract {
    pub fn new() -> Self {
        Self
    }

    /// Initialize the DAO configuration. Only callable once.
    #[neo_method]
    pub fn configure(
        owner_ptr: i64,
        owner_len: i64,
        token_ptr: i64,
        token_len: i64,
        quorum: i64,
    ) -> bool {
        if quorum <= 0 {
            return false;
        }
        let owner = match read_address(owner_ptr, owner_len) {
            Some(a) => a,
            None => return false,
        };
        if !ensure_witness(&owner) {
            return false;
        }
        let token = match read_address(token_ptr, token_len) {
            Some(a) => a,
            None => return false,
        };
        let ctx = match storage_context() {
            Some(c) => c,
            None => return false,
        };
        if load_config(&ctx).is_some() {
            return false;
        }
        let cfg = DaoConfig {
            owner,
            token,
            quorum,
        };
        store_config(&ctx, &cfg).is_ok()
    }

    /// Create a governance proposal.
    #[neo_method]
    pub fn propose(
        proposer_ptr: i64,
        proposer_len: i64,
        target_ptr: i64,
        target_len: i64,
        method_ptr: i64,
        method_len: i64,
        args_ptr: i64,
        args_len: i64,
        title_ptr: i64,
        title_len: i64,
        start_time: i64,
        end_time: i64,
    ) -> bool {
        if end_time <= start_time {
            return false;
        }
        let proposer = match read_address(proposer_ptr, proposer_len) {
            Some(a) => a,
            None => return false,
        };
        if !ensure_witness(&proposer) {
            return false;
        }
        let ctx = match storage_context() {
            Some(c) => c,
            None => return false,
        };
        if load_config(&ctx).is_none() {
            return false;
        }
        let target = match read_address(target_ptr, target_len) {
            Some(a) => a,
            None => return false,
        };
        let method = match read_string(method_ptr, method_len) {
            Some(s) => s,
            None => return false,
        };
        let arguments = read_bytes(args_ptr, args_len).unwrap_or_default();
        let title = match read_string(title_ptr, title_len) {
            Some(s) => s,
            None => return false,
        };
        let id = match next_proposal_id(&ctx) {
            Some(i) => i,
            None => return false,
        };
        let proposal = Proposal {
            id,
            proposer: proposer.clone(),
            target,
            method,
            arguments,
            title: title.clone(),
            description: String::new(),
            start_time,
            end_time,
            yes_votes: 0,
            no_votes: 0,
            executed: false,
        };
        if store_proposal(&ctx, id, &proposal).is_err() {
            return false;
        }
        let _ = (ProposalCreatedEvt {
            proposal_id: NeoInteger::new(id),
            proposer,
            title: NeoString::from_str(&title),
        })
        .emit();
        true
    }

    /// Cast a vote on a proposal. `side`: 0 = yes, 1 = no.
    ///
    /// Validates that the current block time falls within the proposal's
    /// voting window (`start_time..=end_time`).
    #[neo_method]
    pub fn vote(
        proposal_id: i64,
        voter_ptr: i64,
        voter_len: i64,
        side: i64,
        weight: i64,
    ) -> bool {
        if weight <= 0 || !(0..=1).contains(&side) {
            return false;
        }
        let voter = match read_address(voter_ptr, voter_len) {
            Some(a) => a,
            None => return false,
        };
        if !ensure_witness(&voter) {
            return false;
        }
        let ctx = match storage_context() {
            Some(c) => c,
            None => return false,
        };
        if has_voted(&ctx, proposal_id, &voter) {
            return false;
        }
        let stake = load_stake(&ctx, &voter);
        if stake <= 0 || weight > stake {
            return false;
        }
        let mut proposal = match load_proposal(&ctx, proposal_id) {
            Some(p) => p,
            None => return false,
        };
        if proposal.executed {
            return false;
        }
        // Enforce voting window: reject votes before start or after end.
        let now: i64 = NeoRuntime::get_time()
            .map(|t| t.try_as_i64().unwrap_or(0))
            .unwrap_or(0);
        if now < proposal.start_time || now > proposal.end_time {
            return false;
        }
        let support = side == 0;
        if support {
            proposal.yes_votes += weight;
        } else {
            proposal.no_votes += weight;
        }
        if store_proposal(&ctx, proposal_id, &proposal).is_err() {
            return false;
        }
        if record_vote(&ctx, proposal_id, &voter).is_err() {
            return false;
        }
        let _ = (VoteCastEvt {
            proposal_id: NeoInteger::new(proposal_id),
            voter,
            support: NeoBoolean::new(support),
            weight: NeoInteger::new(weight),
        })
        .emit();
        true
    }

    /// Execute a proposal if quorum is met and yes > no.
    #[neo_method]
    pub fn execute(proposal_id: i64) -> bool {
        let ctx = match storage_context() {
            Some(c) => c,
            None => return false,
        };
        let cfg = match load_config(&ctx) {
            Some(c) => c,
            None => return false,
        };
        let mut proposal = match load_proposal(&ctx, proposal_id) {
            Some(p) => p,
            None => return false,
        };
        if proposal.executed {
            return false;
        }
        let total_votes = proposal.yes_votes + proposal.no_votes;
        if total_votes < cfg.quorum || proposal.yes_votes <= proposal.no_votes {
            return false;
        }
        if execute_proposal(&proposal.target, &proposal.method, &proposal.arguments).is_err() {
            return false;
        }
        proposal.executed = true;
        if store_proposal(&ctx, proposal_id, &proposal).is_err() {
            return false;
        }
        let _ = (ProposalExecutedEvt {
            proposal_id: NeoInteger::new(proposal_id),
        })
        .emit();
        true
    }

    /// Unstake tokens from the DAO.
    ///
    /// Transfer is attempted BEFORE updating storage to prevent
    /// state corruption if the external transfer call fails.
    #[neo_method]
    pub fn unstake(staker_ptr: i64, staker_len: i64, amount: i64) -> bool {
        if amount <= 0 {
            return false;
        }
        let staker = match read_address(staker_ptr, staker_len) {
            Some(a) => a,
            None => return false,
        };
        if !ensure_witness(&staker) {
            return false;
        }
        let ctx = match storage_context() {
            Some(c) => c,
            None => return false,
        };
        let cfg = match load_config(&ctx) {
            Some(c) => c,
            None => return false,
        };
        let current = load_stake(&ctx, &staker);
        if current < amount {
            return false;
        }
        // Transfer tokens FIRST — only update storage on success to prevent
        // accounting corruption if the external call fails.
        let contract_hash = NeoRuntime::get_executing_script_hash().unwrap_or_default();
        if !call_transfer(&cfg.token, &contract_hash, &staker, amount) {
            return false;
        }
        let new_total = current - amount;
        if store_stake(&ctx, &staker, new_total).is_err() {
            return false;
        }
        let _ = (StakeDecreasedEvt {
            staker,
            amount: NeoInteger::new(amount),
            new_total: NeoInteger::new(new_total),
        })
        .emit();
        true
    }

    /// Return the stake balance for a given address.
    #[neo_method(name = "stakeOf")]
    pub fn stake_of(staker_ptr: i64, staker_len: i64) -> i64 {
        let staker = match read_address(staker_ptr, staker_len) {
            Some(a) => a,
            None => return 0,
        };
        let ctx = match storage_context() {
            Some(c) => c,
            None => return 0,
        };
        load_stake(&ctx, &staker)
    }

    /// Return the DAO configuration via notify event.
    #[neo_method(name = "getConfig")]
    pub fn get_config() {
        let ctx = match storage_context() {
            Some(c) => c,
            None => return,
        };
        let cfg = match load_config(&ctx) {
            Some(c) => c,
            None => return,
        };
        let label = NeoString::from_str("getConfig");
        let mut state = NeoArray::new();
        state.push(NeoValue::from(serialize_value(&cfg)));
        let _ = NeoRuntime::notify(&label, &state);
    }

    /// Return a proposal's data via notify event.
    #[neo_method(name = "getProposal")]
    pub fn get_proposal(proposal_id: i64) {
        let ctx = match storage_context() {
            Some(c) => c,
            None => return,
        };
        let p = match load_proposal(&ctx, proposal_id) {
            Some(p) => p,
            None => return,
        };
        let label = NeoString::from_str("getProposal");
        let mut state = NeoArray::new();
        state.push(NeoValue::from(serialize_value(&p)));
        let _ = NeoRuntime::notify(&label, &state);
    }

    /// Handle incoming NEP-17 token payments as stake deposits.
    ///
    /// Only accepts the configured governance token; rejects payments
    /// from any other NEP-17 contract.
    #[neo_method(name = "onNEP17Payment")]
    pub fn on_nep17_payment(from_ptr: i64, from_len: i64, amount: i64, _data: i64) {
        if amount <= 0 {
            return;
        }
        let from = match read_address(from_ptr, from_len) {
            Some(a) => a,
            None => return,
        };
        let ctx = match storage_context() {
            Some(c) => c,
            None => return,
        };
        // Verify the calling contract is the configured governance token.
        let cfg = match load_config(&ctx) {
            Some(c) => c,
            None => return,
        };
        let caller = NeoRuntime::get_calling_script_hash()
            .unwrap_or_else(|_| NeoByteString::new(vec![]));
        if caller.as_slice() != cfg.token.as_slice() {
            return;
        }
        let current = load_stake(&ctx, &from);
        let new_total = current + amount;
        let _ = store_stake(&ctx, &from, new_total);
        let _ = (StakeIncreasedEvt {
            staker: from,
            amount: NeoInteger::new(amount),
            new_total: NeoInteger::new(new_total),
        })
        .emit();
    }
}

impl Default for NeoGovernanceDaoContract {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    // Integration tests require NeoVM runtime stubs.
}
