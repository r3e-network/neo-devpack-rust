use core::slice;
use neo_devpack::{codec, prelude::*};
use serde::{Deserialize, Serialize};

const CONFIG_KEY: &[u8] = b"oracle:config";
const REQUEST_COUNTER_KEY: &[u8] = b"oracle:counter";
const RESPONSE_PREFIX: &[u8] = b"oracle:response:";

#[derive(Clone, Serialize, Deserialize)]
struct OracleConfig {
    owner: NeoByteString,
    oracle: NeoByteString,
}

#[derive(Clone, Serialize, Deserialize)]
struct OracleResponseRecord {
    request_id: i64,
    status_code: i64,
    data: NeoByteString,
}

neo_manifest_overlay!(
    r#"{
    "name": "NeoOracleConsumer",
    "features": { "storage": true }
}"#
);

#[neo_event]
pub struct OracleRequestCreated {
    pub request_id: NeoInteger,
    pub url: NeoString,
}

#[neo_event]
pub struct OracleResponseStored {
    pub request_id: NeoInteger,
    pub status_code: NeoInteger,
}

#[allow(improper_ctypes_definitions)]
#[neo_safe]
#[no_mangle]
pub extern "C" fn getConfig() -> NeoByteString {
    storage_context()
        .and_then(|ctx| load_config(&ctx))
        .map(|config| serialize_value(&config))
        .unwrap_or_else(|| NeoByteString::new(Vec::new()))
}

#[neo_safe]
#[no_mangle]
pub extern "C" fn lastRequestId() -> i64 {
    storage_context()
        .and_then(|ctx| load_from_storage(&ctx, REQUEST_COUNTER_KEY))
        .unwrap_or(0i64)
}

#[allow(improper_ctypes_definitions)]
#[neo_safe]
#[no_mangle]
pub extern "C" fn getResponse(request_id: i64) -> NeoByteString {
    storage_context()
        .and_then(|ctx| load_response(&ctx, request_id))
        .map(|record| serialize_value(&record))
        .unwrap_or_else(|| NeoByteString::new(Vec::new()))
}

#[no_mangle]
pub extern "C" fn configure(
    owner_ptr: i64,
    owner_len: i64,
    oracle_ptr: i64,
    oracle_len: i64,
) -> i64 {
    let Some(ctx) = storage_context() else {
        return 0;
    };
    if load_config(&ctx).is_some() {
        return 0;
    }
    let Some(owner) = read_address(owner_ptr, owner_len) else {
        return 0;
    };
    let Some(oracle) = read_address(oracle_ptr, oracle_len) else {
        return 0;
    };

    let config = OracleConfig { owner, oracle };

    if store_config(&ctx, &config).is_err() {
        return 0;
    }
    if store_to_storage(&ctx, REQUEST_COUNTER_KEY, &0i64).is_err() {
        return 0;
    }

    1
}

#[no_mangle]
pub extern "C" fn request(
    url_ptr: i64,
    url_len: i64,
    filter_ptr: i64,
    filter_len: i64,
    user_data_ptr: i64,
    user_data_len: i64,
) -> i64 {
    let Some(ctx) = storage_context() else {
        return 0;
    };
    let Some(config) = load_config(&ctx) else {
        return 0;
    };
    if !ensure_witness(&config.owner) {
        return 0;
    }

    let Some(url) = read_string(url_ptr, url_len) else {
        return 0;
    };
    let filter = read_string(filter_ptr, filter_len).unwrap_or_default();
    let user_data = read_bytes(user_data_ptr, user_data_len).unwrap_or_default();

    let request_id = match next_request_id(&ctx) {
        Some(id) => id,
        None => return 0,
    };

    let contract_hash = match NeoRuntime::get_executing_script_hash() {
        Ok(hash) => hash,
        Err(_) => return 0,
    };

    let mut args = NeoArray::new();
    args.push(NeoValue::from(NeoString::from_str(&url)));
    args.push(NeoValue::from(NeoString::from_str(&filter)));
    args.push(NeoValue::from(contract_hash.clone()));
    args.push(NeoValue::from(NeoString::from_str("onOracleResponse")));
    args.push(NeoValue::from(NeoByteString::from_slice(&user_data)));

    if NeoContractRuntime::call(&config.oracle, &NeoString::from_str("request"), &args).is_err() {
        return 0;
    }

    OracleRequestCreated {
        request_id: NeoInteger::new(request_id),
        url: NeoString::from_str(&url),
    }
    .emit()
    .ok();

    request_id
}

#[no_mangle]
pub extern "C" fn onOracleResponse(
    request_id: i64,
    status_code: i64,
    data_ptr: i64,
    data_len: i64,
) -> i64 {
    let Some(ctx) = storage_context() else {
        return 0;
    };
    let Some(config) = load_config(&ctx) else {
        return 0;
    };
    let Ok(call_hash) = NeoRuntime::get_calling_script_hash() else {
        return 0;
    };
    if !addresses_equal(&call_hash, &config.oracle) {
        return 0;
    }

    let Some(data) = read_bytes(data_ptr, data_len) else {
        return 0;
    };

    let record = OracleResponseRecord {
        request_id,
        status_code,
        data: NeoByteString::from_slice(&data),
    };

    if store_response(&ctx, request_id, &record).is_err() {
        return 0;
    }

    OracleResponseStored {
        request_id: NeoInteger::new(request_id),
        status_code: NeoInteger::new(status_code),
    }
    .emit()
    .ok();

    1
}

fn storage_context() -> Option<NeoStorageContext> {
    NeoStorage::get_context().ok()
}

fn load_config(ctx: &NeoStorageContext) -> Option<OracleConfig> {
    load_from_storage(ctx, CONFIG_KEY)
}

fn store_config(ctx: &NeoStorageContext, config: &OracleConfig) -> NeoResult<()> {
    store_to_storage(ctx, CONFIG_KEY, config)
}

fn next_request_id(ctx: &NeoStorageContext) -> Option<i64> {
    let current: i64 = load_from_storage(ctx, REQUEST_COUNTER_KEY).unwrap_or(0);
    let next = current.checked_add(1)?;
    store_to_storage(ctx, REQUEST_COUNTER_KEY, &next).ok()?;
    Some(next)
}

fn response_key(request_id: i64) -> Vec<u8> {
    let mut key = RESPONSE_PREFIX.to_vec();
    key.extend_from_slice(&request_id.to_le_bytes());
    key
}

fn load_response(ctx: &NeoStorageContext, request_id: i64) -> Option<OracleResponseRecord> {
    load_from_storage(ctx, &response_key(request_id))
}

fn store_response(
    ctx: &NeoStorageContext,
    request_id: i64,
    record: &OracleResponseRecord,
) -> NeoResult<()> {
    store_to_storage(ctx, &response_key(request_id), record)
}

fn read_address(ptr: i64, len: i64) -> Option<NeoByteString> {
    let bytes = read_bytes(ptr, len)?;
    if bytes.len() != 20 {
        return None;
    }
    Some(NeoByteString::from_slice(&bytes))
}

fn read_string(ptr: i64, len: i64) -> Option<String> {
    let bytes = read_bytes(ptr, len)?;
    String::from_utf8(bytes).ok()
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
    if ptr == 0 || len < 0 {
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
        NeoStorage::delete(&ctx, &NeoByteString::from_slice(CONFIG_KEY)).ok();
        NeoStorage::delete(&ctx, &NeoByteString::from_slice(REQUEST_COUNTER_KEY)).ok();
        if let Ok(iter) = NeoStorage::find(&ctx, &NeoByteString::from_slice(RESPONSE_PREFIX)) {
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

    fn configure_sample() -> OracleConfig {
        reset_state();
        let owner = address(0x66);
        let oracle = address(0x00);
        assert_eq!(
            configure(
                owner.as_ptr() as i64,
                owner.len() as i64,
                oracle.as_ptr() as i64,
                oracle.len() as i64,
            ),
            1
        );
        let config_bytes = getConfig();
        codec::deserialize(config_bytes.as_slice()).expect("config decode")
    }

    #[test]
    fn configure_and_request_assigns_id() {
        let _guard = test_lock().lock().unwrap();
        let _config = configure_sample();
        let url = b"https://api.example.com".to_vec();
        let filter = b"$.price".to_vec();
        let user = b"asset:neo".to_vec();
        let request_id = request(
            url.as_ptr() as i64,
            url.len() as i64,
            filter.as_ptr() as i64,
            filter.len() as i64,
            user.as_ptr() as i64,
            user.len() as i64,
        );
        assert_eq!(request_id, 1);
        assert_eq!(lastRequestId(), 1);
    }

    #[test]
    fn store_response_and_query() {
        let _guard = test_lock().lock().unwrap();
        configure_sample();
        let data = b"{\"value\":42}".to_vec();
        assert_eq!(
            onOracleResponse(1, 200, data.as_ptr() as i64, data.len() as i64),
            1
        );

        let response_bytes = getResponse(1);
        let record: OracleResponseRecord =
            codec::deserialize(response_bytes.as_slice()).expect("response decode");
        assert_eq!(record.status_code, 200);
        assert_eq!(record.data.as_slice(), data.as_slice());
    }
}
