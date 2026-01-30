use neo_devpack::prelude::*;

const TOTAL_SUPPLY_KEY: &[u8] = b"token:total_supply";
const BALANCE_PREFIX: &[u8] = b"token:balance:";

neo_manifest_overlay!(r#"{
    "name": "SampleNEP17",
    "supportedstandards": ["NEP-17"],
    "features": { "storage": true },
    "abi": {
        "methods": [
            {
                "name": "init",
                "parameters": [
                    {"name": "owner", "type": "Integer"},
                    {"name": "amount", "type": "Integer"}
                ],
                "returntype": "Boolean"
            },
            {
                "name": "transfer",
                "parameters": [
                    {"name": "from", "type": "Integer"},
                    {"name": "to", "type": "Integer"},
                    {"name": "amount", "type": "Integer"}
                ],
                "returntype": "Boolean"
            },
            {
                "name": "onNEP17Payment",
                "parameters": [
                    {"name": "from", "type": "Integer"},
                    {"name": "amount", "type": "Integer"},
                    {"name": "data", "type": "Integer"}
                ],
                "returntype": "Void"
            }
        ],
        "events": [
            {
                "name": "Transfer",
                "parameters": [
                    {"name": "from", "type": "Integer"},
                    {"name": "to", "type": "Integer"},
                    {"name": "amount", "type": "Integer"}
                ]
            }
        ]
    }
}"#);

#[neo_event]
pub struct TransferEvent {
    pub from: NeoInteger,
    pub to: NeoInteger,
    pub amount: NeoInteger,
}

fn storage_context() -> Option<NeoStorageContext> {
    NeoStorage::get_context().ok()
}

fn balance_key(account: i64) -> Vec<u8> {
    let mut key = BALANCE_PREFIX.to_vec();
    key.extend_from_slice(&account.to_le_bytes());
    key
}

fn read_i64(bytes: &NeoByteString) -> i64 {
    let raw = bytes.as_slice();
    if raw.is_empty() {
        0
    } else {
        let mut buffer = [0u8; 8];
        let copy_len = raw.len().min(8);
        buffer[..copy_len].copy_from_slice(&raw[..copy_len]);
        i64::from_le_bytes(buffer)
    }
}

fn write_i64(value: i64) -> NeoByteString {
    NeoByteString::from_slice(&value.to_le_bytes())
}

fn load_total_supply(ctx: &NeoStorageContext) -> NeoResult<i64> {
    let key = NeoByteString::from_slice(TOTAL_SUPPLY_KEY);
    Ok(read_i64(&NeoStorage::get(ctx, &key)?))
}

fn store_total_supply(ctx: &NeoStorageContext, value: i64) -> NeoResult<()> {
    let key = NeoByteString::from_slice(TOTAL_SUPPLY_KEY);
    let encoded = write_i64(value);
    NeoStorage::put(ctx, &key, &encoded)
}

fn load_balance(ctx: &NeoStorageContext, account: i64) -> NeoResult<i64> {
    let key = NeoByteString::from_slice(&balance_key(account));
    Ok(read_i64(&NeoStorage::get(ctx, &key)?))
}

fn store_balance(ctx: &NeoStorageContext, account: i64, value: i64) -> NeoResult<()> {
    let key = NeoByteString::from_slice(&balance_key(account));
    if value == 0 {
        NeoStorage::delete(ctx, &key)
    } else {
        let encoded = write_i64(value);
        NeoStorage::put(ctx, &key, &encoded)
    }
}

#[no_mangle]
pub extern "C" fn init(owner: i64, amount: i64) -> i64 {
    if amount <= 0 {
        return 0;
    }

    let Some(ctx) = storage_context() else {
        return 0;
    };

    if load_total_supply(&ctx).unwrap_or(0) != 0 {
        return 0; // already initialised
    }

    if store_total_supply(&ctx, amount).is_err() {
        return 0;
    }

    if store_balance(&ctx, owner, amount).is_err() {
        return 0;
    }

    TransferEvent {
        from: NeoInteger::new(0),
        to: NeoInteger::new(owner),
        amount: NeoInteger::new(amount),
    }
    .emit()
    .ok();

    1
}

#[no_mangle]
#[neo_safe]
pub extern "C" fn totalSupply() -> i64 {
    storage_context()
        .and_then(|ctx| load_total_supply(&ctx).ok())
        .unwrap_or(0)
}

#[no_mangle]
#[neo_safe]
pub extern "C" fn balanceOf(account: i64) -> i64 {
    storage_context()
        .and_then(|ctx| load_balance(&ctx, account).ok())
        .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn transfer(from: i64, to: i64, amount: i64) -> i64 {
    if amount <= 0 || from == to {
        return 0;
    }

    let Some(ctx) = storage_context() else {
        return 0;
    };

    let witness = NeoByteString::from_slice(&from.to_le_bytes());
    if !NeoRuntime::check_witness(&witness)
        .map(|flag| flag.as_bool())
        .unwrap_or(false)
    {
        return 0;
    }

    let from_balance = match load_balance(&ctx, from) {
        Ok(value) => value,
        Err(_) => return 0,
    };

    if from_balance < amount {
        return 0;
    }

    let to_balance = load_balance(&ctx, to).unwrap_or(0);

    if store_balance(&ctx, from, from_balance - amount).is_err() {
        return 0;
    }
    if store_balance(&ctx, to, to_balance + amount).is_err() {
        return 0;
    }

    TransferEvent {
        from: NeoInteger::new(from),
        to: NeoInteger::new(to),
        amount: NeoInteger::new(amount),
    }
    .emit()
    .ok();

    1
}

#[no_mangle]
pub extern "C" fn onNEP17Payment(_from: i64, _amount: i64, _data: i64) {}
