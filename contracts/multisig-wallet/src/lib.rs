// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "SampleMultisig"
}"#
);

// Storage keys
const CONFIG_THRESHOLD_KEY: &[u8] = b"cfg:threshold";
const CONFIG_OWNER_COUNT_KEY: &[u8] = b"cfg:owners";
const CONFIG_OWNER_PREFIX: &[u8] = b"cfg:owner:";
const PROPOSAL_COUNTER_KEY: &[u8] = b"proposal:counter";
const PROPOSAL_PREFIX: &[u8] = b"proposal:";
const PROPOSER_SUFFIX: &[u8] = b":proposer";
const TARGET_SUFFIX: &[u8] = b":target";
const METHOD_SUFFIX: &[u8] = b":method";
const ARG_SUFFIX: &[u8] = b":args";
const APPROVAL_COUNT_SUFFIX: &[u8] = b":approvals";
const APPROVAL_PREFIX: &[u8] = b":approval:";
const EXECUTED_SUFFIX: &[u8] = b":executed";

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

fn put_u16(ctx: &NeoStorageContext, key: &[u8], value: u16) -> bool {
    NeoStorage::put(
        ctx,
        &NeoByteString::from_slice(key),
        &NeoByteString::from_slice(&value.to_le_bytes()),
    )
    .is_ok()
}

fn get_u16(ctx: &NeoStorageContext, key: &[u8]) -> Option<u16> {
    let data = NeoStorage::get(ctx, &NeoByteString::from_slice(key)).ok()?;
    if data.len() != 2 {
        return None;
    }
    let s = data.as_slice();
    Some(u16::from_le_bytes([s[0], s[1]]))
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

fn put_string(ctx: &NeoStorageContext, key: &[u8], value: &str) -> bool {
    let mut buffer = Vec::with_capacity(2 + value.len());
    buffer.extend_from_slice(&(value.len() as u16).to_le_bytes());
    buffer.extend_from_slice(value.as_bytes());
    NeoStorage::put(
        ctx,
        &NeoByteString::from_slice(key),
        &NeoByteString::from_slice(&buffer),
    )
    .is_ok()
}

fn get_string(ctx: &NeoStorageContext, key: &[u8]) -> Option<String> {
    let data = NeoStorage::get(ctx, &NeoByteString::from_slice(key)).ok()?;
    if data.is_empty() {
        return None;
    }
    let bytes = data.as_slice();
    if bytes.len() < 2 {
        return None;
    }
    let len = u16::from_le_bytes([bytes[0], bytes[1]]) as usize;
    if bytes.len() - 2 != len {
        return None;
    }
    String::from_utf8(bytes[2..].to_vec()).ok()
}

// --- Key builders ---

fn config_owner_key(index: u16) -> Vec<u8> {
    let mut key = CONFIG_OWNER_PREFIX.to_vec();
    key.extend_from_slice(&index.to_le_bytes());
    key
}

fn proposal_field_key(id: i64, suffix: &[u8]) -> Vec<u8> {
    let mut key = PROPOSAL_PREFIX.to_vec();
    key.extend_from_slice(&id.to_le_bytes());
    key.extend_from_slice(suffix);
    key
}

fn proposal_approval_key(id: i64, index: u16) -> Vec<u8> {
    let mut key = PROPOSAL_PREFIX.to_vec();
    key.extend_from_slice(&id.to_le_bytes());
    key.extend_from_slice(APPROVAL_PREFIX);
    key.extend_from_slice(&index.to_le_bytes());
    key
}

// --- Config ---

fn load_config(ctx: &NeoStorageContext) -> Option<(Vec<i64>, i64)> {
    let threshold = get_i64(ctx, CONFIG_THRESHOLD_KEY)?;
    let owner_count = get_u16(ctx, CONFIG_OWNER_COUNT_KEY)? as usize;
    let mut owners = Vec::with_capacity(owner_count);
    for index in 0..owner_count {
        let key = config_owner_key(index as u16);
        let owner_id = get_i64(ctx, &key)?;
        owners.push(owner_id);
    }
    Some((owners, threshold))
}

fn store_config(ctx: &NeoStorageContext, owners: &[i64], threshold: i64) -> bool {
    if !put_i64(ctx, CONFIG_THRESHOLD_KEY, threshold) {
        return false;
    }
    if !put_u16(ctx, CONFIG_OWNER_COUNT_KEY, owners.len() as u16) {
        return false;
    }
    for (index, owner) in owners.iter().enumerate() {
        let key = config_owner_key(index as u16);
        if !put_i64(ctx, &key, *owner) {
            return false;
        }
    }
    true
}

fn is_owner(owners: &[i64], account: i64) -> bool {
    owners.iter().any(|&o| o == account)
}

// --- Proposal storage ---

fn next_proposal_id(ctx: &NeoStorageContext) -> Option<i64> {
    let current = get_i64(ctx, PROPOSAL_COUNTER_KEY).unwrap_or(0);
    let next = current.checked_add(1)?;
    if !put_i64(ctx, PROPOSAL_COUNTER_KEY, next) {
        return None;
    }
    Some(next)
}

fn store_proposal(
    ctx: &NeoStorageContext,
    id: i64,
    proposer: i64,
    target: i64,
    method: &str,
    arg_data: i64,
    approvals: &[i64],
    executed: bool,
) -> bool {
    put_i64(ctx, &proposal_field_key(id, PROPOSER_SUFFIX), proposer)
        && put_i64(ctx, &proposal_field_key(id, TARGET_SUFFIX), target)
        && put_string(ctx, &proposal_field_key(id, METHOD_SUFFIX), method)
        && put_i64(ctx, &proposal_field_key(id, ARG_SUFFIX), arg_data)
        && put_u16(ctx, &proposal_field_key(id, APPROVAL_COUNT_SUFFIX), approvals.len() as u16)
        && {
            for (idx, &a) in approvals.iter().enumerate() {
                if !put_i64(ctx, &proposal_approval_key(id, idx as u16), a) {
                    return false;
                }
            }
            true
        }
        && put_bool(ctx, &proposal_field_key(id, EXECUTED_SUFFIX), executed)
}

struct ProposalData {
    proposer: i64,
    target: i64,
    method: String,
    arg_data: i64,
    approvals: Vec<i64>,
    executed: bool,
}

fn load_proposal(ctx: &NeoStorageContext, id: i64) -> Option<ProposalData> {
    let proposer = get_i64(ctx, &proposal_field_key(id, PROPOSER_SUFFIX))?;
    let target = get_i64(ctx, &proposal_field_key(id, TARGET_SUFFIX))?;
    let method = get_string(ctx, &proposal_field_key(id, METHOD_SUFFIX))?;
    let arg_data = get_i64(ctx, &proposal_field_key(id, ARG_SUFFIX)).unwrap_or(0);
    let approval_count = get_u16(ctx, &proposal_field_key(id, APPROVAL_COUNT_SUFFIX)).unwrap_or(0);
    let mut approvals = Vec::with_capacity(approval_count as usize);
    for idx in 0..approval_count {
        let a = get_i64(ctx, &proposal_approval_key(id, idx))?;
        approvals.push(a);
    }
    let executed = get_bool(ctx, &proposal_field_key(id, EXECUTED_SUFFIX))?;
    Some(ProposalData {
        proposer,
        target,
        method,
        arg_data,
        approvals,
        executed,
    })
}

// --- Execution ---

fn execute_proposal_call(target_id: i64, method: &str, arg_data: i64) -> bool {
    // Build a contract call with integer arguments
    let target_bytes = target_id.to_le_bytes();
    let target = NeoByteString::from_slice(&target_bytes);
    let mut args = NeoArray::new();
    args.push(NeoValue::from(NeoInteger::new(arg_data)));
    NeoContractRuntime::call(&target, &NeoString::from_str(method), &args).is_ok()
}

// Events
#[neo_event]
pub struct ProposalCreatedEvt {
    pub proposal_id: NeoInteger,
    pub proposer: NeoInteger,
    pub target: NeoInteger,
    pub method: NeoString,
}

#[neo_event]
pub struct ProposalExecutedEvt {
    pub proposal_id: NeoInteger,
}

#[neo_contract]
pub struct SampleMultisigContract;

#[neo_contract]
impl SampleMultisigContract {
    pub fn new() -> Self {
        Self
    }

    /// Configure the multisig wallet with owners and threshold.
    /// Owners are passed as a series of i64 IDs packed into owner1..ownerN parameters.
    /// Can only be called once -- rejects reconfiguration after initial setup.
    #[neo_method]
    pub fn configure(owner1: i64, owner2: i64, owner3: i64, owner_count: i64, threshold: i64) -> bool {
        if threshold <= 0 || owner_count <= 0 || threshold > owner_count {
            return false;
        }
        if owner_count > 3 || owner1 == 0 {
            return false;
        }
        let mut owners = Vec::with_capacity(owner_count as usize);
        owners.push(owner1);
        if owner_count >= 2 {
            if owner2 == 0 { return false; }
            owners.push(owner2);
        }
        if owner_count >= 3 {
            if owner3 == 0 { return false; }
            owners.push(owner3);
        }
        let ctx = match storage_ctx() {
            Some(c) => c,
            None => return false,
        };
        // Prevent reconfiguration
        if load_config(&ctx).is_some() {
            return false;
        }
        store_config(&ctx, &owners, threshold)
    }

    /// Create a new proposal.
    #[neo_method]
    pub fn propose(
        proposer_id: i64,
        target_id: i64,
        method_id: i64,
        arg_data: i64,
    ) -> bool {
        if proposer_id == 0 || target_id == 0 {
            return false;
        }
        let ctx = match storage_ctx() {
            Some(c) => c,
            None => return false,
        };
        let (owners, _threshold) = match load_config(&ctx) {
            Some(c) => c,
            None => return false,
        };
        if !is_owner(&owners, proposer_id) {
            return false;
        }
        let id = match next_proposal_id(&ctx) {
            Some(i) => i,
            None => return false,
        };
        let method_str = method_id.to_string();
        let approvals = vec![proposer_id];
        if !store_proposal(&ctx, id, proposer_id, target_id, &method_str, arg_data, &approvals, false) {
            return false;
        }
        let _ = (ProposalCreatedEvt {
            proposal_id: NeoInteger::new(id),
            proposer: NeoInteger::new(proposer_id),
            target: NeoInteger::new(target_id),
            method: NeoString::from_str(&method_str),
        })
        .emit();
        true
    }

    /// Approve an existing proposal.
    #[neo_method]
    pub fn approve(proposal_id: i64, signer_id: i64) -> bool {
        if signer_id == 0 {
            return false;
        }
        let ctx = match storage_ctx() {
            Some(c) => c,
            None => return false,
        };
        let (owners, _threshold) = match load_config(&ctx) {
            Some(c) => c,
            None => return false,
        };
        if !is_owner(&owners, signer_id) {
            return false;
        }
        let mut proposal = match load_proposal(&ctx, proposal_id) {
            Some(p) => p,
            None => return false,
        };
        if proposal.executed {
            return false;
        }
        if proposal.approvals.iter().any(|&a| a == signer_id) {
            return false;
        }
        proposal.approvals.push(signer_id);
        store_proposal(
            &ctx,
            proposal_id,
            proposal.proposer,
            proposal.target,
            &proposal.method,
            proposal.arg_data,
            &proposal.approvals,
            proposal.executed,
        )
    }

    /// Execute a proposal once threshold approvals are reached.
    #[neo_method]
    pub fn execute(proposal_id: i64, executor_id: i64) -> bool {
        if executor_id == 0 {
            return false;
        }
        let ctx = match storage_ctx() {
            Some(c) => c,
            None => return false,
        };
        let (owners, threshold) = match load_config(&ctx) {
            Some(c) => c,
            None => return false,
        };
        if !is_owner(&owners, executor_id) {
            return false;
        }
        let proposal = match load_proposal(&ctx, proposal_id) {
            Some(p) => p,
            None => return false,
        };
        if proposal.executed {
            return false;
        }
        if (proposal.approvals.len() as i64) < threshold {
            return false;
        }
        if !execute_proposal_call(proposal.target, &proposal.method, proposal.arg_data) {
            return false;
        }
        if !store_proposal(
            &ctx,
            proposal_id,
            proposal.proposer,
            proposal.target,
            &proposal.method,
            proposal.arg_data,
            &proposal.approvals,
            true,
        ) {
            return false;
        }
        let _ = (ProposalExecutedEvt {
            proposal_id: NeoInteger::new(proposal_id),
        })
        .emit();
        true
    }

    /// Return the current wallet configuration via notify.
    #[neo_method(safe, name = "getConfig")]
    pub fn get_config() {
        let ctx = match storage_ctx() {
            Some(c) => c,
            None => return,
        };
        let (owners, threshold) = match load_config(&ctx) {
            Some(c) => c,
            None => return,
        };
        let label = NeoString::from_str("getConfig");
        let mut state = NeoArray::new();
        state.push(NeoValue::from(NeoInteger::new(threshold)));
        state.push(NeoValue::from(NeoInteger::new(owners.len() as i64)));
        for owner in &owners {
            state.push(NeoValue::from(NeoInteger::new(*owner)));
        }
        let _ = NeoRuntime::notify(&label, &state);
    }
}

impl Default for SampleMultisigContract {
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
