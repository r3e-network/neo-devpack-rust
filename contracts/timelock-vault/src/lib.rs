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

fn ensure_witness_i64(account: i64) -> bool {
    NeoRuntime::check_witness_i64(account)
        .map(|flag| flag.as_bool())
        .unwrap_or(false)
}

/// Authoritative block time for maturity enforcement.
///
/// On-chain (`wasm32`) the timestamp comes from `System.Runtime.GetTime`, so a
/// caller cannot bypass the timelock by lying about the time. The caller-supplied
/// `fallback` is used only in host-side smoke tests where the runtime is not
/// simulated (the syscall returns 0).
#[inline(always)]
fn runtime_time(fallback: i64) -> i64 {
    #[cfg(target_arch = "wasm32")]
    {
        NeoRuntime::get_time_i64().unwrap_or(0)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = NeoRuntime::get_time_i64();
        fallback
    }
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
        if !ensure_witness_i64(caller_id) {
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
        if !ensure_witness_i64(caller_id) {
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
        // Maturity is enforced against the chain's block time, not a
        // caller-supplied value, so an attacker cannot fast-forward a vault.
        if runtime_time(current_time) < unlock_time {
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
    fn is_mature_follows_time_guardrails() {
        assert!(TimelockVaultContract::is_mature(10, 10));
        assert!(!TimelockVaultContract::is_mature(11, 10));
    }

    #[test]
    fn queue_release_requires_witness() {
        // Without the caller in the witness set the queue must be rejected
        // even when every other input is valid.
        let _guard = setup_witnesses(&[]);
        assert!(!TimelockVaultContract::queue_release(5, 6, 100, 10));

        let _guard = setup_witnesses(&[5]);
        assert!(TimelockVaultContract::queue_release(5, 6, 100, 10));
    }

    #[test]
    fn release_enforces_maturity_against_runtime_time() {
        let _guard = setup_witnesses(&[7]);
        // Queue a vault triggered by account 7 unlocking at time 100.
        assert!(TimelockVaultContract::queue_release(7, 8, 100, 100));
        // Same caller cannot release before maturity (host fallback path).
        assert!(!TimelockVaultContract::release(1, 7, 50));
        // Once the supplied time is at/after unlock, release succeeds.
        assert!(TimelockVaultContract::release(1, 7, 100));
        // Releasing twice is rejected.
        assert!(!TimelockVaultContract::release(1, 7, 100));
    }
}
