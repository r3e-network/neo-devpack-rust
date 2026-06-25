// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

#![allow(clippy::too_many_arguments)]

use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "NeoEscrow"
}"#
);

// Storage key constants (numeric prefixes avoid heap-allocated Vec<u8> key construction)
const KEY_PAYER: i64 = 1;
const KEY_PAYEE: i64 = 2;
const KEY_ARBITER: i64 = 3;
const KEY_TOKEN: i64 = 4;
const KEY_AMOUNT: i64 = 5;
const KEY_RELEASE_HEIGHT: i64 = 6;
const KEY_REFUND_HEIGHT: i64 = 7;
const KEY_STATUS: i64 = 8;

// Status constants
const STATUS_ACTIVE: i64 = 1;
const STATUS_RELEASED: i64 = 2;
const STATUS_REFUNDED: i64 = 3;

const KEY_STRIDE: i64 = 16;

fn make_key(escrow_id: i64, field: i64) -> i64 {
    escrow_id * KEY_STRIDE + field
}

/// Store an i64 value under the given key.
fn storage_put_i64(escrow_id: i64, field: i64, value: i64) -> bool {
    let key = make_key(escrow_id, field);
    RawStorage::put_i64_key(key, value);
    true
}

/// Load an i64 value from storage, returning 0 if absent.
fn storage_get_i64(escrow_id: i64, field: i64) -> i64 {
    let key = make_key(escrow_id, field);
    RawStorage::get_i64_key_or_zero(key)
}

// Events use primitive fields so wasm builds do not construct BigInt wrappers.
#[neo_event]
pub struct EscrowConfigured {
    pub escrow_id: i64,
    pub payer: i64,
    pub payee: i64,
    pub amount: i64,
}

#[neo_event]
pub struct EscrowReleased {
    pub escrow_id: i64,
}

#[neo_event]
pub struct EscrowRefunded {
    pub escrow_id: i64,
}

#[neo_contract]
pub struct NeoEscrowContract;

#[neo_contract]
impl NeoEscrowContract {
    pub fn new() -> Self {
        Self
    }

    /// Configure a new escrow. All accounts and the token are passed as i64 identifiers.
    #[neo_method]
    pub fn configure(
        escrow_id: i64,
        payer: i64,
        payee: i64,
        arbiter: i64,
        token: i64,
        amount: i64,
        release_height: i64,
        refund_height: i64,
    ) -> bool {
        if escrow_id <= 0
            || escrow_id > i64::MAX / KEY_STRIDE
            || amount <= 0
            || release_height < 0
            || refund_height < release_height
        {
            return false;
        }
        if payer <= 0 || payee <= 0 || arbiter <= 0 || token <= 0 {
            return false;
        }
        // The funding payer (and only the payer) may register an escrow on its
        // own behalf; require a runtime witness to prove the caller controls
        // the payer account (X1).
        if !NeoRuntime::require_witness_i64(payer) {
            return false;
        }
        // Prevent re-initialization
        if storage_get_i64(escrow_id, KEY_STATUS) != 0 {
            return false;
        }
        storage_put_i64(escrow_id, KEY_PAYER, payer);
        storage_put_i64(escrow_id, KEY_PAYEE, payee);
        storage_put_i64(escrow_id, KEY_ARBITER, arbiter);
        storage_put_i64(escrow_id, KEY_TOKEN, token);
        storage_put_i64(escrow_id, KEY_AMOUNT, amount);
        storage_put_i64(escrow_id, KEY_RELEASE_HEIGHT, release_height);
        storage_put_i64(escrow_id, KEY_REFUND_HEIGHT, refund_height);
        storage_put_i64(escrow_id, KEY_STATUS, STATUS_ACTIVE);
        let _ = (EscrowConfigured {
            escrow_id,
            payer,
            payee,
            amount,
        })
        .emit();
        true
    }

    /// Release escrow funds. Caller must be payer or arbiter.
    #[neo_method]
    pub fn release(escrow_id: i64, caller: i64) -> bool {
        if escrow_id <= 0 || caller <= 0 {
            return false;
        }
        // Caller identity must be runtime-witnessed, not trusted as a parameter
        // (X1: otherwise an attacker passes the arbiter's id and releases).
        if !NeoRuntime::require_witness_i64(caller) {
            return false;
        }
        let status = storage_get_i64(escrow_id, KEY_STATUS);
        if status != STATUS_ACTIVE {
            return false;
        }
        let arbiter = storage_get_i64(escrow_id, KEY_ARBITER);
        let payer = storage_get_i64(escrow_id, KEY_PAYER);
        if caller != arbiter && caller != payer {
            return false;
        }
        storage_put_i64(escrow_id, KEY_STATUS, STATUS_RELEASED);
        let _ = (EscrowReleased { escrow_id }).emit();
        true
    }

    /// Refund escrow. Caller must be payee or arbiter.
    #[neo_method]
    pub fn refund(escrow_id: i64, caller: i64) -> bool {
        if escrow_id <= 0 || caller <= 0 {
            return false;
        }
        // Witness the caller identity (X1).
        if !NeoRuntime::require_witness_i64(caller) {
            return false;
        }
        let status = storage_get_i64(escrow_id, KEY_STATUS);
        if status != STATUS_ACTIVE {
            return false;
        }
        let arbiter = storage_get_i64(escrow_id, KEY_ARBITER);
        let payee = storage_get_i64(escrow_id, KEY_PAYEE);
        if caller != arbiter && caller != payee {
            return false;
        }
        storage_put_i64(escrow_id, KEY_STATUS, STATUS_REFUNDED);
        let _ = (EscrowRefunded { escrow_id }).emit();
        true
    }

    /// Return escrow state via notify: [status, amount, release_h, refund_h]
    #[neo_method(safe, name = "getState")]
    pub fn get_state(escrow_id: i64) {
        let status = storage_get_i64(escrow_id, KEY_STATUS);
        if status == 0 {
            return;
        }
        let amount = storage_get_i64(escrow_id, KEY_AMOUNT);
        let release_h = storage_get_i64(escrow_id, KEY_RELEASE_HEIGHT);
        let refund_h = storage_get_i64(escrow_id, KEY_REFUND_HEIGHT);

        #[cfg(target_arch = "wasm32")]
        {
            let _ = (status, amount, release_h, refund_h);
            let _ = NeoRuntime::notify_event("getState");
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let label = NeoString::from_str("getState");
            let mut state = NeoArray::new();
            state.push(NeoValue::from(status));
            state.push(NeoValue::from(amount));
            state.push(NeoValue::from(release_h));
            state.push(NeoValue::from(refund_h));
            let _ = NeoRuntime::notify(&label, &state);
        }
    }

    #[neo_method(name = "onNEP17Payment")]
    pub fn on_nep17_payment(_from: i64, _amount: i64, _data: i64) {}
}

impl Default for NeoEscrowContract {
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
    fn configure_rejects_invalid_inputs() {
        let _g = setup_witnesses(&[1]);
        // escrow_id must be > 0
        assert!(!NeoEscrowContract::configure(0, 1, 2, 3, 4, 100, 10, 20));
        // amount must be > 0
        assert!(!NeoEscrowContract::configure(1, 1, 2, 3, 4, 0, 10, 20));
        // release_height must be >= 0
        assert!(!NeoEscrowContract::configure(1, 1, 2, 3, 4, 100, -1, 20));
        // refund_height must be >= release_height
        assert!(!NeoEscrowContract::configure(1, 1, 2, 3, 4, 100, 20, 10));
        // payer must be > 0
        assert!(!NeoEscrowContract::configure(1, 0, 2, 3, 4, 100, 10, 20));
        // payee must be > 0
        assert!(!NeoEscrowContract::configure(1, 1, 0, 3, 4, 100, 10, 20));
        // arbiter must be > 0
        assert!(!NeoEscrowContract::configure(1, 1, 2, 0, 4, 100, 10, 20));
        // token must be > 0
        assert!(!NeoEscrowContract::configure(1, 1, 2, 3, 0, 100, 10, 20));
    }

    #[test]
    fn configure_requires_payer_witness() {
        // Without payer in the witness set, configure must be rejected even
        // when all other inputs are valid (X1 authorization bypass).
        {
            let _g = setup_witnesses(&[]);
            assert!(!NeoEscrowContract::configure(1, 1, 2, 3, 4, 100, 10, 20));
        }
        // With payer witnessed, configure succeeds.
        {
            let _g = setup_witnesses(&[1]);
            assert!(NeoEscrowContract::configure(1, 1, 2, 3, 4, 100, 10, 20));
        }
    }

    #[test]
    fn release_refund_require_caller_witness() {
        // Single witness scope (reset_host_state clears storage between scopes).
        // Configure as payer=1, payee=2, arbiter=3 with both 1 and 3 witnessed.
        let _g = setup_witnesses(&[1, 3]);
        assert!(NeoEscrowContract::configure(2, 1, 2, 3, 4, 100, 10, 20));

        // A non-witnessed "caller" cannot release even by passing the arbiter's
        // id (X1). We simulate the attacker by temporarily clearing witnesses
        // is not possible in-scope; instead assert that a witnessed arbiter
        // CAN release (positive auth), then the status change blocks refund.
        assert!(NeoEscrowContract::release(2, 3));
        // Already released -> refund rejected regardless of caller.
        assert!(!NeoEscrowContract::refund(2, 3));
    }

    #[test]
    fn release_rejects_unwitnessed_caller() {
        // Configure under one scope, then prove a caller id that was NEVER
        // witnessed is rejected. Reset clears storage, so configure + the
        // negative release must share a scope. Use witnesses [1] (payer only);
        // caller=3 (arbiter) is not witnessed, so release(2, 3) must fail.
        let _g = setup_witnesses(&[1]);
        assert!(NeoEscrowContract::configure(3, 1, 2, 3, 4, 100, 10, 20));
        assert!(!NeoEscrowContract::release(3, 3));
        assert!(!NeoEscrowContract::refund(3, 3));
    }

    #[test]
    fn release_rejects_invalid_inputs() {
        assert!(!NeoEscrowContract::release(0, 1));
        assert!(!NeoEscrowContract::release(1, 0));
    }

    #[test]
    fn refund_rejects_invalid_inputs() {
        assert!(!NeoEscrowContract::refund(0, 1));
        assert!(!NeoEscrowContract::refund(1, 0));
    }

    #[test]
    fn make_key_deterministic() {
        let k1 = make_key(1, KEY_PAYER);
        let k2 = make_key(1, KEY_PAYER);
        assert_eq!(k1, k2);
        // Different field produces different key
        let k3 = make_key(1, KEY_PAYEE);
        assert_ne!(k1, k3);
        // Different escrow_id produces different key
        let k4 = make_key(2, KEY_PAYER);
        assert_ne!(k1, k4);
    }
}
