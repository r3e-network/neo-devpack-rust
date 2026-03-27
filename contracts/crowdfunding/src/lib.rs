// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "NeoCrowdfund"
}"#
);

// Storage key constants — fixed-length tags avoid heap allocation.
// Layout: [prefix 3][campaign_id 8][suffix up to 8] = max 19 bytes.
const PREFIX: [u8; 3] = *b"cf:";
const SUFFIX_OWNER: [u8; 6] = *b":owner";
const SUFFIX_TOKEN: [u8; 6] = *b":token";
const SUFFIX_GOAL: [u8; 5] = *b":goal";
const SUFFIX_DEADLINE: [u8; 9] = *b":deadline";
const SUFFIX_MIN: [u8; 4] = *b":min";
const SUFFIX_RAISED: [u8; 7] = *b":raised";
const SUFFIX_FINAL: [u8; 6] = *b":final";

// Contribution keys: [prefix 11][campaign_id 8][:][contributor 8] = 28 bytes.
const CONTRIB_PREFIX: [u8; 11] = *b"cf:contrib:";

// Maximum key buffer size: 3 + 8 + 9 = 20 for campaign keys.
const MAX_CAMPAIGN_KEY: usize = 20;
// Contribution key size: 11 + 8 + 1 + 8 = 28.
const CONTRIB_KEY_LEN: usize = 28;

/// Build a campaign storage key into a fixed buffer. Returns the used length.
fn campaign_key(buf: &mut [u8; MAX_CAMPAIGN_KEY], id: i64, suffix: &[u8]) -> usize {
    let id_bytes = id.to_le_bytes();
    let len = PREFIX.len() + id_bytes.len() + suffix.len();
    let mut pos = 0;
    buf[pos..pos + PREFIX.len()].copy_from_slice(&PREFIX);
    pos += PREFIX.len();
    buf[pos..pos + 8].copy_from_slice(&id_bytes);
    pos += 8;
    buf[pos..pos + suffix.len()].copy_from_slice(suffix);
    len
}

/// Build a contribution key into a fixed buffer. Always 28 bytes.
fn contrib_key(buf: &mut [u8; CONTRIB_KEY_LEN], campaign_id: i64, contributor: i64) {
    let cid = campaign_id.to_le_bytes();
    let ctr = contributor.to_le_bytes();
    let mut pos = 0;
    buf[pos..pos + CONTRIB_PREFIX.len()].copy_from_slice(&CONTRIB_PREFIX);
    pos += CONTRIB_PREFIX.len();
    buf[pos..pos + 8].copy_from_slice(&cid);
    pos += 8;
    buf[pos] = b':';
    pos += 1;
    buf[pos..pos + 8].copy_from_slice(&ctr);
}

fn storage_put_i64(ctx: &NeoStorageContext, key: &[u8], value: i64) -> bool {
    NeoStorage::put(
        ctx,
        &NeoByteString::from_slice(key),
        &NeoByteString::from_slice(&value.to_le_bytes()),
    )
    .is_ok()
}

fn storage_get_i64(ctx: &NeoStorageContext, key: &[u8]) -> Option<i64> {
    let data = NeoStorage::get(ctx, &NeoByteString::from_slice(key)).ok()?;
    let s = data.as_slice();
    if s.len() != 8 {
        return None;
    }
    let mut buf = [0u8; 8];
    buf.copy_from_slice(s);
    Some(i64::from_le_bytes(buf))
}

fn storage_put_bool(ctx: &NeoStorageContext, key: &[u8], value: bool) -> bool {
    NeoStorage::put(
        ctx,
        &NeoByteString::from_slice(key),
        &NeoByteString::from_slice(&[value as u8]),
    )
    .is_ok()
}

fn storage_get_bool(ctx: &NeoStorageContext, key: &[u8]) -> Option<bool> {
    let data = NeoStorage::get(ctx, &NeoByteString::from_slice(key)).ok()?;
    let s = data.as_slice();
    if s.len() != 1 {
        return None;
    }
    Some(s[0] != 0)
}

fn storage_has_key(ctx: &NeoStorageContext, key: &[u8]) -> bool {
    NeoStorage::get(ctx, &NeoByteString::from_slice(key))
        .ok()
        .map(|d| !d.is_empty())
        .unwrap_or(false)
}

fn ensure_witness_i64(account: i64) -> bool {
    let bs = NeoByteString::from_slice(&account.to_le_bytes());
    NeoRuntime::check_witness(&bs)
        .map(|flag| flag.as_bool())
        .unwrap_or(false)
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
        owner: i64,
        token: i64,
        goal: i64,
        deadline: i64,
        min_contribution: i64,
    ) -> bool {
        if campaign_id <= 0 || goal <= 0 || deadline <= 0 || min_contribution <= 0 {
            return false;
        }
        if owner == 0 || token == 0 {
            return false;
        }
        if !ensure_witness_i64(owner) {
            return false;
        }
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return false,
        };

        // Check campaign does not already exist
        let mut kb = [0u8; MAX_CAMPAIGN_KEY];
        let kl = campaign_key(&mut kb, campaign_id, &SUFFIX_OWNER);
        if storage_has_key(&ctx, &kb[..kl]) {
            return false;
        }

        // Store owner (as i64)
        storage_put_i64(&ctx, &kb[..kl], owner);

        // Store token
        let kl = campaign_key(&mut kb, campaign_id, &SUFFIX_TOKEN);
        storage_put_i64(&ctx, &kb[..kl], token);

        // Store goal
        let kl = campaign_key(&mut kb, campaign_id, &SUFFIX_GOAL);
        storage_put_i64(&ctx, &kb[..kl], goal);

        // Store deadline
        let kl = campaign_key(&mut kb, campaign_id, &SUFFIX_DEADLINE);
        storage_put_i64(&ctx, &kb[..kl], deadline);

        // Store min contribution
        let kl = campaign_key(&mut kb, campaign_id, &SUFFIX_MIN);
        storage_put_i64(&ctx, &kb[..kl], min_contribution);

        // Store raised = 0
        let kl = campaign_key(&mut kb, campaign_id, &SUFFIX_RAISED);
        storage_put_i64(&ctx, &kb[..kl], 0);

        // Store finalized = false
        let kl = campaign_key(&mut kb, campaign_id, &SUFFIX_FINAL);
        storage_put_bool(&ctx, &kb[..kl], false);

        let _ = (CampaignCreated {
            campaign_id: NeoInteger::new(campaign_id),
            owner: NeoByteString::from_slice(&owner.to_le_bytes()),
            goal: NeoInteger::new(goal),
        })
        .emit();
        true
    }

    #[neo_method(safe, name = "contributionOf")]
    pub fn contribution_of(campaign_id: i64, contributor: i64) -> i64 {
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return 0,
        };
        let mut ck = [0u8; CONTRIB_KEY_LEN];
        contrib_key(&mut ck, campaign_id, contributor);
        storage_get_i64(&ctx, &ck).unwrap_or(0)
    }

    #[neo_method]
    pub fn finalize(campaign_id: i64, caller: i64) -> bool {
        if campaign_id <= 0 {
            return false;
        }
        if caller == 0 {
            return false;
        }
        if !ensure_witness_i64(caller) {
            return false;
        }
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return false,
        };

        // Check caller is the owner
        let mut kb = [0u8; MAX_CAMPAIGN_KEY];
        let kl = campaign_key(&mut kb, campaign_id, &SUFFIX_OWNER);
        let stored_owner = match storage_get_i64(&ctx, &kb[..kl]) {
            Some(v) => v,
            None => return false,
        };
        if caller != stored_owner {
            return false;
        }

        // Check not already finalized
        let kl = campaign_key(&mut kb, campaign_id, &SUFFIX_FINAL);
        if storage_get_bool(&ctx, &kb[..kl]).unwrap_or(false) {
            return false;
        }

        // Mark finalized
        storage_put_bool(&ctx, &kb[..kl], true);

        // Read raised amount
        let kl = campaign_key(&mut kb, campaign_id, &SUFFIX_RAISED);
        let raised = storage_get_i64(&ctx, &kb[..kl]).unwrap_or(0);

        let _ = (CampaignFinalized {
            campaign_id: NeoInteger::new(campaign_id),
            total_raised: NeoInteger::new(raised),
        })
        .emit();
        true
    }

    #[neo_method(name = "claimRefund")]
    pub fn claim_refund(campaign_id: i64, contributor: i64) -> bool {
        if campaign_id <= 0 {
            return false;
        }
        if contributor == 0 {
            return false;
        }
        if !ensure_witness_i64(contributor) {
            return false;
        }
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return false,
        };

        let mut ck = [0u8; CONTRIB_KEY_LEN];
        contrib_key(&mut ck, campaign_id, contributor);

        let amount = storage_get_i64(&ctx, &ck).unwrap_or(0);
        if amount <= 0 {
            return false;
        }

        let _ = NeoStorage::delete(&ctx, &NeoByteString::from_slice(&ck));

        let _ = (RefundClaimed {
            campaign_id: NeoInteger::new(campaign_id),
            contributor: NeoByteString::from_slice(&contributor.to_le_bytes()),
            amount: NeoInteger::new(amount),
        })
        .emit();
        true
    }

    /// Return campaign state via notify: [goal, raised, deadline, min, finalized]
    #[neo_method(safe, name = "getCampaign")]
    pub fn get_campaign(campaign_id: i64) {
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return,
        };
        let mut kb = [0u8; MAX_CAMPAIGN_KEY];

        let kl = campaign_key(&mut kb, campaign_id, &SUFFIX_GOAL);
        let goal = match storage_get_i64(&ctx, &kb[..kl]) {
            Some(v) => v,
            None => return,
        };

        let kl = campaign_key(&mut kb, campaign_id, &SUFFIX_RAISED);
        let raised = storage_get_i64(&ctx, &kb[..kl]).unwrap_or(0);

        let kl = campaign_key(&mut kb, campaign_id, &SUFFIX_DEADLINE);
        let deadline = storage_get_i64(&ctx, &kb[..kl]).unwrap_or(0);

        let kl = campaign_key(&mut kb, campaign_id, &SUFFIX_MIN);
        let min = storage_get_i64(&ctx, &kb[..kl]).unwrap_or(0);

        let kl = campaign_key(&mut kb, campaign_id, &SUFFIX_FINAL);
        let finalized = storage_get_bool(&ctx, &kb[..kl]).unwrap_or(false);

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
    pub fn on_nep17_payment(from: i64, amount: i64, data: i64) {
        if amount <= 0 || data <= 0 || from == 0 {
            return;
        }
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return,
        };
        let campaign_id = data;

        let mut ck = [0u8; CONTRIB_KEY_LEN];
        contrib_key(&mut ck, campaign_id, from);

        let current = storage_get_i64(&ctx, &ck).unwrap_or(0);
        let new_contrib = match current.checked_add(amount) {
            Some(v) => v,
            None => return,
        };
        storage_put_i64(&ctx, &ck, new_contrib);

        let mut kb = [0u8; MAX_CAMPAIGN_KEY];
        let kl = campaign_key(&mut kb, campaign_id, &SUFFIX_RAISED);
        let raised = storage_get_i64(&ctx, &kb[..kl]).unwrap_or(0);
        let new_raised = match raised.checked_add(amount) {
            Some(v) => v,
            None => return,
        };
        storage_put_i64(&ctx, &kb[..kl], new_raised);

        let _ = (ContributionReceived {
            campaign_id: NeoInteger::new(campaign_id),
            contributor: NeoByteString::from_slice(&from.to_le_bytes()),
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
    #[test]
    fn contract_compiles() {
        // Compilation test - verifies contract module parses correctly
    }
}
