use core::slice;
use neo_devpack::prelude::*;

const TOTAL_SUPPLY_KEY: &[u8] = b"nft:total_supply";
const OWNER_PREFIX: &[u8] = b"nft:owner:";
const BALANCE_PREFIX: &[u8] = b"nft:balance:";

neo_manifest_overlay!(
    r#"{
    "name": "SampleNEP11",
    "supportedstandards": ["NEP-11"],
    "features": { "storage": true },
    "abi": {
        "methods": [
            {
                "name": "initialize",
                "parameters": [
                    {"name": "owner", "type": "Hash160"}
                ],
                "returntype": "Boolean"
            },
            {
                "name": "mint",
                "parameters": [
                    {"name": "owner", "type": "Hash160"},
                    {"name": "token_id", "type": "ByteString"}
                ],
                "returntype": "Boolean"
            },
            {
                "name": "transfer",
                "parameters": [
                    {"name": "from", "type": "Hash160"},
                    {"name": "to", "type": "Hash160"},
                    {"name": "token_id", "type": "ByteString"},
                    {"name": "data", "type": "Any"}
                ],
                "returntype": "Boolean"
            },
            {
                "name": "balanceOf",
                "parameters": [
                    {"name": "owner", "type": "Hash160"}
                ],
                "returntype": "Integer"
            },
            {
                "name": "ownerOf",
                "parameters": [
                    {"name": "token_id", "type": "ByteString"}
                ],
                "returntype": "Hash160"
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
                    {"name": "token_id", "type": "ByteString"}
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
    pub token_id: NeoByteString,
}

fn storage_context() -> Option<NeoStorageContext> {
    NeoStorage::get_context().ok()
}

fn owner_key(token_id: &NeoByteString) -> Vec<u8> {
    let mut key = OWNER_PREFIX.to_vec();
    key.extend_from_slice(token_id.as_slice());
    key
}

fn balance_key(account: &NeoByteString) -> Vec<u8> {
    let mut key = BALANCE_PREFIX.to_vec();
    key.extend_from_slice(account.as_slice());
    key
}

fn load_owner(
    ctx: &NeoStorageContext,
    token_id: &NeoByteString,
) -> NeoResult<Option<NeoByteString>> {
    let key = NeoByteString::from_slice(&owner_key(token_id));
    let value = NeoStorage::get(ctx, &key)?;
    if value.is_empty() {
        return Ok(None);
    }
    Ok(Some(value))
}

fn store_owner(
    ctx: &NeoStorageContext,
    token_id: &NeoByteString,
    owner: Option<&NeoByteString>,
) -> NeoResult<()> {
    let key = NeoByteString::from_slice(&owner_key(token_id));
    match owner {
        Some(addr) => NeoStorage::put(ctx, &key, addr),
        None => NeoStorage::delete(ctx, &key),
    }
}

fn load_balance(ctx: &NeoStorageContext, account: &NeoByteString) -> NeoResult<i64> {
    let key = NeoByteString::from_slice(&balance_key(account));
    let value = NeoStorage::get(ctx, &key)?;
    if value.is_empty() {
        Ok(0)
    } else {
        read_i64(&value)
    }
}

fn store_balance(ctx: &NeoStorageContext, account: &NeoByteString, balance: i64) -> NeoResult<()> {
    let key = NeoByteString::from_slice(&balance_key(account));
    if balance == 0 {
        NeoStorage::delete(ctx, &key)
    } else {
        let value = NeoByteString::from_slice(&balance.to_le_bytes());
        NeoStorage::put(ctx, &key, &value)
    }
}

fn load_total_supply(ctx: &NeoStorageContext) -> NeoResult<i64> {
    let key = NeoByteString::from_slice(TOTAL_SUPPLY_KEY);
    let value = NeoStorage::get(ctx, &key)?;
    if value.is_empty() {
        Ok(0)
    } else {
        read_i64(&value)
    }
}

fn store_total_supply(ctx: &NeoStorageContext, supply: i64) -> NeoResult<()> {
    let key = NeoByteString::from_slice(TOTAL_SUPPLY_KEY);
    let value = NeoByteString::from_slice(&supply.to_le_bytes());
    NeoStorage::put(ctx, &key, &value)
}

fn read_i64(bytes: &NeoByteString) -> NeoResult<i64> {
    let data = bytes.as_slice();
    if data.is_empty() {
        return Ok(0);
    }
    if data.len() != 8 {
        return Err(NeoError::new("Invalid i64 data length"));
    }
    let mut buf = [0u8; 8];
    buf.copy_from_slice(data);
    Ok(i64::from_le_bytes(buf))
}

fn addresses_equal(left: &NeoByteString, right: &NeoByteString) -> bool {
    left.as_slice() == right.as_slice()
}

fn ensure_witness(account: &NeoByteString) -> bool {
    NeoRuntime::check_witness(account)
        .ok()
        .map(|b| b.as_bool())
        .unwrap_or(false)
}

#[neo_safe]
#[no_mangle]
pub extern "C" fn totalSupply() -> i64 {
    storage_context()
        .and_then(|ctx| load_total_supply(&ctx).ok())
        .unwrap_or(0)
}

#[allow(improper_ctypes_definitions)]
#[neo_safe]
#[no_mangle]
pub extern "C" fn balanceOf(owner_ptr: i64, owner_len: i64) -> i64 {
    let Some(owner) = read_address(owner_ptr, owner_len) else {
        return 0;
    };
    storage_context()
        .and_then(|ctx| load_balance(&ctx, &owner).ok())
        .unwrap_or(0)
}

#[allow(improper_ctypes_definitions)]
#[neo_safe]
#[no_mangle]
pub extern "C" fn ownerOf(token_id_ptr: i64, token_id_len: i64) -> NeoByteString {
    let Some(token_id) = read_bytes(token_id_ptr, token_id_len) else {
        return NeoByteString::new(Vec::new());
    };
    let token_id = NeoByteString::from_slice(&token_id);
    storage_context()
        .and_then(|ctx| load_owner(&ctx, &token_id).ok())
        .flatten()
        .unwrap_or_else(|| NeoByteString::new(Vec::new()))
}

#[allow(improper_ctypes_definitions)]
#[no_mangle]
pub extern "C" fn initialize(owner_ptr: i64, owner_len: i64) -> i64 {
    let Some(ctx) = storage_context() else {
        return 0;
    };

    let Some(owner) = read_address(owner_ptr, owner_len) else {
        return 0;
    };

    if !ensure_witness(&owner) {
        return 0;
    }

    let key = NeoByteString::from_slice(TOTAL_SUPPLY_KEY);
    if NeoStorage::get(&ctx, &key)
        .ok()
        .map(|v| !v.is_empty())
        .unwrap_or(false)
    {
        return 0;
    }

    if store_total_supply(&ctx, 0).is_err() {
        return 0;
    }

    1
}

#[allow(improper_ctypes_definitions)]
#[no_mangle]
pub extern "C" fn mint(
    owner_ptr: i64,
    owner_len: i64,
    token_id_ptr: i64,
    token_id_len: i64,
) -> i64 {
    let Some(ctx) = storage_context() else {
        return 0;
    };

    let Some(owner) = read_address(owner_ptr, owner_len) else {
        return 0;
    };

    let Some(token_id) = read_bytes(token_id_ptr, token_id_len) else {
        return 0;
    };
    let token_id = NeoByteString::from_slice(&token_id);

    if !ensure_witness(&owner) {
        return 0;
    }

    if load_owner(&ctx, &token_id).ok().flatten().is_some() {
        return 0;
    }

    let current_balance = match load_balance(&ctx, &owner) {
        Ok(value) => value,
        Err(_) => 0,
    };
    let balance = match current_balance.checked_add(1) {
        Some(value) => value,
        None => return 0,
    };

    if store_owner(&ctx, &token_id, Some(&owner)).is_err() {
        return 0;
    }
    if store_balance(&ctx, &owner, balance).is_err() {
        return 0;
    }

    let current_supply = match load_total_supply(&ctx) {
        Ok(value) => value,
        Err(_) => 0,
    };
    let supply = match current_supply.checked_add(1) {
        Some(value) => value,
        None => return 0,
    };

    if store_total_supply(&ctx, supply).is_err() {
        return 0;
    }

    TransferEvent {
        from: NeoByteString::new(Vec::new()),
        to: owner.clone(),
        token_id: token_id.clone(),
    }
    .emit()
    .ok();

    1
}

#[allow(improper_ctypes_definitions)]
#[no_mangle]
pub extern "C" fn transfer(
    from_ptr: i64,
    from_len: i64,
    to_ptr: i64,
    to_len: i64,
    token_id_ptr: i64,
    token_id_len: i64,
    _data_ptr: i64,
    _data_len: i64,
) -> i64 {
    let Some(ctx) = storage_context() else {
        return 0;
    };

    let Some(from) = read_address(from_ptr, from_len) else {
        return 0;
    };
    let Some(to) = read_address(to_ptr, to_len) else {
        return 0;
    };

    if addresses_equal(&from, &to) {
        return 0;
    }

    let Some(token_id) = read_bytes(token_id_ptr, token_id_len) else {
        return 0;
    };
    let token_id = NeoByteString::from_slice(&token_id);

    if !ensure_witness(&from) {
        return 0;
    }

    let current_owner = match load_owner(&ctx, &token_id) {
        Ok(Some(owner)) => owner,
        Ok(None) => return 0,
        Err(_) => return 0,
    };

    if !addresses_equal(&current_owner, &from) {
        return 0;
    }

    let from_balance = match load_balance(&ctx, &from) {
        Ok(value) => value,
        Err(_) => 0,
    };
    let to_balance = match load_balance(&ctx, &to) {
        Ok(value) => value,
        Err(_) => 0,
    };

    if from_balance <= 0 {
        return 0;
    }

    let new_from_balance = match from_balance.checked_sub(1) {
        Some(value) => value,
        None => return 0,
    };
    let new_to_balance = match to_balance.checked_add(1) {
        Some(value) => value,
        None => return 0,
    };

    if store_owner(&ctx, &token_id, Some(&to)).is_err() {
        return 0;
    }
    if store_balance(&ctx, &from, new_from_balance).is_err() {
        return 0;
    }
    if store_balance(&ctx, &to, new_to_balance).is_err() {
        return 0;
    }

    TransferEvent { from, to, token_id }.emit().ok();

    1
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
        if let Ok(iter) = NeoStorage::find(&ctx, &NeoByteString::from_slice(OWNER_PREFIX)) {
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

    #[test]
    fn total_supply_starts_at_zero() {
        let _guard = test_lock().lock().unwrap();
        reset_state();
        assert_eq!(totalSupply(), 0);
    }

    #[test]
    fn balance_of_empty_account_is_zero() {
        let _guard = test_lock().lock().unwrap();
        reset_state();
        let addr = address(0x44);
        assert_eq!(balanceOf(addr.as_ptr() as i64, addr.len() as i64), 0);
    }

    #[test]
    fn initialize_requires_witness() {
        let _guard = test_lock().lock().unwrap();
        reset_state();
        let owner = address(0x55);
        assert_eq!(initialize(owner.as_ptr() as i64, owner.len() as i64), 0);
    }
}
