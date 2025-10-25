use core::slice;
use neo_devpack::{codec, prelude::*};
use serde::{Deserialize, Serialize};

const CONFIG_KEY: &[u8] = b"dao:config";
const PROPOSAL_COUNTER_KEY: &[u8] = b"dao:counter";
const PROPOSAL_PREFIX: &[u8] = b"dao:proposal:";
const STAKE_PREFIX: &[u8] = b"dao:stake:";
const VOTE_PREFIX: &[u8] = b"dao:vote:";

#[derive(Clone, Serialize, Deserialize)]
struct DaoConfig {
    owner: NeoByteString,
    token: NeoByteString,
    quorum: i64,
}

#[derive(Clone, Serialize, Deserialize)]
struct Proposal {
    id: i64,
    proposer: NeoByteString,
    target: NeoByteString,
    method: String,
    title: String,
    description: String,
    start_time: i64,
    end_time: i64,
    yes_votes: i64,
    no_votes: i64,
    executed: bool,
}

neo_manifest_overlay!(
    r#"{
    "name": "NeoGovernanceDAO",
    "features": { "storage": true }
}"#
);

#[neo_event]
pub struct ProposalCreated {
    pub proposal_id: NeoInteger,
    pub proposer: NeoByteString,
    pub title: NeoString,
}

#[neo_event]
pub struct VoteCast {
    pub proposal_id: NeoInteger,
    pub voter: NeoByteString,
    pub support: NeoBoolean,
    pub weight: NeoInteger,
}

#[neo_event]
pub struct ProposalExecuted {
    pub proposal_id: NeoInteger,
}

#[neo_event]
pub struct StakeIncreased {
    pub staker: NeoByteString,
    pub amount: NeoInteger,
    pub new_total: NeoInteger,
}

#[neo_event]
pub struct StakeDecreased {
    pub staker: NeoByteString,
    pub amount: NeoInteger,
    pub new_total: NeoInteger,
}

#[allow(improper_ctypes_definitions)]
#[neo_safe]
#[no_mangle]
pub extern "C" fn getConfig() -> NeoByteString {
    storage_context()
        .and_then(|ctx| load_config(&ctx))
        .map(|config| serialize_value(&config))
        .unwrap_or_else(|| NeoByteString::new(Vec::new()))
}

#[allow(improper_ctypes_definitions)]
#[neo_safe]
#[no_mangle]
pub extern "C" fn getProposal(proposal_id: i64) -> NeoByteString {
    storage_context()
        .and_then(|ctx| load_proposal(&ctx, proposal_id))
        .map(|proposal| serialize_value(&proposal))
        .unwrap_or_else(|| NeoByteString::new(Vec::new()))
}

#[neo_safe]
#[no_mangle]
pub extern "C" fn stakeOf(address_ptr: i64, address_len: i64) -> i64 {
    let Some(ctx) = storage_context() else {
        return 0;
    };
    let Some(address) = read_address(address_ptr, address_len) else {
        return 0;
    };
    load_stake(&ctx, &address)
}

#[no_mangle]
pub extern "C" fn configure(
    owner_ptr: i64,
    owner_len: i64,
    token_ptr: i64,
    token_len: i64,
    quorum: i64,
) -> i64 {
    if quorum <= 0 {
        return 0;
    }
    let Some(ctx) = storage_context() else {
        return 0;
    };
    if load_config(&ctx).is_some() {
        return 0;
    }
    let Some(owner) = read_address(owner_ptr, owner_len) else {
        return 0;
    };
    let Some(token) = read_address(token_ptr, token_len) else {
        return 0;
    };

    let config = DaoConfig {
        owner: owner.clone(),
        token: token.clone(),
        quorum,
    };

    if store_config(&ctx, &config).is_err() {
        return 0;
    }
    if store_to_storage(&ctx, PROPOSAL_COUNTER_KEY, &0i64).is_err() {
        return 0;
    }

    StakeIncreased {
        staker: owner,
        amount: NeoInteger::new(0),
        new_total: NeoInteger::new(0),
    }
    .emit()
    .ok();

    1
}

#[no_mangle]
pub extern "C" fn propose(
    proposer_ptr: i64,
    proposer_len: i64,
    target_ptr: i64,
    target_len: i64,
    method_ptr: i64,
    method_len: i64,
    title_ptr: i64,
    title_len: i64,
    description_ptr: i64,
    description_len: i64,
    start_time: i64,
    end_time: i64,
) -> i64 {
    if start_time >= end_time {
        return 0;
    }
    let Some(ctx) = storage_context() else {
        return 0;
    };
    let Some(config) = load_config(&ctx) else {
        return 0;
    };

    let Some(proposer) = read_address(proposer_ptr, proposer_len) else {
        return 0;
    };
    if !ensure_witness(&proposer) {
        return 0;
    }
    if load_stake(&ctx, &proposer) <= 0 {
        return 0;
    }

    let Some(target) = read_address(target_ptr, target_len) else {
        return 0;
    };
    let Some(method) = read_string(method_ptr, method_len) else {
        return 0;
    };
    if method.is_empty() {
        return 0;
    }
    let Some(title) = read_string(title_ptr, title_len) else {
        return 0;
    };
    let Some(description) = read_string(description_ptr, description_len) else {
        return 0;
    };

    let proposal_id = match next_proposal_id(&ctx) {
        Some(id) => id,
        None => return 0,
    };

    let proposal = Proposal {
        id: proposal_id,
        proposer: proposer.clone(),
        target: target.clone(),
        method: method.clone(),
        title: title.clone(),
        description,
        start_time,
        end_time,
        yes_votes: 0,
        no_votes: 0,
        executed: false,
    };

    if store_proposal(&ctx, proposal_id, &proposal).is_err() {
        return 0;
    }

    ProposalCreated {
        proposal_id: NeoInteger::new(proposal_id),
        proposer,
        title: NeoString::from_str(&title),
    }
    .emit()
    .ok();

    // silence unused warning for config
    let _ = config;

    proposal_id
}

#[no_mangle]
pub extern "C" fn vote(
    proposal_id: i64,
    voter_ptr: i64,
    voter_len: i64,
    support: i64,
    current_time: i64,
) -> i64 {
    if support != 0 && support != 1 {
        return 0;
    }
    let Some(ctx) = storage_context() else {
        return 0;
    };
    let Some(mut proposal) = load_proposal(&ctx, proposal_id) else {
        return 0;
    };
    if proposal.executed {
        return 0;
    }
    if current_time < proposal.start_time || current_time > proposal.end_time {
        return 0;
    }

    let Some(voter) = read_address(voter_ptr, voter_len) else {
        return 0;
    };
    if !ensure_witness(&voter) {
        return 0;
    }

    if has_voted(&ctx, proposal_id, &voter) {
        return 0;
    }

    let weight = load_stake(&ctx, &voter);
    if weight <= 0 {
        return 0;
    }

    if record_vote(&ctx, proposal_id, &voter).is_err() {
        return 0;
    }

    if support == 1 {
        proposal.yes_votes = proposal.yes_votes.saturating_add(weight);
    } else {
        proposal.no_votes = proposal.no_votes.saturating_add(weight);
    }

    if store_proposal(&ctx, proposal_id, &proposal).is_err() {
        return 0;
    }

    VoteCast {
        proposal_id: NeoInteger::new(proposal_id),
        voter,
        support: NeoBoolean::new(support == 1),
        weight: NeoInteger::new(weight),
    }
    .emit()
    .ok();

    1
}

#[no_mangle]
pub extern "C" fn execute(proposal_id: i64) -> i64 {
    let Some(ctx) = storage_context() else {
        return 0;
    };
    let Some(config) = load_config(&ctx) else {
        return 0;
    };
    let Some(mut proposal) = load_proposal(&ctx, proposal_id) else {
        return 0;
    };
    if proposal.executed {
        return 0;
    }
    if proposal.yes_votes < config.quorum || proposal.yes_votes <= proposal.no_votes {
        return 0;
    }

    let args = NeoArray::<NeoValue>::new();
    if NeoContractRuntime::call(
        &proposal.target,
        &NeoString::from_str(&proposal.method),
        &args,
    )
    .is_err()
    {
        return 0;
    }

    proposal.executed = true;
    if store_proposal(&ctx, proposal_id, &proposal).is_err() {
        return 0;
    }

    ProposalExecuted {
        proposal_id: NeoInteger::new(proposal_id),
    }
    .emit()
    .ok();

    1
}

#[no_mangle]
pub extern "C" fn unstake(address_ptr: i64, address_len: i64, amount: i64) -> i64 {
    if amount <= 0 {
        return 0;
    }
    let Some(ctx) = storage_context() else {
        return 0;
    };
    let Some(config) = load_config(&ctx) else {
        return 0;
    };
    let Some(address) = read_address(address_ptr, address_len) else {
        return 0;
    };
    if !ensure_witness(&address) {
        return 0;
    }

    let current = load_stake(&ctx, &address);
    if current < amount {
        return 0;
    }

    let new_total = current - amount;
    if store_stake(&ctx, &address, new_total).is_err() {
        return 0;
    }

    let contract_hash = match NeoRuntime::get_executing_script_hash() {
        Ok(hash) => hash,
        Err(_) => return 0,
    };

    if !call_transfer(&config.token, &contract_hash, &address, amount) {
        return 0;
    }

    StakeDecreased {
        staker: address,
        amount: NeoInteger::new(amount),
        new_total: NeoInteger::new(new_total),
    }
    .emit()
    .ok();

    1
}

#[allow(improper_ctypes_definitions)]
#[no_mangle]
pub extern "C" fn onNEP17Payment(from: NeoByteString, amount: i64, data: NeoByteString) {
    if amount <= 0 {
        return;
    }
    let Some(ctx) = storage_context() else {
        return;
    };
    let Some(config) = load_config(&ctx) else {
        return;
    };
    let Ok(call_hash) = NeoRuntime::get_calling_script_hash() else {
        return;
    };
    if !addresses_equal(&call_hash, &config.token) {
        return;
    }
    if !data.is_empty() && data.as_slice() != b"stake" {
        return;
    }

    let current = load_stake(&ctx, &from);
    let new_total = current.saturating_add(amount);
    if store_stake(&ctx, &from, new_total).is_err() {
        return;
    }

    StakeIncreased {
        staker: from,
        amount: NeoInteger::new(amount),
        new_total: NeoInteger::new(new_total),
    }
    .emit()
    .ok();
}

fn storage_context() -> Option<NeoStorageContext> {
    NeoStorage::get_context().ok()
}

fn load_config(ctx: &NeoStorageContext) -> Option<DaoConfig> {
    load_from_storage(ctx, CONFIG_KEY)
}

fn store_config(ctx: &NeoStorageContext, config: &DaoConfig) -> NeoResult<()> {
    store_to_storage(ctx, CONFIG_KEY, config)
}

fn proposal_key(id: i64) -> Vec<u8> {
    let mut key = PROPOSAL_PREFIX.to_vec();
    key.extend_from_slice(&id.to_le_bytes());
    key
}

fn load_proposal(ctx: &NeoStorageContext, id: i64) -> Option<Proposal> {
    load_from_storage(ctx, &proposal_key(id))
}

fn store_proposal(ctx: &NeoStorageContext, id: i64, proposal: &Proposal) -> NeoResult<()> {
    store_to_storage(ctx, &proposal_key(id), proposal)
}

fn next_proposal_id(ctx: &NeoStorageContext) -> Option<i64> {
    let current: i64 = load_from_storage(ctx, PROPOSAL_COUNTER_KEY).unwrap_or(0);
    let next = current.checked_add(1)?;
    store_to_storage(ctx, PROPOSAL_COUNTER_KEY, &next).ok()?;
    Some(next)
}

fn stake_key(address: &NeoByteString) -> Vec<u8> {
    let mut key = STAKE_PREFIX.to_vec();
    key.extend_from_slice(address.as_slice());
    key
}

fn load_stake(ctx: &NeoStorageContext, address: &NeoByteString) -> i64 {
    load_from_storage(ctx, &stake_key(address)).unwrap_or(0i64)
}

fn store_stake(ctx: &NeoStorageContext, address: &NeoByteString, amount: i64) -> NeoResult<()> {
    if amount == 0 {
        let key = NeoByteString::from_slice(&stake_key(address));
        NeoStorage::delete(ctx, &key)
    } else {
        store_to_storage(ctx, &stake_key(address), &amount)
    }
}

fn vote_key(id: i64, address: &NeoByteString) -> Vec<u8> {
    let mut key = VOTE_PREFIX.to_vec();
    key.extend_from_slice(&id.to_le_bytes());
    key.push(b':');
    key.extend_from_slice(address.as_slice());
    key
}

fn has_voted(ctx: &NeoStorageContext, id: i64, address: &NeoByteString) -> bool {
    load_from_storage(ctx, &vote_key(id, address)).unwrap_or(false)
}

fn record_vote(ctx: &NeoStorageContext, id: i64, address: &NeoByteString) -> NeoResult<()> {
    store_to_storage(ctx, &vote_key(id, address), &true)
}

fn read_address(ptr: i64, len: i64) -> Option<NeoByteString> {
    let bytes = read_bytes(ptr, len)?;
    if bytes.len() != 20 {
        return None;
    }
    Some(NeoByteString::from_slice(&bytes))
}

fn read_string(ptr: i64, len: i64) -> Option<String> {
    let bytes = read_bytes(ptr, len)?;
    String::from_utf8(bytes).ok()
}

fn read_bytes(ptr: i64, len: i64) -> Option<Vec<u8>> {
    if ptr == 0 || len <= 0 {
        return None;
    }
    let len = len as usize;
    let slice = unsafe { slice::from_raw_parts(ptr as *const u8, len) };
    Some(slice.to_vec())
}

fn ensure_witness(account: &NeoByteString) -> bool {
    NeoRuntime::check_witness(account)
        .map(|flag| flag.as_bool())
        .unwrap_or(false)
}

fn addresses_equal(left: &NeoByteString, right: &NeoByteString) -> bool {
    left.as_slice() == right.as_slice()
}

fn call_transfer(
    token: &NeoByteString,
    from: &NeoByteString,
    to: &NeoByteString,
    amount: i64,
) -> bool {
    let mut args = NeoArray::new();
    args.push(NeoValue::from(from.clone()));
    args.push(NeoValue::from(to.clone()));
    args.push(NeoValue::from(NeoInteger::new(amount)));

    match NeoContractRuntime::call(token, &NeoString::from_str("transfer"), &args) {
        Ok(value) => value
            .as_boolean()
            .map(|flag| flag.as_bool())
            .unwrap_or(true),
        Err(_) => false,
    }
}

fn load_from_storage<T>(ctx: &NeoStorageContext, key: &[u8]) -> Option<T>
where
    T: for<'de> Deserialize<'de>,
{
    let key_bytes = NeoByteString::from_slice(key);
    let data = NeoStorage::get(ctx, &key_bytes).ok()?;
    if data.is_empty() {
        return None;
    }
    codec::deserialize(data.as_slice()).ok()
}

fn store_to_storage<T>(ctx: &NeoStorageContext, key: &[u8], value: &T) -> NeoResult<()>
where
    T: Serialize,
{
    let encoded = codec::serialize(value)?;
    let key_bytes = NeoByteString::from_slice(key);
    let value_bytes = NeoByteString::from_slice(&encoded);
    NeoStorage::put(ctx, &key_bytes, &value_bytes)
}

fn serialize_value<T: Serialize>(value: &T) -> NeoByteString {
    match codec::serialize(value) {
        Ok(bytes) => NeoByteString::from_slice(&bytes),
        Err(_) => NeoByteString::new(Vec::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn address(byte: u8) -> Vec<u8> {
        vec![byte; 20]
    }

    fn reset_state() {
        let ctx = storage_context().unwrap();
        NeoStorage::delete(&ctx, &NeoByteString::from_slice(CONFIG_KEY)).ok();
        NeoStorage::delete(&ctx, &NeoByteString::from_slice(PROPOSAL_COUNTER_KEY)).ok();
        if let Ok(iter) = NeoStorage::find(&ctx, &NeoByteString::from_slice(PROPOSAL_PREFIX)) {
            let mut iterator = iter;
            while iterator.has_next() {
                if let Some(entry) = iterator.next() {
                    if let Some(key) = entry
                        .as_struct()
                        .and_then(|st| st.get_field("key"))
                        .and_then(NeoValue::as_byte_string)
                    {
                        NeoStorage::delete(&ctx, &key).ok();
                    }
                }
            }
        }
        if let Ok(iter) = NeoStorage::find(&ctx, &NeoByteString::from_slice(STAKE_PREFIX)) {
            let mut iterator = iter;
            while iterator.has_next() {
                if let Some(entry) = iterator.next() {
                    if let Some(key) = entry
                        .as_struct()
                        .and_then(|st| st.get_field("key"))
                        .and_then(NeoValue::as_byte_string)
                    {
                        NeoStorage::delete(&ctx, &key).ok();
                    }
                }
            }
        }
        if let Ok(iter) = NeoStorage::find(&ctx, &NeoByteString::from_slice(VOTE_PREFIX)) {
            let mut iterator = iter;
            while iterator.has_next() {
                if let Some(entry) = iterator.next() {
                    if let Some(key) = entry
                        .as_struct()
                        .and_then(|st| st.get_field("key"))
                        .and_then(NeoValue::as_byte_string)
                    {
                        NeoStorage::delete(&ctx, &key).ok();
                    }
                }
            }
        }
    }

    fn configure_sample(quorum: i64) -> DaoConfig {
        reset_state();
        let owner = address(0x55);
        let token = address(0x00);

        let status = configure(
            owner.as_ptr() as i64,
            owner.len() as i64,
            token.as_ptr() as i64,
            token.len() as i64,
            quorum,
        );
        assert_eq!(status, 1);

        let config_bytes = getConfig();
        codec::deserialize(config_bytes.as_slice()).expect("config decode")
    }

    #[test]
    fn configure_and_get_config() {
        let _guard = test_lock().lock().unwrap();
        let config = configure_sample(100);
        assert_eq!(config.quorum, 100);
    }

    #[test]
    fn stake_via_payment_updates_balance() {
        let _guard = test_lock().lock().unwrap();
        let config = configure_sample(100);
        let staker = config.owner.clone();
        let data = NeoByteString::from_slice(b"stake");
        onNEP17Payment(staker.clone(), 250, data);

        let staker_bytes = staker.as_slice().to_vec();
        let amount = stakeOf(staker_bytes.as_ptr() as i64, staker_bytes.len() as i64);
        assert_eq!(amount, 250);
    }

    #[test]
    fn proposal_vote_and_execute_flow() {
        let _guard = test_lock().lock().unwrap();
        let config = configure_sample(150);
        let voter = config.owner.clone();
        onNEP17Payment(voter.clone(), 200, NeoByteString::from_slice(b"stake"));

        let target = address(0x00);
        let method = b"upgrade".to_vec();
        let title = b"Upgrade".to_vec();
        let desc = b"Execute upgrade".to_vec();
        let proposal_id = propose(
            voter.as_slice().as_ptr() as i64,
            voter.len() as i64,
            target.as_ptr() as i64,
            target.len() as i64,
            method.as_ptr() as i64,
            method.len() as i64,
            title.as_ptr() as i64,
            title.len() as i64,
            desc.as_ptr() as i64,
            desc.len() as i64,
            0,
            100,
        );
        assert!(proposal_id > 0);

        let status = vote(
            proposal_id,
            voter.as_slice().as_ptr() as i64,
            voter.len() as i64,
            1,
            10,
        );
        assert_eq!(status, 1);

        assert_eq!(execute(proposal_id), 1);
        let stored = load_proposal(&storage_context().unwrap(), proposal_id).unwrap();
        assert!(stored.executed);
    }

    #[test]
    fn double_vote_rejected() {
        let _guard = test_lock().lock().unwrap();
        let config = configure_sample(50);
        let voter = config.owner.clone();
        onNEP17Payment(voter.clone(), 100, NeoByteString::from_slice(b"stake"));

        let target = address(0x00);
        let method = b"noop".to_vec();
        let title = b"Noop".to_vec();
        let desc = b"No operation".to_vec();
        let proposal_id = propose(
            voter.as_slice().as_ptr() as i64,
            voter.len() as i64,
            target.as_ptr() as i64,
            target.len() as i64,
            method.as_ptr() as i64,
            method.len() as i64,
            title.as_ptr() as i64,
            title.len() as i64,
            desc.as_ptr() as i64,
            desc.len() as i64,
            0,
            100,
        );
        assert!(proposal_id > 0);

        assert_eq!(
            vote(
                proposal_id,
                voter.as_slice().as_ptr() as i64,
                voter.len() as i64,
                1,
                10
            ),
            1
        );
        assert_eq!(
            vote(
                proposal_id,
                voter.as_slice().as_ptr() as i64,
                voter.len() as i64,
                0,
                20
            ),
            0
        );
    }
}
