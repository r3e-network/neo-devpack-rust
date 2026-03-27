// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

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

/// Build a fixed-size storage key from escrow_id and field tag.
/// Layout: 8 bytes (escrow_id LE) + 8 bytes (field LE) = 16 bytes on the stack.
fn make_key(escrow_id: i64, field: i64) -> NeoByteString {
    let mut buf = [0u8; 16];
    let id_bytes = escrow_id.to_le_bytes();
    let field_bytes = field.to_le_bytes();
    let mut i = 0;
    while i < 8 {
        buf[i] = id_bytes[i];
        buf[8 + i] = field_bytes[i];
        i += 1;
    }
    NeoByteString::from_slice(&buf)
}

/// Store an i64 value under the given key.
fn storage_put_i64(ctx: &NeoStorageContext, escrow_id: i64, field: i64, value: i64) -> bool {
    let key = make_key(escrow_id, field);
    let val = NeoByteString::from_slice(&value.to_le_bytes());
    NeoStorage::put(ctx, &key, &val).is_ok()
}

/// Load an i64 value from storage, returning 0 if absent.
fn storage_get_i64(ctx: &NeoStorageContext, escrow_id: i64, field: i64) -> i64 {
    let key = make_key(escrow_id, field);
    match NeoStorage::get(ctx, &key) {
        Ok(data) => {
            let s = data.as_slice();
            if s.len() != 8 {
                return 0;
            }
            let mut buf = [0u8; 8];
            let mut i = 0;
            while i < 8 {
                buf[i] = s[i];
                i += 1;
            }
            i64::from_le_bytes(buf)
        }
        Err(_) => 0,
    }
}

// Events -- all fields are NeoInteger (i64-based), no NeoByteString
#[neo_event]
pub struct EscrowConfigured {
    pub escrow_id: NeoInteger,
    pub payer: NeoInteger,
    pub payee: NeoInteger,
    pub amount: NeoInteger,
}

#[neo_event]
pub struct EscrowReleased {
    pub escrow_id: NeoInteger,
}

#[neo_event]
pub struct EscrowRefunded {
    pub escrow_id: NeoInteger,
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
        if escrow_id <= 0 || amount <= 0 || release_height < 0 || refund_height < release_height {
            return false;
        }
        if payer <= 0 || payee <= 0 || arbiter <= 0 || token <= 0 {
            return false;
        }
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return false,
        };
        // Prevent re-initialization
        if storage_get_i64(&ctx, escrow_id, KEY_STATUS) != 0 {
            return false;
        }
        storage_put_i64(&ctx, escrow_id, KEY_PAYER, payer);
        storage_put_i64(&ctx, escrow_id, KEY_PAYEE, payee);
        storage_put_i64(&ctx, escrow_id, KEY_ARBITER, arbiter);
        storage_put_i64(&ctx, escrow_id, KEY_TOKEN, token);
        storage_put_i64(&ctx, escrow_id, KEY_AMOUNT, amount);
        storage_put_i64(&ctx, escrow_id, KEY_RELEASE_HEIGHT, release_height);
        storage_put_i64(&ctx, escrow_id, KEY_REFUND_HEIGHT, refund_height);
        storage_put_i64(&ctx, escrow_id, KEY_STATUS, STATUS_ACTIVE);
        let _ = (EscrowConfigured {
            escrow_id: NeoInteger::new(escrow_id),
            payer: NeoInteger::new(payer),
            payee: NeoInteger::new(payee),
            amount: NeoInteger::new(amount),
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
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return false,
        };
        let status = storage_get_i64(&ctx, escrow_id, KEY_STATUS);
        if status != STATUS_ACTIVE {
            return false;
        }
        let arbiter = storage_get_i64(&ctx, escrow_id, KEY_ARBITER);
        let payer = storage_get_i64(&ctx, escrow_id, KEY_PAYER);
        if caller != arbiter && caller != payer {
            return false;
        }
        storage_put_i64(&ctx, escrow_id, KEY_STATUS, STATUS_RELEASED);
        let _ = (EscrowReleased {
            escrow_id: NeoInteger::new(escrow_id),
        })
        .emit();
        true
    }

    /// Refund escrow. Caller must be payee or arbiter.
    #[neo_method]
    pub fn refund(escrow_id: i64, caller: i64) -> bool {
        if escrow_id <= 0 || caller <= 0 {
            return false;
        }
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return false,
        };
        let status = storage_get_i64(&ctx, escrow_id, KEY_STATUS);
        if status != STATUS_ACTIVE {
            return false;
        }
        let arbiter = storage_get_i64(&ctx, escrow_id, KEY_ARBITER);
        let payee = storage_get_i64(&ctx, escrow_id, KEY_PAYEE);
        if caller != arbiter && caller != payee {
            return false;
        }
        storage_put_i64(&ctx, escrow_id, KEY_STATUS, STATUS_REFUNDED);
        let _ = (EscrowRefunded {
            escrow_id: NeoInteger::new(escrow_id),
        })
        .emit();
        true
    }

    /// Return escrow state via notify: [status, amount, release_h, refund_h]
    #[neo_method(safe, name = "getState")]
    pub fn get_state(escrow_id: i64) {
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return,
        };
        let status = storage_get_i64(&ctx, escrow_id, KEY_STATUS);
        if status == 0 {
            return;
        }
        let amount = storage_get_i64(&ctx, escrow_id, KEY_AMOUNT);
        let release_h = storage_get_i64(&ctx, escrow_id, KEY_RELEASE_HEIGHT);
        let refund_h = storage_get_i64(&ctx, escrow_id, KEY_REFUND_HEIGHT);
        let label = NeoString::from_str("getState");
        let mut state = NeoArray::new();
        state.push(NeoValue::from(NeoInteger::new(status)));
        state.push(NeoValue::from(NeoInteger::new(amount)));
        state.push(NeoValue::from(NeoInteger::new(release_h)));
        state.push(NeoValue::from(NeoInteger::new(refund_h)));
        let _ = NeoRuntime::notify(&label, &state);
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

    #[test]
    fn contract_compiles() {
        // Compilation test - verifies contract module parses correctly
    }

    #[test]
    fn configure_rejects_invalid_inputs() {
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
        assert_eq!(k1.as_slice(), k2.as_slice());
        // Different field produces different key
        let k3 = make_key(1, KEY_PAYEE);
        assert_ne!(k1.as_slice(), k3.as_slice());
        // Different escrow_id produces different key
        let k4 = make_key(2, KEY_PAYER);
        assert_ne!(k1.as_slice(), k4.as_slice());
    }
}
