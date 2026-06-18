// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "NeoCrowdfund"
}"#
);

// Campaign field keys use direct i64 storage, avoiding wasm linear-memory key
// materialisation for the hot path: [campaign_id * 16 + field].
const CAMPAIGN_KEY_STRIDE: i64 = 16;
const MAX_CAMPAIGN_ID: i64 = i64::MAX / CAMPAIGN_KEY_STRIDE;
const FIELD_OWNER: i64 = 1;
const FIELD_TOKEN: i64 = 2;
const FIELD_GOAL: i64 = 3;
const FIELD_DEADLINE: i64 = 4;
const FIELD_MIN: i64 = 5;
const FIELD_RAISED: i64 = 6;
const FIELD_FINAL: i64 = 7;

// Contribution keys: [prefix 11][campaign_id 8][:][contributor 8] = 28 bytes.
const CONTRIB_PREFIX: [u8; 11] = *b"cf:contrib:";

// Contribution key size: 11 + 8 + 1 + 8 = 28.
const CONTRIB_KEY_LEN: usize = 28;

#[inline(always)]
fn valid_campaign_id(campaign_id: i64) -> bool {
    campaign_id > 0 && campaign_id <= MAX_CAMPAIGN_ID
}

#[inline(always)]
fn campaign_key(campaign_id: i64, field: i64) -> i64 {
    campaign_id * CAMPAIGN_KEY_STRIDE + field
}

#[inline(always)]
fn campaign_put_i64(campaign_id: i64, field: i64, value: i64) {
    RawStorage::put_i64_key(campaign_key(campaign_id, field), value);
}

#[inline(always)]
fn campaign_get_i64(campaign_id: i64, field: i64) -> i64 {
    RawStorage::get_i64_key_or_zero(campaign_key(campaign_id, field))
}

/// Build a contribution key without heap allocation. Always 28 bytes.
#[inline(always)]
fn contrib_key(key: &mut RawKeyBuilder<CONTRIB_KEY_LEN>, campaign_id: i64, contributor: i64) {
    key.clear();
    key.push_bytes(&CONTRIB_PREFIX);
    key.push_i64_le(campaign_id);
    key.push_byte(b':');
    key.push_i64_le(contributor);
}

fn storage_put_i64(key: &[u8], value: i64) -> bool {
    RawStorage::put_i64(key, value);
    true
}

fn storage_get_i64(key: &[u8]) -> Option<i64> {
    RawStorage::get_i64(key)
}

fn ensure_witness_i64(account: i64) -> bool {
    NeoRuntime::check_witness_i64(account)
        .map(|flag| flag.as_bool())
        .unwrap_or(false)
}

fn calling_contract_id() -> i64 {
    NeoRuntime::get_calling_script_hash_i64().unwrap_or(0)
}

fn current_time_i64() -> Option<i64> {
    NeoRuntime::get_time_i64().ok()
}

// Events
#[neo_event]
pub struct CampaignCreated {
    pub campaign_id: i64,
    pub owner: i64,
    pub goal: i64,
}

#[neo_event]
pub struct ContributionReceived {
    pub campaign_id: i64,
    pub contributor: i64,
    pub amount: i64,
}

#[neo_event]
pub struct CampaignFinalized {
    pub campaign_id: i64,
    pub total_raised: i64,
}

#[neo_event]
pub struct RefundClaimed {
    pub campaign_id: i64,
    pub contributor: i64,
    pub amount: i64,
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
        if !valid_campaign_id(campaign_id)
            || goal <= 0
            || deadline <= 0
            || min_contribution <= 0
        {
            return false;
        }
        if owner == 0 || token == 0 {
            return false;
        }
        if !ensure_witness_i64(owner) {
            return false;
        }

        // Check campaign does not already exist
        if campaign_get_i64(campaign_id, FIELD_OWNER) != 0 {
            return false;
        }

        // Store owner (as i64)
        campaign_put_i64(campaign_id, FIELD_OWNER, owner);

        // Store token
        campaign_put_i64(campaign_id, FIELD_TOKEN, token);

        // Store goal
        campaign_put_i64(campaign_id, FIELD_GOAL, goal);

        // Store deadline
        campaign_put_i64(campaign_id, FIELD_DEADLINE, deadline);

        // Store min contribution
        campaign_put_i64(campaign_id, FIELD_MIN, min_contribution);

        // Store raised = 0
        campaign_put_i64(campaign_id, FIELD_RAISED, 0);

        // Store finalized = false
        campaign_put_i64(campaign_id, FIELD_FINAL, 0);

        let _ = (CampaignCreated {
            campaign_id,
            owner,
            goal,
        })
        .emit();
        true
    }

    #[neo_method(safe, name = "contributionOf")]
    pub fn contribution_of(campaign_id: i64, contributor: i64) -> i64 {
        if !valid_campaign_id(campaign_id) {
            return 0;
        }
        let mut key = RawKeyBuilder::new();
        contrib_key(&mut key, campaign_id, contributor);
        storage_get_i64(key.as_slice()).unwrap_or(0)
    }

    #[neo_method]
    pub fn finalize(campaign_id: i64, caller: i64) -> bool {
        if !valid_campaign_id(campaign_id) {
            return false;
        }
        if caller == 0 {
            return false;
        }
        if !ensure_witness_i64(caller) {
            return false;
        }

        // Check caller is the owner
        let stored_owner = campaign_get_i64(campaign_id, FIELD_OWNER);
        if stored_owner == 0 {
            return false;
        }
        if caller != stored_owner {
            return false;
        }

        // Check not already finalized
        if campaign_get_i64(campaign_id, FIELD_FINAL) != 0 {
            return false;
        }
        let deadline = campaign_get_i64(campaign_id, FIELD_DEADLINE);
        let now = match current_time_i64() {
            Some(t) => t,
            None => return false,
        };
        if now < deadline {
            return false;
        }

        // Mark finalized
        campaign_put_i64(campaign_id, FIELD_FINAL, 1);

        // Read raised amount
        let raised = campaign_get_i64(campaign_id, FIELD_RAISED);

        let _ = (CampaignFinalized {
            campaign_id,
            total_raised: raised,
        })
        .emit();
        true
    }

    #[neo_method(name = "claimRefund")]
    pub fn claim_refund(campaign_id: i64, contributor: i64) -> bool {
        if !valid_campaign_id(campaign_id) {
            return false;
        }
        if contributor == 0 {
            return false;
        }
        if !ensure_witness_i64(contributor) {
            return false;
        }
        if campaign_get_i64(campaign_id, FIELD_FINAL) == 0 {
            return false;
        }
        let goal = campaign_get_i64(campaign_id, FIELD_GOAL);
        let raised = campaign_get_i64(campaign_id, FIELD_RAISED);
        if goal == 0 || raised >= goal {
            return false;
        }

        let mut key = RawKeyBuilder::new();
        contrib_key(&mut key, campaign_id, contributor);

        let amount = storage_get_i64(key.as_slice()).unwrap_or(0);
        if amount <= 0 {
            return false;
        }

        RawStorage::delete(key.as_slice());

        let _ = (RefundClaimed {
            campaign_id,
            contributor,
            amount,
        })
        .emit();
        true
    }

    /// Return campaign state via notify: [goal, raised, deadline, min, finalized]
    #[neo_method(safe, name = "getCampaign")]
    pub fn get_campaign(campaign_id: i64) {
        if !valid_campaign_id(campaign_id) {
            return;
        }
        let goal = campaign_get_i64(campaign_id, FIELD_GOAL);
        if goal == 0 {
            return;
        }

        let raised = campaign_get_i64(campaign_id, FIELD_RAISED);

        let deadline = campaign_get_i64(campaign_id, FIELD_DEADLINE);

        let min = campaign_get_i64(campaign_id, FIELD_MIN);

        let finalized = campaign_get_i64(campaign_id, FIELD_FINAL) != 0;

        #[cfg(target_arch = "wasm32")]
        {
            let _ = (goal, raised, deadline, min, finalized);
            let _ = NeoRuntime::notify_event("getCampaign");
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let label = NeoString::from_str("getCampaign");
            let mut state = NeoArray::new();
            state.push(NeoValue::from(goal));
            state.push(NeoValue::from(raised));
            state.push(NeoValue::from(deadline));
            state.push(NeoValue::from(min));
            state.push(NeoValue::from(finalized));
            let _ = NeoRuntime::notify(&label, &state);
        }
    }

    #[neo_method(name = "onNEP17Payment")]
    pub fn on_nep17_payment(from: i64, amount: i64, data: i64) {
        if amount <= 0 || !valid_campaign_id(data) || from == 0 {
            return;
        }
        let campaign_id = data;
        let token = campaign_get_i64(campaign_id, FIELD_TOKEN);
        let min_contribution = campaign_get_i64(campaign_id, FIELD_MIN);
        let deadline = campaign_get_i64(campaign_id, FIELD_DEADLINE);
        if token == 0 || min_contribution <= 0 || deadline <= 0 {
            return;
        }
        if calling_contract_id() != token {
            return;
        }
        if amount < min_contribution || campaign_get_i64(campaign_id, FIELD_FINAL) != 0 {
            return;
        }
        let now = match current_time_i64() {
            Some(t) => t,
            None => return,
        };
        if now > deadline {
            return;
        }

        let mut contrib = RawKeyBuilder::new();
        contrib_key(&mut contrib, campaign_id, from);

        let current = storage_get_i64(contrib.as_slice()).unwrap_or(0);
        let new_contrib = match current.checked_add(amount) {
            Some(v) => v,
            None => return,
        };
        storage_put_i64(contrib.as_slice(), new_contrib);

        let raised = campaign_get_i64(campaign_id, FIELD_RAISED);
        let new_raised = match raised.checked_add(amount) {
            Some(v) => v,
            None => return,
        };
        campaign_put_i64(campaign_id, FIELD_RAISED, new_raised);

        let _ = (ContributionReceived {
            campaign_id,
            contributor: from,
            amount,
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
