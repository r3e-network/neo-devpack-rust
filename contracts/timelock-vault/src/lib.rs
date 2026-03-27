// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "TimelockVault"
}"#
);

// Storage keys
const VAULT_PREFIX: &[u8] = b"vault:";
const BENEFICIARY_SUFFIX: &[u8] = b":ben";
const AMOUNT_SUFFIX: &[u8] = b":amt";
const UNLOCK_SUFFIX: &[u8] = b":unlock";
const RELEASED_SUFFIX: &[u8] = b":released";
const VAULT_COUNTER_KEY: &[u8] = b"vault:counter";

fn vault_key(id: i64, suffix: &[u8]) -> Vec<u8> {
    let mut key = VAULT_PREFIX.to_vec();
    key.extend_from_slice(&id.to_le_bytes());
    key.extend_from_slice(suffix);
    key
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
    if data.len() != 8 {
        return None;
    }
    let s = data.as_slice();
    let mut buf = [0u8; 8];
    buf.copy_from_slice(s);
    Some(i64::from_le_bytes(buf))
}

// Events
#[neo_event]
pub struct VaultQueued {
    pub vault_id: NeoInteger,
    pub beneficiary: NeoInteger,
    pub amount: NeoInteger,
    pub unlock_time: NeoInteger,
}

#[neo_event]
pub struct VaultReleased {
    pub vault_id: NeoInteger,
    pub beneficiary: NeoInteger,
    pub amount: NeoInteger,
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
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return false,
        };
        let id = storage_get_i64(&ctx, VAULT_COUNTER_KEY).unwrap_or(0) + 1;
        storage_put_i64(&ctx, VAULT_COUNTER_KEY, id);
        storage_put_i64(&ctx, &vault_key(id, BENEFICIARY_SUFFIX), beneficiary_id);
        storage_put_i64(&ctx, &vault_key(id, AMOUNT_SUFFIX), amount);
        storage_put_i64(&ctx, &vault_key(id, UNLOCK_SUFFIX), unlock_time);
        storage_put_i64(&ctx, &vault_key(id, RELEASED_SUFFIX), 0);
        let _ = (VaultQueued {
            vault_id: NeoInteger::new(id),
            beneficiary: NeoInteger::new(beneficiary_id),
            amount: NeoInteger::new(amount),
            unlock_time: NeoInteger::new(unlock_time),
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
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return false,
        };
        let released = storage_get_i64(&ctx, &vault_key(vault_id, RELEASED_SUFFIX)).unwrap_or(1);
        if released != 0 {
            return false;
        }
        let unlock_time = match storage_get_i64(&ctx, &vault_key(vault_id, UNLOCK_SUFFIX)) {
            Some(t) => t,
            None => return false,
        };
        if current_time < unlock_time {
            return false;
        }
        let amount = storage_get_i64(&ctx, &vault_key(vault_id, AMOUNT_SUFFIX)).unwrap_or(0);
        let beneficiary_id = storage_get_i64(&ctx, &vault_key(vault_id, BENEFICIARY_SUFFIX)).unwrap_or(0);
        storage_put_i64(&ctx, &vault_key(vault_id, RELEASED_SUFFIX), 1);
        let _ = (VaultReleased {
            vault_id: NeoInteger::new(vault_id),
            beneficiary: NeoInteger::new(beneficiary_id),
            amount: NeoInteger::new(amount),
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
