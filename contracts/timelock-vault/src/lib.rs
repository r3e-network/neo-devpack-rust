// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "TimelockVault"
}"#
);

const KEY_COUNTER: i64 = -1;
const KEY_STRIDE: i64 = 16;
const FIELD_BENEFICIARY: i64 = 1;
const FIELD_AMOUNT: i64 = 2;
const FIELD_UNLOCK: i64 = 3;
const FIELD_RELEASED: i64 = 4;

fn vault_key(id: i64, field: i64) -> i64 {
    id * KEY_STRIDE + field
}

fn storage_put_i64(key: i64, value: i64) -> bool {
    RawStorage::put_i64_key(key, value);
    true
}

fn storage_get_i64(key: i64) -> i64 {
    RawStorage::get_i64_key_or_zero(key)
}

// Events
#[neo_event]
pub struct VaultQueued {
    pub vault_id: i64,
    pub beneficiary: i64,
    pub amount: i64,
    pub unlock_time: i64,
}

#[neo_event]
pub struct VaultReleased {
    pub vault_id: i64,
    pub beneficiary: i64,
    pub amount: i64,
}

#[neo_contract]
pub struct TimelockVaultContract;

#[neo_contract]
impl TimelockVaultContract {
    pub fn new() -> Self {
        Self
    }

    #[neo_method]
    pub fn queue_release(
        caller_id: i64,
        beneficiary_id: i64,
        amount: i64,
        unlock_time: i64,
    ) -> bool {
        if amount <= 0 || unlock_time <= 0 || caller_id == 0 || beneficiary_id == 0 {
            return false;
        }
        let id = match storage_get_i64(KEY_COUNTER).checked_add(1) {
            Some(next) if next > 0 && next <= i64::MAX / KEY_STRIDE => next,
            _ => return false,
        };
        storage_put_i64(KEY_COUNTER, id);
        storage_put_i64(vault_key(id, FIELD_BENEFICIARY), beneficiary_id);
        storage_put_i64(vault_key(id, FIELD_AMOUNT), amount);
        storage_put_i64(vault_key(id, FIELD_UNLOCK), unlock_time);
        storage_put_i64(vault_key(id, FIELD_RELEASED), 0);
        let _ = (VaultQueued {
            vault_id: id,
            beneficiary: beneficiary_id,
            amount,
            unlock_time,
        })
        .emit();
        true
    }

    #[neo_method(safe)]
    pub fn is_mature(unlock_time: i64, current_time: i64) -> bool {
        current_time >= unlock_time
    }

    #[neo_method]
    pub fn release(vault_id: i64, caller_id: i64, current_time: i64) -> bool {
        if vault_id <= 0 || caller_id == 0 {
            return false;
        }
        if vault_id > i64::MAX / KEY_STRIDE {
            return false;
        }
        let released = storage_get_i64(vault_key(vault_id, FIELD_RELEASED));
        if released != 0 {
            return false;
        }
        let unlock_time = storage_get_i64(vault_key(vault_id, FIELD_UNLOCK));
        if unlock_time == 0 {
            return false;
        }
        if current_time < unlock_time {
            return false;
        }
        let amount = storage_get_i64(vault_key(vault_id, FIELD_AMOUNT));
        let beneficiary_id = storage_get_i64(vault_key(vault_id, FIELD_BENEFICIARY));
        storage_put_i64(vault_key(vault_id, FIELD_RELEASED), 1);
        let _ = (VaultReleased {
            vault_id,
            beneficiary: beneficiary_id,
            amount,
        })
        .emit();
        true
    }
}

impl Default for TimelockVaultContract {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::TimelockVaultContract;

    #[test]
    fn is_mature_follows_time_guardrails() {
        assert!(TimelockVaultContract::is_mature(10, 10));
        assert!(!TimelockVaultContract::is_mature(11, 10));
    }
}
