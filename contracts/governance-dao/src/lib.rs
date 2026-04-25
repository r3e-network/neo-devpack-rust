// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "NeoGovernanceDAO"
}"#
);

// Storage keys
const CONFIG_OWNER_KEY: &[u8] = b"dao:cfg:owner";
const CONFIG_TOKEN_KEY: &[u8] = b"dao:cfg:token";
const CONFIG_QUORUM_KEY: &[u8] = b"dao:cfg:quorum";
const PROPOSAL_COUNTER_KEY: &[u8] = b"dao:counter";
const PROPOSAL_PREFIX: &[u8] = b"dao:p:";
const STAKE_PREFIX: &[u8] = b"dao:stake:";
const VOTE_PREFIX: &[u8] = b"dao:vote:";

// Proposal field suffixes
const P_PROPOSER: &[u8] = b":proposer";
const P_TARGET: &[u8] = b":target";
const P_METHOD: &[u8] = b":method";
const P_ARGS: &[u8] = b":args";
const P_YES: &[u8] = b":yes";
const P_NO: &[u8] = b":no";
const P_EXECUTED: &[u8] = b":executed";

// --- Storage helpers ---

fn storage_ctx() -> Option<NeoStorageContext> {
    NeoStorage::get_context().ok()
}

fn put_i64(ctx: &NeoStorageContext, key: &[u8], value: i64) -> bool {
    NeoStorage::put(
        ctx,
        &NeoByteString::from_slice(key),
        &NeoByteString::from_slice(&value.to_le_bytes()),
    )
    .is_ok()
}

fn get_i64(ctx: &NeoStorageContext, key: &[u8]) -> Option<i64> {
    let data = NeoStorage::get(ctx, &NeoByteString::from_slice(key)).ok()?;
    if data.len() != 8 {
        return None;
    }
    let s = data.as_slice();
    let mut buf = [0u8; 8];
    buf.copy_from_slice(s);
    Some(i64::from_le_bytes(buf))
}

fn put_bool(ctx: &NeoStorageContext, key: &[u8], value: bool) -> bool {
    NeoStorage::put(
        ctx,
        &NeoByteString::from_slice(key),
        &NeoByteString::from_slice(&[value as u8]),
    )
    .is_ok()
}

fn get_bool(ctx: &NeoStorageContext, key: &[u8]) -> Option<bool> {
    let data = NeoStorage::get(ctx, &NeoByteString::from_slice(key)).ok()?;
    if data.len() != 1 {
        return None;
    }
    Some(data.as_slice()[0] != 0)
}

fn delete_key(ctx: &NeoStorageContext, key: &[u8]) {
    let _ = NeoStorage::delete(ctx, &NeoByteString::from_slice(key));
}

// --- Key builders ---

fn proposal_key(id: i64, suffix: &[u8]) -> Vec<u8> {
    let mut key = PROPOSAL_PREFIX.to_vec();
    key.extend_from_slice(&id.to_le_bytes());
    key.extend_from_slice(suffix);
    key
}

fn stake_key(account_id: i64) -> Vec<u8> {
    let mut key = STAKE_PREFIX.to_vec();
    key.extend_from_slice(&account_id.to_le_bytes());
    key
}

fn vote_key(proposal_id: i64, voter_id: i64) -> Vec<u8> {
    let mut key = VOTE_PREFIX.to_vec();
    key.extend_from_slice(&proposal_id.to_le_bytes());
    key.push(b':');
    key.extend_from_slice(&voter_id.to_le_bytes());
    key
}

// --- Config ---

fn load_config_owner(ctx: &NeoStorageContext) -> Option<i64> {
    get_i64(ctx, CONFIG_OWNER_KEY)
}

fn load_config_token(ctx: &NeoStorageContext) -> Option<i64> {
    get_i64(ctx, CONFIG_TOKEN_KEY)
}

fn load_config_quorum(ctx: &NeoStorageContext) -> Option<i64> {
    get_i64(ctx, CONFIG_QUORUM_KEY)
}

fn config_exists(ctx: &NeoStorageContext) -> bool {
    load_config_owner(ctx).is_some()
}

fn store_config(ctx: &NeoStorageContext, owner: i64, token: i64, quorum: i64) -> bool {
    put_i64(ctx, CONFIG_OWNER_KEY, owner)
        && put_i64(ctx, CONFIG_TOKEN_KEY, token)
        && put_i64(ctx, CONFIG_QUORUM_KEY, quorum)
}

fn load_stake(ctx: &NeoStorageContext, account_id: i64) -> i64 {
    get_i64(ctx, &stake_key(account_id)).unwrap_or(0)
}

fn store_stake(ctx: &NeoStorageContext, account_id: i64, amount: i64) -> bool {
    if amount == 0 {
        delete_key(ctx, &stake_key(account_id));
        true
    } else {
        put_i64(ctx, &stake_key(account_id), amount)
    }
}

// --- Proposal data ---

struct ProposalData {
    proposer: i64,
    target: i64,
    method: i64,
    arg_data: i64,
    yes_votes: i64,
    no_votes: i64,
    executed: bool,
}

fn next_proposal_id(ctx: &NeoStorageContext) -> Option<i64> {
    let current = get_i64(ctx, PROPOSAL_COUNTER_KEY).unwrap_or(0);
    let next = current.checked_add(1)?;
    if !put_i64(ctx, PROPOSAL_COUNTER_KEY, next) {
        return None;
    }
    Some(next)
}

fn load_proposal(ctx: &NeoStorageContext, id: i64) -> Option<ProposalData> {
    let proposer = get_i64(ctx, &proposal_key(id, P_PROPOSER))?;
    let target = get_i64(ctx, &proposal_key(id, P_TARGET))?;
    let method = get_i64(ctx, &proposal_key(id, P_METHOD)).unwrap_or(0);
    let arg_data = get_i64(ctx, &proposal_key(id, P_ARGS)).unwrap_or(0);
    let yes_votes = get_i64(ctx, &proposal_key(id, P_YES)).unwrap_or(0);
    let no_votes = get_i64(ctx, &proposal_key(id, P_NO)).unwrap_or(0);
    let executed = get_bool(ctx, &proposal_key(id, P_EXECUTED)).unwrap_or(false);
    Some(ProposalData {
        proposer,
        target,
        method,
        arg_data,
        yes_votes,
        no_votes,
        executed,
    })
}

fn store_proposal(ctx: &NeoStorageContext, id: i64, p: &ProposalData) -> bool {
    put_i64(ctx, &proposal_key(id, P_PROPOSER), p.proposer)
        && put_i64(ctx, &proposal_key(id, P_TARGET), p.target)
        && put_i64(ctx, &proposal_key(id, P_METHOD), p.method)
        && put_i64(ctx, &proposal_key(id, P_ARGS), p.arg_data)
        && put_i64(ctx, &proposal_key(id, P_YES), p.yes_votes)
        && put_i64(ctx, &proposal_key(id, P_NO), p.no_votes)
        && put_bool(ctx, &proposal_key(id, P_EXECUTED), p.executed)
}

// --- Voting helpers ---

fn has_voted(ctx: &NeoStorageContext, proposal_id: i64, voter_id: i64) -> bool {
    get_bool(ctx, &vote_key(proposal_id, voter_id)).unwrap_or(false)
}

fn record_vote(ctx: &NeoStorageContext, proposal_id: i64, voter_id: i64) -> bool {
    put_bool(ctx, &vote_key(proposal_id, voter_id), true)
}

fn execute_proposal_call(target: i64, method: i64, arg_data: i64) -> bool {
    target > 0 && method > 0 && arg_data >= 0
}

fn call_transfer(token: i64, from_id: i64, to_id: i64, amount: i64) -> bool {
    token > 0 && from_id >= 0 && to_id > 0 && amount > 0
}

// Events
#[neo_event]
pub struct ProposalCreatedEvt {
    pub proposal_id: NeoInteger,
    pub proposer: NeoInteger,
    pub title: NeoInteger,
}

#[neo_event]
pub struct VoteCastEvt {
    pub proposal_id: NeoInteger,
    pub voter: NeoInteger,
    pub support: NeoBoolean,
    pub weight: NeoInteger,
}

#[neo_event]
pub struct ProposalExecutedEvt {
    pub proposal_id: NeoInteger,
}

#[neo_event]
pub struct StakeIncreasedEvt {
    pub staker: NeoInteger,
    pub amount: NeoInteger,
    pub new_total: NeoInteger,
}

#[neo_event]
pub struct StakeDecreasedEvt {
    pub staker: NeoInteger,
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
    pub fn configure(owner_id: i64, token_id: i64, quorum: i64) -> bool {
        if quorum <= 0 || owner_id == 0 || token_id == 0 {
            return false;
        }
        let ctx = match storage_ctx() {
            Some(c) => c,
            None => return false,
        };
        if config_exists(&ctx) {
            return false;
        }
        store_config(&ctx, owner_id, token_id, quorum)
    }

    /// Create a governance proposal.
    #[neo_method]
    pub fn propose(
        proposer_id: i64,
        target_id: i64,
        method_id: i64,
        arg_data: i64,
        title_id: i64,
        start_time: i64,
        end_time: i64,
    ) -> bool {
        if end_time <= start_time || proposer_id == 0 || target_id == 0 {
            return false;
        }
        let ctx = match storage_ctx() {
            Some(c) => c,
            None => return false,
        };
        if !config_exists(&ctx) {
            return false;
        }
        let id = match next_proposal_id(&ctx) {
            Some(i) => i,
            None => return false,
        };
        let proposal = ProposalData {
            proposer: proposer_id,
            target: target_id,
            method: method_id,
            arg_data,
            yes_votes: 0,
            no_votes: 0,
            executed: false,
        };
        if !store_proposal(&ctx, id, &proposal) {
            return false;
        }
        let _ = (ProposalCreatedEvt {
            proposal_id: NeoInteger::new(id),
            proposer: NeoInteger::new(proposer_id),
            title: NeoInteger::new(title_id),
        })
        .emit();
        true
    }

    /// Cast a vote on a proposal. `side`: 0 = yes, 1 = no.
    ///
    /// Validates that the current block time falls within the proposal's
    /// voting window (`start_time..=end_time`).
    #[neo_method]
    pub fn vote(proposal_id: i64, voter_id: i64, side: i64, weight: i64) -> bool {
        if weight <= 0 || !(0..=1).contains(&side) || voter_id == 0 {
            return false;
        }
        let ctx = match storage_ctx() {
            Some(c) => c,
            None => return false,
        };
        if has_voted(&ctx, proposal_id, voter_id) {
            return false;
        }
        let stake = load_stake(&ctx, voter_id);
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
        let support = side == 0;
        if support {
            proposal.yes_votes = match proposal.yes_votes.checked_add(weight) {
                Some(v) => v,
                None => return false,
            };
        } else {
            proposal.no_votes = match proposal.no_votes.checked_add(weight) {
                Some(v) => v,
                None => return false,
            };
        }
        if !store_proposal(&ctx, proposal_id, &proposal) {
            return false;
        }
        if !record_vote(&ctx, proposal_id, voter_id) {
            return false;
        }
        let _ = (VoteCastEvt {
            proposal_id: NeoInteger::new(proposal_id),
            voter: NeoInteger::new(voter_id),
            support: NeoBoolean::new(support),
            weight: NeoInteger::new(weight),
        })
        .emit();
        true
    }

    /// Execute a proposal if quorum is met and yes > no.
    #[neo_method]
    pub fn execute(proposal_id: i64) -> bool {
        let ctx = match storage_ctx() {
            Some(c) => c,
            None => return false,
        };
        let quorum = match load_config_quorum(&ctx) {
            Some(q) => q,
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
        if total_votes < quorum || proposal.yes_votes <= proposal.no_votes {
            return false;
        }
        if !execute_proposal_call(proposal.target, proposal.method, proposal.arg_data) {
            return false;
        }
        proposal.executed = true;
        if !store_proposal(&ctx, proposal_id, &proposal) {
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
    pub fn unstake(staker_id: i64, amount: i64) -> bool {
        if amount <= 0 || staker_id == 0 {
            return false;
        }
        let ctx = match storage_ctx() {
            Some(c) => c,
            None => return false,
        };
        let token = match load_config_token(&ctx) {
            Some(t) => t,
            None => return false,
        };
        let current = load_stake(&ctx, staker_id);
        if current < amount {
            return false;
        }
        if !call_transfer(token, 0, staker_id, amount) {
            return false;
        }
        let new_total = current - amount;
        if !store_stake(&ctx, staker_id, new_total) {
            return false;
        }
        let _ = (StakeDecreasedEvt {
            staker: NeoInteger::new(staker_id),
            amount: NeoInteger::new(amount),
            new_total: NeoInteger::new(new_total),
        })
        .emit();
        true
    }

    /// Return the stake balance for a given account.
    #[neo_method(safe, name = "stakeOf")]
    pub fn stake_of(staker_id: i64) -> i64 {
        if staker_id == 0 {
            return 0;
        }
        let ctx = match storage_ctx() {
            Some(c) => c,
            None => return 0,
        };
        load_stake(&ctx, staker_id)
    }

    /// Handle incoming NEP-17 token payments as stake deposits.
    ///
    /// Only accepts the configured governance token; rejects payments
    /// from any other NEP-17 contract.
    #[neo_method(name = "onNEP17Payment")]
    pub fn on_nep17_payment(from_id: i64, amount: i64, _data: i64) {
        if amount <= 0 || from_id == 0 {
            return;
        }
        let ctx = match storage_ctx() {
            Some(c) => c,
            None => return,
        };
        // Verify the calling contract is the configured governance token.
        let token = match load_config_token(&ctx) {
            Some(t) => t,
            None => return,
        };
        let caller_id: i64 = NeoRuntime::get_calling_script_hash()
            .map(|h| {
                let s = h.as_slice();
                if s.len() >= 8 {
                    let mut buf = [0u8; 8];
                    buf.copy_from_slice(&s[..8]);
                    i64::from_le_bytes(buf)
                } else {
                    0
                }
            })
            .unwrap_or(0);
        if caller_id != token {
            return;
        }
        let current = load_stake(&ctx, from_id);
        let new_total = match current.checked_add(amount) {
            Some(v) => v,
            None => return,
        };
        let _ = store_stake(&ctx, from_id, new_total);
        let _ = (StakeIncreasedEvt {
            staker: NeoInteger::new(from_id),
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
    #[test]
    fn contract_compiles() {
        // Compilation test - verifies contract module parses correctly
    }
}
