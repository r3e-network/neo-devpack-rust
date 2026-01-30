use core::slice;
use neo_devpack::prelude::*;

const TOTAL_SUPPLY_KEY: &[u8] = b"token:total_supply";
const BALANCE_PREFIX: &[u8] = b"token:balance:";

neo_manifest_overlay!(
    r#"{
    "name": "SampleNEP17",
    "supportedstandards": ["NEP-17"],
    "features": { "storage": true },
    "abi": {
        "methods": [
            {
                "name": "init",
                "parameters": [
                    {"name": "owner", "type": "Hash160"},
                    {"name": "amount", "type": "Integer"}
                ],
                "returntype": "Boolean"
            },
            {
                "name": "transfer",
                "parameters": [
                    {"name": "from", "type": "Hash160"},
                    {"name": "to", "type": "Hash160"},
                    {"name": "amount", "type": "Integer"},
                    {"name": "data", "type": "Any"}
                ],
                "returntype": "Boolean"
            },
            {
                "name": "balanceOf",
                "parameters": [
                    {"name": "account", "type": "Hash160"}
                ],
                "returntype": "Integer"
            },
            {
                "name": "totalSupply",
                "parameters": [],
                "returntype": "Integer"
            }
        ],
        "events": [
            {
                "name": "Transfer",
                "parameters": [
                    {"name": "from", "type": "Hash160"},
                    {"name": "to", "type": "Hash160"},
                    {"name": "amount", "type": "Integer"}
                ]
            }
        ]
    }
}"#
);

#[neo_event]
pub struct TransferEvent {
    pub from: NeoByteString,
    pub to: NeoByteString,
    pub amount: NeoInteger,
}

fn storage_context() -> Option<NeoStorageContext> {
    NeoStorage::get_context().ok()
}

fn balance_key(account: &NeoByteString) -> Vec<u8> {
    let mut key = BALANCE_PREFIX.to_vec();
    key.extend_from_slice(account.as_slice());
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
    let data = NeoStorage::get(ctx, &key)?;
    Ok(read_i64(&data))
}

fn store_total_supply(ctx: &NeoStorageContext, value: i64) -> NeoResult<()> {
    let key = NeoByteString::from_slice(TOTAL_SUPPLY_KEY);
    let encoded = write_i64(value);
    NeoStorage::put(ctx, &key, &encoded)
}

fn load_balance(ctx: &NeoStorageContext, account: &NeoByteString) -> NeoResult<i64> {
    let key = NeoByteString::from_slice(&balance_key(account));
    let data = NeoStorage::get(ctx, &key)?;
    Ok(read_i64(&data))
}

fn store_balance(ctx: &NeoStorageContext, account: &NeoByteString, value: i64) -> NeoResult<()> {
    let key = NeoByteString::from_slice(&balance_key(account));
    if value == 0 {
        NeoStorage::delete(ctx, &key)
    } else {
        let encoded = write_i64(value);
        NeoStorage::put(ctx, &key, &encoded)
    }
}

#[allow(improper_ctypes_definitions)]
#[no_mangle]
pub extern "C" fn init(owner_ptr: i64, owner_len: i64, amount: i64) -> i64 {
    if amount <= 0 {
        return 0;
    }

    let Some(ctx) = storage_context() else {
        return 0;
    };

    if load_total_supply(&ctx).unwrap_or(0) != 0 {
        return 0;
    }

    let Some(owner) = read_address(owner_ptr, owner_len) else {
        return 0;
    };

    match store_total_supply(&ctx, amount) {
        Ok(_) => {}
        Err(_) => return 0,
    }

    match store_balance(&ctx, &owner, amount) {
        Ok(_) => {}
        Err(_) => return 0,
    }

    TransferEvent {
        from: NeoByteString::new(Vec::new()),
        to: owner,
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

#[allow(improper_ctypes_definitions)]
#[no_mangle]
#[neo_safe]
pub extern "C" fn balanceOf(account_ptr: i64, account_len: i64) -> i64 {
    let Some(account) = read_address(account_ptr, account_len) else {
        return 0;
    };
    storage_context()
        .and_then(|ctx| load_balance(&ctx, &account).ok())
        .unwrap_or(0)
}

#[allow(improper_ctypes_definitions)]
#[no_mangle]
pub extern "C" fn transfer(
    from_ptr: i64,
    from_len: i64,
    to_ptr: i64,
    to_len: i64,
    amount: i64,
    _data_ptr: i64,
    _data_len: i64,
) -> i64 {
    if amount <= 0 {
        return 0;
    }

    let Some(ctx) = storage_context() else {
        return 0;
    };

    let Some(from) = read_address(from_ptr, from_len) else {
        return 0;
    };
    let Some(to) = read_address(to_ptr, to_len) else {
        return 0;
    };

    if from.as_slice() == to.as_slice() {
        return 0;
    }

    if !ensure_witness(&from) {
        return 0;
    }

    let from_balance = match load_balance(&ctx, &from) {
        Ok(value) => value,
        Err(_) => return 0,
    };

    if from_balance < amount {
        return 0;
    }

    let to_balance = match load_balance(&ctx, &to) {
        Ok(value) => value,
        Err(_) => 0,
    };

    let new_from_balance = match from_balance.checked_sub(amount) {
        Some(value) => value,
        None => return 0,
    };

    let new_to_balance = match to_balance.checked_add(amount) {
        Some(value) => value,
        None => return 0,
    };

    if store_balance(&ctx, &from, new_from_balance).is_err() {
        return 0;
    }
    if store_balance(&ctx, &to, new_to_balance).is_err() {
        return 0;
    }

    TransferEvent {
        from,
        to,
        amount: NeoInteger::new(amount),
    }
    .emit()
    .ok();

    1
}

#[allow(improper_ctypes_definitions)]
#[no_mangle]
pub extern "C" fn onNEP17Payment(from: NeoByteString, amount: i64, _data: NeoByteString) {
    if amount <= 0 {
        return;
    }

    let Some(ctx) = storage_context() else {
        return;
    };

    let current_balance = match load_balance(&ctx, &from) {
        Ok(value) => value,
        Err(_) => 0,
    };

    let new_balance = match current_balance.checked_add(amount) {
        Some(value) => value,
        None => return,
    };

    if store_balance(&ctx, &from, new_balance).is_err() {
        return;
    }

    TransferEvent {
        from: NeoByteString::new(Vec::new()),
        to: from,
        amount: NeoInteger::new(amount),
    }
    .emit()
    .ok();
}

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
    let slice = unsafe { slice::from_raw_parts(ptr as *const u8, len) };
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
        NeoStorage::delete(&ctx, &NeoByteString::from_slice(TOTAL_SUPPLY_KEY)).ok();
        if let Ok(iter) = NeoStorage::find(&ctx, &NeoByteString::from_slice(BALANCE_PREFIX)) {
            let mut iterator = iter;
            while iterator.has_next() {
                if let Some(entry) = iterator.next() {
                    if let Some(key) = entry
                        .as_struct()
                        .and_then(|st| st.get_field("key"))
                        .and_then(NeoValue::as_byte_string)
                    {
                        NeoStorage::delete(&ctx, &key).ok();
                    }
                }
            }
        }
    }

    fn configure_sample(amount: i64) {
        reset_state();
        let owner = address(0x44);
        assert_eq!(init(owner.as_ptr() as i64, owner.len() as i64, amount), 1);
    }

    #[test]
    fn init_and_supply() {
        let _guard = test_lock().lock().unwrap();
        configure_sample(1_000_000);
        assert_eq!(totalSupply(), 1_000_000);
    }

    #[test]
    fn balance_reflects_supply() {
        let _guard = test_lock().lock().unwrap();
        let owner = address(0x55);
        configure_sample(500_000);
        let balance = balanceOf(owner.as_ptr() as i64, owner.len() as i64);
        assert_eq!(balance, 500_000);
    }
}
