use core::slice;
use neo_devpack::{codec, prelude::*};
use serde::{Deserialize, Serialize};

const STATE_KEY: &[u8] = b"escrow:state";

#[derive(Clone, Serialize, Deserialize)]
struct EscrowState {
    payer: NeoByteString,
    payee: NeoByteString,
    arbiter: NeoByteString,
    token: NeoByteString,
    amount: i64,
    funded: bool,
    released: bool,
}

neo_manifest_overlay!(
    r#"{
    "name": "NeoEscrow",
    "supportedstandards": ["NEP-17"],
    "features": { "storage": true }
}"#
);

#[neo_event]
pub struct EscrowConfigured {
    pub payer: NeoByteString,
    pub payee: NeoByteString,
    pub arbiter: NeoByteString,
    pub amount: NeoInteger,
}

#[neo_event]
pub struct EscrowFunded {
    pub payer: NeoByteString,
    pub amount: NeoInteger,
}

#[neo_event]
pub struct EscrowReleased {
    pub payee: NeoByteString,
    pub amount: NeoInteger,
}

#[neo_event]
pub struct EscrowRefunded {
    pub payer: NeoByteString,
    pub amount: NeoInteger,
}

#[allow(improper_ctypes_definitions)]
#[neo_safe]
#[no_mangle]
pub extern "C" fn getState() -> NeoByteString {
    storage_context()
        .and_then(|ctx| load_state(&ctx))
        .map(|state| serialize_value(&state))
        .unwrap_or_else(|| NeoByteString::new(Vec::new()))
}

#[no_mangle]
pub extern "C" fn configure(
    payer_ptr: i64,
    payer_len: i64,
    payee_ptr: i64,
    payee_len: i64,
    arbiter_ptr: i64,
    arbiter_len: i64,
    token_ptr: i64,
    token_len: i64,
    amount: i64,
) -> i64 {
    if amount <= 0 {
        return 0;
    }

    let Some(ctx) = storage_context() else {
        return 0;
    };
    if load_state(&ctx).is_some() {
        return 0;
    }

    let Some(payer) = read_address(payer_ptr, payer_len) else {
        return 0;
    };
    let Some(payee) = read_address(payee_ptr, payee_len) else {
        return 0;
    };
    let Some(arbiter) = read_address(arbiter_ptr, arbiter_len) else {
        return 0;
    };
    let Some(token) = read_address(token_ptr, token_len) else {
        return 0;
    };

    let state = EscrowState {
        payer: payer.clone(),
        payee: payee.clone(),
        arbiter: arbiter.clone(),
        token: token.clone(),
        amount,
        funded: false,
        released: false,
    };

    if store_state(&ctx, &state).is_err() {
        return 0;
    }

    EscrowConfigured {
        payer,
        payee,
        arbiter,
        amount: NeoInteger::new(amount),
    }
    .emit()
    .ok();

    1
}

#[no_mangle]
pub extern "C" fn release(signer_ptr: i64, signer_len: i64) -> i64 {
    let Some(ctx) = storage_context() else {
        return 0;
    };
    let Some(mut state) = load_state(&ctx) else {
        return 0;
    };
    if !state.funded || state.released {
        return 0;
    }

    let Some(signer) = read_address(signer_ptr, signer_len) else {
        return 0;
    };
    if !addresses_equal(&signer, &state.arbiter) || !ensure_witness(&signer) {
        return 0;
    }

    let contract_hash = match NeoRuntime::get_executing_script_hash() {
        Ok(hash) => hash,
        Err(_) => return 0,
    };

    if !call_transfer(&state.token, &contract_hash, &state.payee, state.amount) {
        return 0;
    }

    state.released = true;
    if store_state(&ctx, &state).is_err() {
        return 0;
    }

    EscrowReleased {
        payee: state.payee.clone(),
        amount: NeoInteger::new(state.amount),
    }
    .emit()
    .ok();

    1
}

#[no_mangle]
pub extern "C" fn refund(signer_ptr: i64, signer_len: i64) -> i64 {
    let Some(ctx) = storage_context() else {
        return 0;
    };
    let Some(mut state) = load_state(&ctx) else {
        return 0;
    };
    if !state.funded || state.released {
        return 0;
    }

    let Some(signer) = read_address(signer_ptr, signer_len) else {
        return 0;
    };
    if !addresses_equal(&signer, &state.arbiter) || !ensure_witness(&signer) {
        return 0;
    }

    let contract_hash = match NeoRuntime::get_executing_script_hash() {
        Ok(hash) => hash,
        Err(_) => return 0,
    };

    if !call_transfer(&state.token, &contract_hash, &state.payer, state.amount) {
        return 0;
    }

    state.funded = false;
    state.released = false;
    if store_state(&ctx, &state).is_err() {
        return 0;
    }

    EscrowRefunded {
        payer: state.payer.clone(),
        amount: NeoInteger::new(state.amount),
    }
    .emit()
    .ok();

    1
}

#[allow(improper_ctypes_definitions)]
#[no_mangle]
pub extern "C" fn onNEP17Payment(from: NeoByteString, amount: i64, _data: NeoByteString) {
    let Some(ctx) = storage_context() else {
        return;
    };
    let Some(mut state) = load_state(&ctx) else {
        return;
    };
    if state.funded || amount != state.amount {
        return;
    }

    let Ok(call_hash) = NeoRuntime::get_calling_script_hash() else {
        return;
    };
    if !addresses_equal(&call_hash, &state.token) {
        return;
    }
    if !addresses_equal(&from, &state.payer) {
        return;
    }

    state.funded = true;
    if store_state(&ctx, &state).is_err() {
        return;
    }

    EscrowFunded {
        payer: from,
        amount: NeoInteger::new(amount),
    }
    .emit()
    .ok();
}

fn storage_context() -> Option<NeoStorageContext> {
    NeoStorage::get_context().ok()
}

fn load_state(ctx: &NeoStorageContext) -> Option<EscrowState> {
    load_from_storage(ctx, STATE_KEY)
}

fn store_state(ctx: &NeoStorageContext, state: &EscrowState) -> NeoResult<()> {
    store_to_storage(ctx, STATE_KEY, state)
}

fn read_address(ptr: i64, len: i64) -> Option<NeoByteString> {
    let bytes = read_bytes(ptr, len)?;
    if bytes.len() != 20 {
        return None;
    }
    Some(NeoByteString::from_slice(&bytes))
}

/// Reads bytes from a raw pointer.
/// 
/// # Safety
/// 
/// The caller must ensure that:
/// - `ptr` is a valid, non-null pointer allocated by the NeoVM runtime
/// - `len` bytes starting at `ptr` are valid for reads
/// 
/// These invariants are guaranteed when called from NeoVM contract entry points.
fn read_bytes(ptr: i64, len: i64) -> Option<Vec<u8>> {
    if ptr == 0 || len <= 0 {
        return None;
    }
    let len = len as usize;
    // SAFETY: We've validated ptr is non-null and len is positive.
    // The pointer validity is guaranteed by the NeoVM runtime.
    let slice = unsafe { slice::from_raw_parts(ptr as *const u8, len) };
    Some(slice.to_vec())
}

fn ensure_witness(account: &NeoByteString) -> bool {
    NeoRuntime::check_witness(account)
        .map(|flag| flag.as_bool())
        .unwrap_or(false)
}

fn addresses_equal(left: &NeoByteString, right: &NeoByteString) -> bool {
    left.as_slice() == right.as_slice()
}

fn call_transfer(
    token: &NeoByteString,
    from: &NeoByteString,
    to: &NeoByteString,
    amount: i64,
) -> bool {
    let mut args = NeoArray::new();
    args.push(NeoValue::from(from.clone()));
    args.push(NeoValue::from(to.clone()));
    args.push(NeoValue::from(NeoInteger::new(amount)));

    match NeoContractRuntime::call(token, &NeoString::from_str("transfer"), &args) {
        Ok(value) => value
            .as_boolean()
            .map(|flag| flag.as_bool())
            .unwrap_or(true),
        Err(_) => false,
    }
}

fn load_from_storage<T>(ctx: &NeoStorageContext, key: &[u8]) -> Option<T>
where
    T: for<'de> Deserialize<'de>,
{
    let key_bytes = NeoByteString::from_slice(key);
    let data = NeoStorage::get(ctx, &key_bytes).ok()?;
    if data.is_empty() {
        return None;
    }
    codec::deserialize(data.as_slice()).ok()
}

fn store_to_storage<T>(ctx: &NeoStorageContext, key: &[u8], value: &T) -> NeoResult<()>
where
    T: Serialize,
{
    let encoded = codec::serialize(value)?;
    let key_bytes = NeoByteString::from_slice(key);
    let value_bytes = NeoByteString::from_slice(&encoded);
    NeoStorage::put(ctx, &key_bytes, &value_bytes)
}

fn serialize_value<T: Serialize>(value: &T) -> NeoByteString {
    match codec::serialize(value) {
        Ok(bytes) => NeoByteString::from_slice(&bytes),
        Err(_) => NeoByteString::new(Vec::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn address(byte: u8) -> Vec<u8> {
        vec![byte; 20]
    }

    fn reset_state() {
        let ctx = storage_context().unwrap();
        NeoStorage::delete(&ctx, &NeoByteString::from_slice(STATE_KEY)).ok();
    }

    fn call_configure(amount: i64) -> EscrowState {
        reset_state();
        let payer = address(0x11);
        let payee = address(0x22);
        let arbiter = address(0x33);
        let token = address(0x00);

        let status = configure(
            payer.as_ptr() as i64,
            payer.len() as i64,
            payee.as_ptr() as i64,
            payee.len() as i64,
            arbiter.as_ptr() as i64,
            arbiter.len() as i64,
            token.as_ptr() as i64,
            token.len() as i64,
            amount,
        );
        assert_eq!(status, 1);

        let state_bytes = getState();
        codec::deserialize(state_bytes.as_slice()).expect("state decode")
    }

    #[test]
    fn configure_persists_state() {
        let _guard = test_lock().lock().unwrap();
        let state = call_configure(500);
        assert_eq!(state.amount, 500);
        assert!(!state.funded);
        assert!(!state.released);
    }

    #[test]
    fn funding_and_release_flow() {
        let _guard = test_lock().lock().unwrap();
        let _state = call_configure(1_000);
        let ctx = storage_context().unwrap();
        let mut stored = load_state(&ctx).unwrap();
        assert_eq!(stored.amount, 1_000);

        let payer = stored.payer.clone();
        onNEP17Payment(payer.clone(), 1_000, NeoByteString::new(Vec::new()));

        stored = load_state(&ctx).unwrap();
        assert!(stored.funded);
        assert!(!stored.released);

        let arbiter = stored.arbiter.clone();
        let arbiter_bytes = arbiter.as_slice().to_vec();
        let result = release(arbiter_bytes.as_ptr() as i64, arbiter_bytes.len() as i64);
        assert_eq!(result, 1);

        let final_state = load_state(&ctx).unwrap();
        assert!(final_state.released);
    }

    #[test]
    fn refund_resets_state() {
        let _guard = test_lock().lock().unwrap();
        let state = call_configure(250);
        let ctx = storage_context().unwrap();
        let payer = state.payer.clone();
        onNEP17Payment(payer, 250, NeoByteString::new(Vec::new()));

        let arbiter = state.arbiter.clone();
        let arbiter_bytes = arbiter.as_slice().to_vec();
        let status = refund(arbiter_bytes.as_ptr() as i64, arbiter_bytes.len() as i64);
        assert_eq!(status, 1);

        let final_state = load_state(&ctx).unwrap();
        assert!(!final_state.funded);
        assert!(!final_state.released);
    }
}
