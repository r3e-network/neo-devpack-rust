use neo_devpack::prelude::*;

const RESERVE_X_KEY: &[u8] = b"amm:reserve_x";
const RESERVE_Y_KEY: &[u8] = b"amm:reserve_y";
const FEE_NUMERATOR: i64 = 997;
const FEE_DENOMINATOR: i64 = 1_000;

neo_manifest_overlay!(
    r#"{
    "name": "ConstantProductAMM",
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
                    {"name": "trader", "type": "Integer"},
                    {"name": "amount_in", "type": "Integer"}
                ],
                "returntype": "Integer"
            }
        ],
        "events": [
            {
                "name": "Swap",
                "parameters": [
                    {"name": "trader", "type": "Integer"},
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
    pub trader: NeoInteger,
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
    if reserve_x <= 0 || reserve_y <= 0 {
        return 0;
    }
    let amount_in_with_fee = amount_in * FEE_NUMERATOR;
    let numerator = amount_in_with_fee * reserve_y;
    let denominator = reserve_x * FEE_DENOMINATOR + amount_in_with_fee;
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
    let (x, y) = load_reserves(&ctx).unwrap_or((0, 0));
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

#[no_mangle]
pub extern "C" fn swap(trader: i64, amount_in: i64) -> i64 {
    if amount_in <= 0 {
        return 0;
    }
    let Some(ctx) = storage_context() else {
        return 0;
    };
    let witness = NeoByteString::from_slice(&trader.to_le_bytes());
    if !NeoRuntime::check_witness(&witness)
        .map(|flag| flag.as_bool())
        .unwrap_or(false)
    {
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
    let new_x = reserve_x + amount_in;
    let new_y = reserve_y - amount_out;
    if store_reserves(&ctx, new_x, new_y).is_err() {
        return 0;
    }
    SwapEvent {
        trader: NeoInteger::new(trader),
        amount_in: NeoInteger::new(amount_in),
        amount_out: NeoInteger::new(amount_out),
        new_reserve_x: NeoInteger::new(new_x),
        new_reserve_y: NeoInteger::new(new_y),
    }
    .emit()
    .ok();
    amount_out
}
