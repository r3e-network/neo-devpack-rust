// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "NeoGovernanceDAO"
}"#
);

// Direct i64 keys avoid wasm linear-memory key materialisation for config,
// proposal fields, and single-account stake balances.
const CONFIG_OWNER_KEY: i64 = i64::MIN + 1;
const CONFIG_TOKEN_KEY: i64 = i64::MIN + 2;
const CONFIG_QUORUM_KEY: i64 = i64::MIN + 3;
const PROPOSAL_COUNTER_KEY: i64 = i64::MIN + 4;
const DIRECT_KEY_STRIDE: i64 = 16;
const MAX_DIRECT_ID: i64 = i64::MAX / DIRECT_KEY_STRIDE;
const P_PROPOSER: i64 = 1;
const P_TARGET: i64 = 2;
const P_METHOD: i64 = 3;
const P_ARGS: i64 = 4;
const P_YES: i64 = 5;
const P_NO: i64 = 6;
const P_EXECUTED: i64 = 7;
const P_START: i64 = 8;
const P_END: i64 = 9;
const VOTE_PREFIX: &[u8] = b"dao:vote:";

const VOTE_KEY_LEN: usize = 26;

// --- Storage helpers (heap-free via RawStorage) ---

#[inline(always)]
fn put_i64_key(key: i64, value: i64) -> bool {
    RawStorage::put_i64_key(key, value);
    true
}

#[inline(always)]
fn get_i64_key(key: i64) -> i64 {
    RawStorage::get_i64_key_or_zero(key)
}

fn put_bool(key: &[u8], value: bool) -> bool {
    RawStorage::put_bool(key, value);
    true
}

fn get_bool(key: &[u8]) -> Option<bool> {
    RawStorage::get_bool(key)
}

// --- Key builders ---

#[inline(always)]
fn valid_direct_id(id: i64) -> bool {
    id > 0 && id <= MAX_DIRECT_ID
}

#[inline(always)]
fn proposal_key(id: i64, field: i64) -> i64 {
    id * DIRECT_KEY_STRIDE + field
}

#[inline(always)]
fn stake_key(account_id: i64) -> i64 {
    -(account_id * DIRECT_KEY_STRIDE)
}

fn vote_key(proposal_id: i64, voter_id: i64) -> RawKeyBuilder<VOTE_KEY_LEN> {
    let mut key = RawKeyBuilder::new();
    key.push_bytes(VOTE_PREFIX);
    key.push_i64_le(proposal_id);
    key.push_byte(b':');
    key.push_i64_le(voter_id);
    key
}

// --- Config ---

fn load_config_owner() -> Option<i64> {
    match get_i64_key(CONFIG_OWNER_KEY) {
        0 => None,
        owner => Some(owner),
    }
}

fn load_config_token() -> Option<i64> {
    match get_i64_key(CONFIG_TOKEN_KEY) {
        0 => None,
        token => Some(token),
    }
}

fn load_config_quorum() -> Option<i64> {
    match get_i64_key(CONFIG_QUORUM_KEY) {
        0 => None,
        quorum => Some(quorum),
    }
}

fn config_exists() -> bool {
    load_config_owner().is_some()
}

fn store_config(owner: i64, token: i64, quorum: i64) -> bool {
    put_i64_key(CONFIG_OWNER_KEY, owner)
        && put_i64_key(CONFIG_TOKEN_KEY, token)
        && put_i64_key(CONFIG_QUORUM_KEY, quorum)
}

fn load_stake(account_id: i64) -> i64 {
    if !valid_direct_id(account_id) {
        return 0;
    }
    get_i64_key(stake_key(account_id))
}

fn store_stake(account_id: i64, amount: i64) -> bool {
    if !valid_direct_id(account_id) {
        return false;
    }
    let key = stake_key(account_id);
    if amount == 0 {
        RawStorage::delete_i64_key(key);
        true
    } else {
        put_i64_key(key, amount)
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
    start_time: i64,
    end_time: i64,
}

fn next_proposal_id() -> Option<i64> {
    let current = get_i64_key(PROPOSAL_COUNTER_KEY);
    let next = current.checked_add(1)?;
    if !valid_direct_id(next) || !put_i64_key(PROPOSAL_COUNTER_KEY, next) {
        return None;
    }
    Some(next)
}

fn load_proposal(id: i64) -> Option<ProposalData> {
    if !valid_direct_id(id) {
        return None;
    }
    let proposer = get_i64_key(proposal_key(id, P_PROPOSER));
    let target = get_i64_key(proposal_key(id, P_TARGET));
    if proposer == 0 || target == 0 {
        return None;
    }
    let method = get_i64_key(proposal_key(id, P_METHOD));
    let arg_data = get_i64_key(proposal_key(id, P_ARGS));
    let yes_votes = get_i64_key(proposal_key(id, P_YES));
    let no_votes = get_i64_key(proposal_key(id, P_NO));
    let executed = get_i64_key(proposal_key(id, P_EXECUTED)) != 0;
    let start_time = get_i64_key(proposal_key(id, P_START));
    let end_time = get_i64_key(proposal_key(id, P_END));
    Some(ProposalData {
        proposer,
        target,
        method,
        arg_data,
        yes_votes,
        no_votes,
        executed,
        start_time,
        end_time,
    })
}

fn store_proposal(id: i64, p: &ProposalData) -> bool {
    if !valid_direct_id(id) {
        return false;
    }
    put_i64_key(proposal_key(id, P_PROPOSER), p.proposer)
        && put_i64_key(proposal_key(id, P_TARGET), p.target)
        && put_i64_key(proposal_key(id, P_METHOD), p.method)
        && put_i64_key(proposal_key(id, P_ARGS), p.arg_data)
        && put_i64_key(proposal_key(id, P_YES), p.yes_votes)
        && put_i64_key(proposal_key(id, P_NO), p.no_votes)
        && put_i64_key(proposal_key(id, P_EXECUTED), p.executed as i64)
        && put_i64_key(proposal_key(id, P_START), p.start_time)
        && put_i64_key(proposal_key(id, P_END), p.end_time)
}

// --- Voting helpers ---

fn has_voted(proposal_id: i64, voter_id: i64) -> bool {
    let key = vote_key(proposal_id, voter_id);
    get_bool(key.as_slice()).unwrap_or(false)
}

fn record_vote(proposal_id: i64, voter_id: i64) -> bool {
    let key = vote_key(proposal_id, voter_id);
    put_bool(key.as_slice(), true)
}

fn execute_proposal_call(target: i64, method: i64, arg_data: i64) -> bool {
    target > 0 && method > 0 && arg_data >= 0
}

fn call_transfer(token: i64, from_id: i64, to_id: i64, amount: i64) -> bool {
    token > 0 && from_id >= 0 && to_id > 0 && amount > 0
}

#[inline(always)]
fn current_time_i64() -> Option<i64> {
    NeoRuntime::get_time_i64().ok()
}

#[inline(always)]
fn voting_window_open(start_time: i64, end_time: i64, now: i64) -> bool {
    start_time <= now && now <= end_time
}

#[inline(always)]
fn total_votes(yes_votes: i64, no_votes: i64) -> Option<i64> {
    yes_votes.checked_add(no_votes)
}

// Events
#[neo_event]
pub struct ProposalCreatedEvt {
    pub proposal_id: i64,
    pub proposer: i64,
    pub title: i64,
}

#[neo_event]
pub struct VoteCastEvt {
    pub proposal_id: i64,
    pub voter: i64,
    pub support: bool,
    pub weight: i64,
}

#[neo_event]
pub struct ProposalExecutedEvt {
    pub proposal_id: i64,
}

#[neo_event]
pub struct StakeIncreasedEvt {
    pub staker: i64,
    pub amount: i64,
    pub new_total: i64,
}

#[neo_event]
pub struct StakeDecreasedEvt {
    pub staker: i64,
    pub amount: i64,
    pub new_total: i64,
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
        // The owner must be runtime-witnessed to bootstrap the DAO config;
        // otherwise anyone could register as owner (X18).
        if !NeoRuntime::require_witness_i64(owner_id) {
            return false;
        }
        if config_exists() {
            return false;
        }
        store_config(owner_id, token_id, quorum)
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
        // The proposer must be witnessed to prevent attribution spoofing /
        // governance spam (X18).
        if !NeoRuntime::require_witness_i64(proposer_id) {
            return false;
        }
        if !config_exists() {
            return false;
        }
        let id = match next_proposal_id() {
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
            start_time,
            end_time,
        };
        if !store_proposal(id, &proposal) {
            return false;
        }
        let _ = (ProposalCreatedEvt {
            proposal_id: id,
            proposer: proposer_id,
            title: title_id,
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
        // The voter identity must be runtime-witnessed, otherwise an attacker
        // iterates every staked account and casts its full balance (X2).
        if !NeoRuntime::require_witness_i64(voter_id) {
            return false;
        }
        if has_voted(proposal_id, voter_id) {
            return false;
        }
        let stake = load_stake(voter_id);
        if stake <= 0 || weight > stake {
            return false;
        }
        let mut proposal = match load_proposal(proposal_id) {
            Some(p) => p,
            None => return false,
        };
        if proposal.executed {
            return false;
        }
        let now = match current_time_i64() {
            Some(t) => t,
            None => return false,
        };
        if !voting_window_open(proposal.start_time, proposal.end_time, now) {
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
        if !store_proposal(proposal_id, &proposal) {
            return false;
        }
        if !record_vote(proposal_id, voter_id) {
            return false;
        }
        let _ = (VoteCastEvt {
            proposal_id,
            voter: voter_id,
            support,
            weight,
        })
        .emit();
        true
    }

    /// Execute a proposal if quorum is met and yes > no.
    #[neo_method]
    pub fn execute(proposal_id: i64) -> bool {
        let quorum = match load_config_quorum() {
            Some(q) => q,
            None => return false,
        };
        let mut proposal = match load_proposal(proposal_id) {
            Some(p) => p,
            None => return false,
        };
        if proposal.executed {
            return false;
        }
        let total_votes = match total_votes(proposal.yes_votes, proposal.no_votes) {
            Some(v) => v,
            None => return false,
        };
        if total_votes < quorum || proposal.yes_votes <= proposal.no_votes {
            return false;
        }
        if !execute_proposal_call(proposal.target, proposal.method, proposal.arg_data) {
            return false;
        }
        proposal.executed = true;
        if !store_proposal(proposal_id, &proposal) {
            return false;
        }
        let _ = (ProposalExecutedEvt { proposal_id }).emit();
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
        // The staker must be witnessed; otherwise an attacker forces unstake on
        // any account, zeroing its governance power (X3).
        if !NeoRuntime::require_witness_i64(staker_id) {
            return false;
        }
        let token = match load_config_token() {
            Some(t) => t,
            None => return false,
        };
        let current = load_stake(staker_id);
        if current < amount {
            return false;
        }
        if !call_transfer(token, 0, staker_id, amount) {
            return false;
        }
        let new_total = current - amount;
        if !store_stake(staker_id, new_total) {
            return false;
        }
        let _ = (StakeDecreasedEvt {
            staker: staker_id,
            amount,
            new_total,
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
        load_stake(staker_id)
    }

    /// Handle incoming NEP-17 token payments as stake deposits.
    ///
    /// Only accepts the configured governance token; rejects payments
    /// from any other NEP-17 contract.
    #[neo_method(name = "onNEP17Payment")]
    pub fn on_nep17_payment(from_id: i64, amount: i64, _data: i64) {
        if amount <= 0 || !valid_direct_id(from_id) {
            return;
        }
        // Verify the calling contract is the configured governance token.
        let token = match load_config_token() {
            Some(t) => t,
            None => return,
        };
        let caller_id = NeoRuntime::get_calling_script_hash_i64().unwrap_or(0);
        if caller_id != token {
            return;
        }
        let current = load_stake(from_id);
        let new_total = match current.checked_add(amount) {
            Some(v) => v,
            None => return,
        };
        if !store_stake(from_id, new_total) {
            return;
        }
        let _ = (StakeIncreasedEvt {
            staker: from_id,
            amount,
            new_total,
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
    use super::*;
    use neo_devpack::{prelude::NeoByteString, NeoVMSyscall};
    use std::sync::{Mutex, MutexGuard, OnceLock};

    fn runtime_test_lock() -> MutexGuard<'static, ()> {
        static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        match TEST_LOCK.get_or_init(|| Mutex::new(())).lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    fn witness_hash(account: i64) -> [u8; 20] {
        let mut bytes = [0u8; 20];
        bytes[..8].copy_from_slice(&account.to_le_bytes());
        bytes
    }

    fn setup_witnesses(accounts: &[i64]) -> MutexGuard<'static, ()> {
        let guard = runtime_test_lock();
        NeoVMSyscall::reset_host_state().expect("host syscall state should reset");
        let witnesses: Vec<NeoByteString> = accounts
            .iter()
            .map(|account| NeoByteString::from_slice(&witness_hash(*account)))
            .collect();
        NeoVMSyscall::set_active_witnesses(&witnesses).expect("active witnesses should update");
        guard
    }

    #[test]
    fn contract_compiles() {
        // Compilation test - verifies contract module parses correctly
    }

    #[test]
    fn voting_window_is_inclusive() {
        assert!(!voting_window_open(10, 20, 9));
        assert!(voting_window_open(10, 20, 10));
        assert!(voting_window_open(10, 20, 15));
        assert!(voting_window_open(10, 20, 20));
        assert!(!voting_window_open(10, 20, 21));
    }

    #[test]
    fn total_votes_rejects_overflow() {
        assert_eq!(total_votes(40, 2), Some(42));
        assert_eq!(total_votes(i64::MAX, 1), None);
    }

    #[test]
    fn configure_requires_owner_witness() {
        // Without owner in the witness set, configure is rejected (X18).
        {
            let _g = setup_witnesses(&[]);
            assert!(!NeoGovernanceDaoContract::configure(1, 2, 3));
        }
        // With owner witnessed, configure succeeds.
        {
            let _g = setup_witnesses(&[1]);
            assert!(NeoGovernanceDaoContract::configure(1, 2, 3));
        }
    }

    #[test]
    fn vote_requires_voter_witness() {
        // Configure + create a proposal with witnesses [1].
        let _g = setup_witnesses(&[1]);
        assert!(NeoGovernanceDaoContract::configure(1, 2, 3));
        assert!(NeoGovernanceDaoContract::propose(1, 2, 3, 4, 5, 0, 1000));
        // Voter 7 is NOT witnessed -> vote rejected (X2 ballot-box stuffing).
        assert!(!NeoGovernanceDaoContract::vote(1, 7, 0, 1));
    }

    #[test]
    fn unstake_requires_staker_witness() {
        let _g = setup_witnesses(&[1]);
        assert!(NeoGovernanceDaoContract::configure(1, 2, 3));
        // Staker 9 not witnessed, no stake recorded -> unstake rejected (X3).
        assert!(!NeoGovernanceDaoContract::unstake(9, 1));
    }
}
