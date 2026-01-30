use neo_devpack::prelude::*;

const RESERVE_X_KEY: &[u8] = b"amm:reserve_x";
const RESERVE_Y_KEY: &[u8] = b"amm:reserve_y";
const FEE_NUMERATOR: i64 = 997;
const FEE_DENOMINATOR: i64 = 1_000;

neo_manifest_overlay!(
    r#"{
    "name": "ConstantProductAMM",
    "supportedstandards": ["NEP-17"],
    "features": { "storage": true },
    "abi": {
        "methods": [
            {
                "name": "init",
                "parameters": [
                    {"name": "initial_x", "type": "Integer"},
                    {"name": "initial_y", "type": "Integer"}
                ],
                "returntype": "Boolean"
            },
            {
                "name": "getReserves",
                "parameters": [],
                "returntype": "Integer"
            },
            {
                "name": "quote",
                "parameters": [
                    {"name": "amount_in", "type": "Integer"}
                ],
                "returntype": "Integer"
            },
            {
                "name": "swap",
                "parameters": [
                    {"name": "trader", "type": "Hash160"},
                    {"name": "amount_in", "type": "Integer"}
                ],
                "returntype": "Integer"
            }
        ],
        "events": [
            {
                "name": "Swap",
                "parameters": [
                    {"name": "trader", "type": "Hash160"},
                    {"name": "amount_in", "type": "Integer"},
                    {"name": "amount_out", "type": "Integer"}
                ]
            }
        ]
    }
}"#
);

#[neo_event]
pub struct SwapEvent {
    pub trader: NeoByteString,
    pub amount_in: NeoInteger,
    pub amount_out: NeoInteger,
    pub new_reserve_x: NeoInteger,
    pub new_reserve_y: NeoInteger,
}

fn storage_context() -> Option<NeoStorageContext> {
    NeoStorage::get_context().ok()
}

fn read_i64(bytes: &NeoByteString) -> i64 {
    let data = bytes.as_slice();
    if data.is_empty() {
        0
    } else {
        let mut buf = [0u8; 8];
        let len = data.len().min(8);
        buf[..len].copy_from_slice(&data[..len]);
        i64::from_le_bytes(buf)
    }
}

fn write_i64(value: i64) -> NeoByteString {
    NeoByteString::from_slice(&value.to_le_bytes())
}

fn load_reserves(ctx: &NeoStorageContext) -> NeoResult<(i64, i64)> {
    let x_key = NeoByteString::from_slice(RESERVE_X_KEY);
    let y_key = NeoByteString::from_slice(RESERVE_Y_KEY);
    let x = read_i64(&NeoStorage::get(ctx, &x_key)?);
    let y = read_i64(&NeoStorage::get(ctx, &y_key)?);
    Ok((x, y))
}

fn store_reserves(ctx: &NeoStorageContext, x: i64, y: i64) -> NeoResult<()> {
    let x_key = NeoByteString::from_slice(RESERVE_X_KEY);
    let y_key = NeoByteString::from_slice(RESERVE_Y_KEY);
    NeoStorage::put(ctx, &x_key, &write_i64(x))?;
    NeoStorage::put(ctx, &y_key, &write_i64(y))
}

fn calculate_swap_output(reserve_x: i64, reserve_y: i64, amount_in: i64) -> i64 {
    if reserve_x <= 0 || reserve_y <= 0 || amount_in <= 0 {
        return 0;
    }
    let amount_in_with_fee = match amount_in.checked_mul(FEE_NUMERATOR) {
        Some(value) => value,
        None => return 0,
    };
    let numerator = match amount_in_with_fee.checked_mul(reserve_y) {
        Some(value) => value,
        None => return 0,
    };
    let denominator = match reserve_x.checked_mul(FEE_DENOMINATOR) {
        Some(x_fee) => match x_fee.checked_add(amount_in_with_fee) {
            Some(value) => value,
            None => return 0,
        },
        None => return 0,
    };
    if denominator == 0 {
        0
    } else {
        numerator / denominator
    }
}

#[no_mangle]
pub extern "C" fn init(initial_x: i64, initial_y: i64) -> i64 {
    if initial_x <= 0 || initial_y <= 0 {
        return 0;
    }
    let Some(ctx) = storage_context() else {
        return 0;
    };
    let (x, y) = match load_reserves(&ctx) {
        Ok(reserves) => reserves,
        Err(_) => (0, 0),
    };
    if x != 0 || y != 0 {
        return 0; // already initialised
    }
    if store_reserves(&ctx, initial_x, initial_y).is_err() {
        return 0;
    }
    1
}

#[no_mangle]
#[neo_safe]
pub extern "C" fn getReserves() -> i64 {
    storage_context()
        .and_then(|ctx| load_reserves(&ctx).ok())
        .map(|(x, y)| (x << 32) | (y & 0xFFFF_FFFF))
        .unwrap_or(0)
}

#[no_mangle]
#[neo_safe]
pub extern "C" fn quote(amount_in: i64) -> i64 {
    if amount_in <= 0 {
        return 0;
    }
    storage_context()
        .and_then(|ctx| load_reserves(&ctx).ok())
        .map(|(x, y)| calculate_swap_output(x, y, amount_in))
        .unwrap_or(0)
}

#[allow(improper_ctypes_definitions)]
#[no_mangle]
pub extern "C" fn swap(trader_ptr: i64, trader_len: i64, amount_in: i64) -> i64 {
    if amount_in <= 0 {
        return 0;
    }
    let Some(ctx) = storage_context() else {
        return 0;
    };
    let Some(trader) = read_address(trader_ptr, trader_len) else {
        return 0;
    };
    if !ensure_witness(&trader) {
        return 0;
    }
    let (reserve_x, reserve_y) = match load_reserves(&ctx) {
        Ok(values) => values,
        Err(_) => return 0,
    };
    let amount_out = calculate_swap_output(reserve_x, reserve_y, amount_in);
    if amount_out <= 0 || amount_out >= reserve_y {
        return 0;
    }
    let new_x = match reserve_x.checked_add(amount_in) {
        Some(value) => value,
        None => return 0,
    };
    let new_y = match reserve_y.checked_sub(amount_out) {
        Some(value) => value,
        None => return 0,
    };
    if store_reserves(&ctx, new_x, new_y).is_err() {
        return 0;
    }
    SwapEvent {
        trader,
        amount_in: NeoInteger::new(amount_in),
        amount_out: NeoInteger::new(amount_out),
        new_reserve_x: NeoInteger::new(new_x),
        new_reserve_y: NeoInteger::new(new_y),
    }
    .emit()
    .ok();
    amount_out
}

#[allow(improper_ctypes_definitions)]
fn read_address(ptr: i64, len: i64) -> Option<NeoByteString> {
    let bytes = read_bytes(ptr, len)?;
    if bytes.len() != 20 {
        return None;
    }
    Some(NeoByteString::from_slice(&bytes))
}

fn read_bytes(ptr: i64, len: i64) -> Option<Vec<u8>> {
    if ptr == 0 || len <= 0 {
        return None;
    }
    let len = len as usize;
    let slice = unsafe { core::slice::from_raw_parts(ptr as *const u8, len) };
    Some(slice.to_vec())
}

fn ensure_witness(account: &NeoByteString) -> bool {
    NeoRuntime::check_witness(account)
        .ok()
        .map(|b| b.as_bool())
        .unwrap_or(false)
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
        NeoStorage::delete(&ctx, &NeoByteString::from_slice(RESERVE_X_KEY)).ok();
        NeoStorage::delete(&ctx, &NeoByteString::from_slice(RESERVE_Y_KEY)).ok();
    }

    #[test]
    fn init_stores_reserves() {
        let _guard = test_lock().lock().unwrap();
        reset_state();
        assert_eq!(init(1000, 2000), 1);
        assert_eq!(getReserves(), (1000i64 << 32) | 2000u32 as i64);
    }

    #[test]
    fn quote_calculates_correct_output() {
        let _guard = test_lock().lock().unwrap();
        reset_state();
        init(1000000, 1000000);
        let output = quote(1000);
        assert!(output > 0);
        assert!(output < 1000);
    }

    #[test]
    fn swap_fails_without_witness() {
        let _guard = test_lock().lock().unwrap();
        reset_state();
        init(1000000, 1000000);
        let trader = address(0x42);
        let result = swap(trader.as_ptr() as i64, trader.len() as i64, 1000);
        assert_eq!(result, 0);
    }
}
