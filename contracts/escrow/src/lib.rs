// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "NeoEscrow"
}"#
);

// Storage keys
const CONFIG_PREFIX: &[u8] = b"escrow:";
const PAYER_SUFFIX: &[u8] = b":payer";
const PAYEE_SUFFIX: &[u8] = b":payee";
const ARBITER_SUFFIX: &[u8] = b":arbiter";
const TOKEN_SUFFIX: &[u8] = b":token";
const AMOUNT_SUFFIX: &[u8] = b":amount";
const RELEASE_HEIGHT_SUFFIX: &[u8] = b":release_h";
const REFUND_HEIGHT_SUFFIX: &[u8] = b":refund_h";
const STATUS_SUFFIX: &[u8] = b":status";

// Status constants
const STATUS_ACTIVE: u8 = 1;
const STATUS_RELEASED: u8 = 2;
const STATUS_REFUNDED: u8 = 3;

fn escrow_key(id: i64, suffix: &[u8]) -> Vec<u8> {
    let mut key = CONFIG_PREFIX.to_vec();
    key.extend_from_slice(&id.to_le_bytes());
    key.extend_from_slice(suffix);
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

fn storage_put_u8(ctx: &NeoStorageContext, key: &[u8], value: u8) -> bool {
    storage_put_bytes(ctx, key, &[value])
}

fn storage_get_u8(ctx: &NeoStorageContext, key: &[u8]) -> Option<u8> {
    let bytes = storage_get_bytes(ctx, key)?;
    if bytes.len() != 1 {
        return None;
    }
    Some(bytes[0])
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
pub struct EscrowConfigured {
    pub escrow_id: NeoInteger,
    pub payer: NeoByteString,
    pub payee: NeoByteString,
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

    #[neo_method]
    pub fn configure(
        escrow_id: i64,
        payer_ptr: i64,
        payer_len: i64,
        payee_ptr: i64,
        payee_len: i64,
        arbiter_ptr: i64,
        arbiter_len: i64,
        token_ptr: i64,
        token_len: i64,
        amount: i64,
        release_height: i64,
        refund_height: i64,
    ) -> bool {
        if escrow_id <= 0 || amount <= 0 || release_height < 0 || refund_height < release_height {
            return false;
        }
        let payer = match read_address(payer_ptr, payer_len) {
            Some(a) => a,
            None => return false,
        };
        if !ensure_witness(&payer) {
            return false;
        }
        let payee = match read_address(payee_ptr, payee_len) {
            Some(a) => a,
            None => return false,
        };
        let arbiter = match read_address(arbiter_ptr, arbiter_len) {
            Some(a) => a,
            None => return false,
        };
        let token = match read_address(token_ptr, token_len) {
            Some(a) => a,
            None => return false,
        };
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return false,
        };
        // Prevent re-initialization
        if storage_get_u8(&ctx, &escrow_key(escrow_id, STATUS_SUFFIX)).is_some() {
            return false;
        }
        storage_put_bytes(&ctx, &escrow_key(escrow_id, PAYER_SUFFIX), payer.as_slice());
        storage_put_bytes(&ctx, &escrow_key(escrow_id, PAYEE_SUFFIX), payee.as_slice());
        storage_put_bytes(&ctx, &escrow_key(escrow_id, ARBITER_SUFFIX), arbiter.as_slice());
        storage_put_bytes(&ctx, &escrow_key(escrow_id, TOKEN_SUFFIX), token.as_slice());
        storage_put_i64(&ctx, &escrow_key(escrow_id, AMOUNT_SUFFIX), amount);
        storage_put_i64(&ctx, &escrow_key(escrow_id, RELEASE_HEIGHT_SUFFIX), release_height);
        storage_put_i64(&ctx, &escrow_key(escrow_id, REFUND_HEIGHT_SUFFIX), refund_height);
        storage_put_u8(&ctx, &escrow_key(escrow_id, STATUS_SUFFIX), STATUS_ACTIVE);
        let _ = (EscrowConfigured {
            escrow_id: NeoInteger::new(escrow_id),
            payer,
            payee,
            amount: NeoInteger::new(amount),
        })
        .emit();
        true
    }

    #[neo_method]
    pub fn release(escrow_id: i64, caller_ptr: i64, caller_len: i64) -> bool {
        if escrow_id <= 0 {
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
        let status = match storage_get_u8(&ctx, &escrow_key(escrow_id, STATUS_SUFFIX)) {
            Some(s) => s,
            None => return false,
        };
        if status != STATUS_ACTIVE {
            return false;
        }
        let arbiter = match storage_get_bytes(&ctx, &escrow_key(escrow_id, ARBITER_SUFFIX)) {
            Some(b) => b,
            None => return false,
        };
        let payer = match storage_get_bytes(&ctx, &escrow_key(escrow_id, PAYER_SUFFIX)) {
            Some(b) => b,
            None => return false,
        };
        if caller.as_slice() != arbiter.as_slice() && caller.as_slice() != payer.as_slice() {
            return false;
        }
        storage_put_u8(&ctx, &escrow_key(escrow_id, STATUS_SUFFIX), STATUS_RELEASED);
        let _ = (EscrowReleased {
            escrow_id: NeoInteger::new(escrow_id),
        })
        .emit();
        true
    }

    #[neo_method]
    pub fn refund(escrow_id: i64, caller_ptr: i64, caller_len: i64) -> bool {
        if escrow_id <= 0 {
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
        let status = match storage_get_u8(&ctx, &escrow_key(escrow_id, STATUS_SUFFIX)) {
            Some(s) => s,
            None => return false,
        };
        if status != STATUS_ACTIVE {
            return false;
        }
        let arbiter = match storage_get_bytes(&ctx, &escrow_key(escrow_id, ARBITER_SUFFIX)) {
            Some(b) => b,
            None => return false,
        };
        let payee = match storage_get_bytes(&ctx, &escrow_key(escrow_id, PAYEE_SUFFIX)) {
            Some(b) => b,
            None => return false,
        };
        if caller.as_slice() != arbiter.as_slice() && caller.as_slice() != payee.as_slice() {
            return false;
        }
        storage_put_u8(&ctx, &escrow_key(escrow_id, STATUS_SUFFIX), STATUS_REFUNDED);
        let _ = (EscrowRefunded {
            escrow_id: NeoInteger::new(escrow_id),
        })
        .emit();
        true
    }

    /// Return escrow state via notify: [status, amount, release_h, refund_h]
    #[neo_method(name = "getState")]
    pub fn get_state(escrow_id: i64) {
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return,
        };
        let status = match storage_get_u8(&ctx, &escrow_key(escrow_id, STATUS_SUFFIX)) {
            Some(s) => s,
            None => return,
        };
        let amount = storage_get_i64(&ctx, &escrow_key(escrow_id, AMOUNT_SUFFIX)).unwrap_or(0);
        let release_h = storage_get_i64(&ctx, &escrow_key(escrow_id, RELEASE_HEIGHT_SUFFIX)).unwrap_or(0);
        let refund_h = storage_get_i64(&ctx, &escrow_key(escrow_id, REFUND_HEIGHT_SUFFIX)).unwrap_or(0);
        let label = NeoString::from_str("getState");
        let mut state = NeoArray::new();
        state.push(NeoValue::from(NeoInteger::new(status as i64)));
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
    #[test]
    fn contract_compiles() {
        // Compilation test - verifies contract module parses correctly
    }
}
