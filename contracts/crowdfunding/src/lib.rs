// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "NeoCrowdfund"
}"#
);

// Storage keys
const CAMPAIGN_PREFIX: &[u8] = b"cf:";
const OWNER_SUFFIX: &[u8] = b":owner";
const TOKEN_SUFFIX: &[u8] = b":token";
const GOAL_SUFFIX: &[u8] = b":goal";
const DEADLINE_SUFFIX: &[u8] = b":deadline";
const MIN_CONTRIB_SUFFIX: &[u8] = b":min";
const RAISED_SUFFIX: &[u8] = b":raised";
const FINALIZED_SUFFIX: &[u8] = b":final";
const CONTRIB_PREFIX: &[u8] = b"cf:contrib:";

fn campaign_key(id: i64, suffix: &[u8]) -> Vec<u8> {
    let mut key = CAMPAIGN_PREFIX.to_vec();
    key.extend_from_slice(&id.to_le_bytes());
    key.extend_from_slice(suffix);
    key
}

fn contrib_key(campaign_id: i64, contributor: &NeoByteString) -> Vec<u8> {
    let mut key = CONTRIB_PREFIX.to_vec();
    key.extend_from_slice(&campaign_id.to_le_bytes());
    key.push(b':');
    key.extend_from_slice(contributor.as_slice());
    key
}

fn storage_put_bytes(ctx: &NeoStorageContext, key: &[u8], value: &[u8]) -> bool {
    NeoStorage::put(
        ctx,
        &NeoByteString::from_slice(key),
        &NeoByteString::from_slice(value),
    )
    .is_ok()
}

fn storage_get_bytes(ctx: &NeoStorageContext, key: &[u8]) -> Option<Vec<u8>> {
    let data = NeoStorage::get(ctx, &NeoByteString::from_slice(key)).ok()?;
    if data.is_empty() {
        return None;
    }
    Some(data.as_slice().to_vec())
}

fn storage_put_i64(ctx: &NeoStorageContext, key: &[u8], value: i64) -> bool {
    storage_put_bytes(ctx, key, &value.to_le_bytes())
}

fn storage_get_i64(ctx: &NeoStorageContext, key: &[u8]) -> Option<i64> {
    let bytes = storage_get_bytes(ctx, key)?;
    if bytes.len() != 8 {
        return None;
    }
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&bytes);
    Some(i64::from_le_bytes(buf))
}

fn storage_put_bool(ctx: &NeoStorageContext, key: &[u8], value: bool) -> bool {
    storage_put_bytes(ctx, key, &[value as u8])
}

fn storage_get_bool(ctx: &NeoStorageContext, key: &[u8]) -> Option<bool> {
    let bytes = storage_get_bytes(ctx, key)?;
    if bytes.len() != 1 {
        return None;
    }
    Some(bytes[0] != 0)
}

fn ensure_witness(account: &NeoByteString) -> bool {
    NeoRuntime::check_witness(account)
        .map(|flag| flag.as_bool())
        .unwrap_or(false)
}

fn read_address(ptr: i64, len: i64) -> Option<NeoByteString> {
    if ptr == 0 || len != 20 {
        return None;
    }
    let slice = unsafe { core::slice::from_raw_parts(ptr as *const u8, len as usize) };
    Some(NeoByteString::from_slice(slice))
}

// Events
#[neo_event]
pub struct CampaignCreated {
    pub campaign_id: NeoInteger,
    pub owner: NeoByteString,
    pub goal: NeoInteger,
}

#[neo_event]
pub struct ContributionReceived {
    pub campaign_id: NeoInteger,
    pub contributor: NeoByteString,
    pub amount: NeoInteger,
}

#[neo_event]
pub struct CampaignFinalized {
    pub campaign_id: NeoInteger,
    pub total_raised: NeoInteger,
}

#[neo_event]
pub struct RefundClaimed {
    pub campaign_id: NeoInteger,
    pub contributor: NeoByteString,
    pub amount: NeoInteger,
}

#[neo_contract]
pub struct NeoCrowdfundContract;

#[neo_contract]
impl NeoCrowdfundContract {
    pub fn new() -> Self {
        Self
    }

    #[neo_method]
    pub fn configure(
        campaign_id: i64,
        owner_ptr: i64,
        owner_len: i64,
        token_ptr: i64,
        token_len: i64,
        goal: i64,
        deadline: i64,
        min_contribution: i64,
    ) -> bool {
        if campaign_id <= 0 || goal <= 0 || deadline <= 0 || min_contribution <= 0 {
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
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return false,
        };
        if storage_get_bytes(&ctx, &campaign_key(campaign_id, OWNER_SUFFIX)).is_some() {
            return false;
        }
        storage_put_bytes(&ctx, &campaign_key(campaign_id, OWNER_SUFFIX), owner.as_slice());
        storage_put_bytes(&ctx, &campaign_key(campaign_id, TOKEN_SUFFIX), token.as_slice());
        storage_put_i64(&ctx, &campaign_key(campaign_id, GOAL_SUFFIX), goal);
        storage_put_i64(&ctx, &campaign_key(campaign_id, DEADLINE_SUFFIX), deadline);
        storage_put_i64(&ctx, &campaign_key(campaign_id, MIN_CONTRIB_SUFFIX), min_contribution);
        storage_put_i64(&ctx, &campaign_key(campaign_id, RAISED_SUFFIX), 0);
        storage_put_bool(&ctx, &campaign_key(campaign_id, FINALIZED_SUFFIX), false);
        let _ = (CampaignCreated {
            campaign_id: NeoInteger::new(campaign_id),
            owner,
            goal: NeoInteger::new(goal),
        })
        .emit();
        true
    }

    #[neo_method(name = "contributionOf")]
    pub fn contribution_of(campaign_id: i64, contributor_ptr: i64, contributor_len: i64) -> i64 {
        let contributor = match read_address(contributor_ptr, contributor_len) {
            Some(a) => a,
            None => return 0,
        };
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return 0,
        };
        storage_get_i64(&ctx, &contrib_key(campaign_id, &contributor)).unwrap_or(0)
    }

    #[neo_method]
    pub fn finalize(campaign_id: i64, caller_ptr: i64, caller_len: i64) -> bool {
        if campaign_id <= 0 {
            return false;
        }
        let caller = match read_address(caller_ptr, caller_len) {
            Some(a) => a,
            None => return false,
        };
        if !ensure_witness(&caller) {
            return false;
        }
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return false,
        };
        let owner_bytes = match storage_get_bytes(&ctx, &campaign_key(campaign_id, OWNER_SUFFIX)) {
            Some(b) => b,
            None => return false,
        };
        if caller.as_slice() != owner_bytes.as_slice() {
            return false;
        }
        if storage_get_bool(&ctx, &campaign_key(campaign_id, FINALIZED_SUFFIX)).unwrap_or(false) {
            return false;
        }
        storage_put_bool(&ctx, &campaign_key(campaign_id, FINALIZED_SUFFIX), true);
        let raised = storage_get_i64(&ctx, &campaign_key(campaign_id, RAISED_SUFFIX)).unwrap_or(0);
        let _ = (CampaignFinalized {
            campaign_id: NeoInteger::new(campaign_id),
            total_raised: NeoInteger::new(raised),
        })
        .emit();
        true
    }

    #[neo_method(name = "claimRefund")]
    pub fn claim_refund(campaign_id: i64, contributor_ptr: i64, contributor_len: i64) -> bool {
        if campaign_id <= 0 {
            return false;
        }
        let contributor = match read_address(contributor_ptr, contributor_len) {
            Some(a) => a,
            None => return false,
        };
        if !ensure_witness(&contributor) {
            return false;
        }
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return false,
        };
        let amount = storage_get_i64(&ctx, &contrib_key(campaign_id, &contributor)).unwrap_or(0);
        if amount <= 0 {
            return false;
        }
        let key = NeoByteString::from_slice(&contrib_key(campaign_id, &contributor));
        let _ = NeoStorage::delete(&ctx, &key);
        let _ = (RefundClaimed {
            campaign_id: NeoInteger::new(campaign_id),
            contributor,
            amount: NeoInteger::new(amount),
        })
        .emit();
        true
    }

    /// Return campaign state via notify: [goal, raised, deadline, min, finalized]
    #[neo_method(name = "getCampaign")]
    pub fn get_campaign(campaign_id: i64) {
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return,
        };
        let goal = match storage_get_i64(&ctx, &campaign_key(campaign_id, GOAL_SUFFIX)) {
            Some(v) => v,
            None => return,
        };
        let raised = storage_get_i64(&ctx, &campaign_key(campaign_id, RAISED_SUFFIX)).unwrap_or(0);
        let deadline = storage_get_i64(&ctx, &campaign_key(campaign_id, DEADLINE_SUFFIX)).unwrap_or(0);
        let min = storage_get_i64(&ctx, &campaign_key(campaign_id, MIN_CONTRIB_SUFFIX)).unwrap_or(0);
        let finalized = storage_get_bool(&ctx, &campaign_key(campaign_id, FINALIZED_SUFFIX)).unwrap_or(false);
        let label = NeoString::from_str("getCampaign");
        let mut state = NeoArray::new();
        state.push(NeoValue::from(NeoInteger::new(goal)));
        state.push(NeoValue::from(NeoInteger::new(raised)));
        state.push(NeoValue::from(NeoInteger::new(deadline)));
        state.push(NeoValue::from(NeoInteger::new(min)));
        state.push(NeoValue::from(NeoBoolean::new(finalized)));
        let _ = NeoRuntime::notify(&label, &state);
    }

    #[neo_method(name = "onNEP17Payment")]
    pub fn on_nep17_payment(from_ptr: i64, from_len: i64, amount: i64, data: i64) {
        if amount <= 0 || data <= 0 {
            return;
        }
        let from = match read_address(from_ptr, from_len) {
            Some(a) => a,
            None => return,
        };
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return,
        };
        let campaign_id = data;
        let current = storage_get_i64(&ctx, &contrib_key(campaign_id, &from)).unwrap_or(0);
        storage_put_i64(&ctx, &contrib_key(campaign_id, &from), current + amount);
        let raised = storage_get_i64(&ctx, &campaign_key(campaign_id, RAISED_SUFFIX)).unwrap_or(0);
        storage_put_i64(&ctx, &campaign_key(campaign_id, RAISED_SUFFIX), raised + amount);
        let _ = (ContributionReceived {
            campaign_id: NeoInteger::new(campaign_id),
            contributor: from,
            amount: NeoInteger::new(amount),
        })
        .emit();
    }
}

impl Default for NeoCrowdfundContract {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    // Integration tests require NeoVM runtime stubs.
}
