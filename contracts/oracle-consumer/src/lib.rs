// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "NeoOracleConsumer"
}"#
);

// Storage keys
const CONFIG_OWNER_KEY: &[u8] = b"oracle:owner";
const CONFIG_ORACLE_KEY: &[u8] = b"oracle:addr";
const REQUEST_COUNTER_KEY: &[u8] = b"oracle:counter";
const RESPONSE_PREFIX: &[u8] = b"oracle:resp:";
const RESPONSE_STATUS_SUFFIX: &[u8] = b":status";
const RESPONSE_DATA_SUFFIX: &[u8] = b":data";

fn response_key(id: i64, suffix: &[u8]) -> Vec<u8> {
    let mut key = RESPONSE_PREFIX.to_vec();
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

fn read_bytes(ptr: i64, len: i64) -> Option<Vec<u8>> {
    if ptr == 0 || len <= 0 {
        return None;
    }
    let slice = unsafe { core::slice::from_raw_parts(ptr as *const u8, len as usize) };
    Some(slice.to_vec())
}

// Events
#[neo_event]
pub struct OracleConfigured {
    pub owner: NeoByteString,
    pub oracle: NeoByteString,
}

#[neo_event]
pub struct OracleRequestSent {
    pub request_id: NeoInteger,
}

#[neo_event]
pub struct OracleResponseReceived {
    pub request_id: NeoInteger,
    pub status_code: NeoInteger,
}

#[neo_contract]
pub struct NeoOracleConsumerContract;

#[neo_contract]
impl NeoOracleConsumerContract {
    pub fn new() -> Self {
        Self
    }

    #[neo_method]
    pub fn configure(owner_ptr: i64, owner_len: i64, oracle_ptr: i64, oracle_len: i64) -> bool {
        let owner = match read_address(owner_ptr, owner_len) {
            Some(a) => a,
            None => return false,
        };
        if !ensure_witness(&owner) {
            return false;
        }
        let oracle = match read_address(oracle_ptr, oracle_len) {
            Some(a) => a,
            None => return false,
        };
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return false,
        };
        if storage_get_bytes(&ctx, CONFIG_OWNER_KEY).is_some() {
            return false;
        }
        storage_put_bytes(&ctx, CONFIG_OWNER_KEY, owner.as_slice());
        storage_put_bytes(&ctx, CONFIG_ORACLE_KEY, oracle.as_slice());
        let _ = (OracleConfigured {
            owner,
            oracle,
        })
        .emit();
        true
    }

    #[neo_method]
    pub fn request(
        url_ptr: i64,
        url_len: i64,
        filter_ptr: i64,
        filter_len: i64,
        user_data_ptr: i64,
        user_data_len: i64,
    ) -> i64 {
        let _url = match read_bytes(url_ptr, url_len) {
            Some(b) => b,
            None => return 0,
        };
        let _filter = match read_bytes(filter_ptr, filter_len) {
            Some(b) => b,
            None => return 0,
        };
        let _user_data = match read_bytes(user_data_ptr, user_data_len) {
            Some(b) => b,
            None => return 0,
        };
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return 0,
        };
        let current = storage_get_i64(&ctx, REQUEST_COUNTER_KEY).unwrap_or(0);
        let next = current + 1;
        storage_put_i64(&ctx, REQUEST_COUNTER_KEY, next);
        let _ = (OracleRequestSent {
            request_id: NeoInteger::new(next),
        })
        .emit();
        next
    }

    #[neo_method(name = "onOracleResponse")]
    pub fn on_oracle_response(
        request_id: i64,
        status_code: i64,
        data_ptr: i64,
        data_len: i64,
    ) -> bool {
        if request_id <= 0 {
            return false;
        }
        let data = match read_bytes(data_ptr, data_len) {
            Some(b) => b,
            None => return false,
        };
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return false,
        };
        storage_put_i64(&ctx, &response_key(request_id, RESPONSE_STATUS_SUFFIX), status_code);
        storage_put_bytes(&ctx, &response_key(request_id, RESPONSE_DATA_SUFFIX), &data);
        let _ = (OracleResponseReceived {
            request_id: NeoInteger::new(request_id),
            status_code: NeoInteger::new(status_code),
        })
        .emit();
        true
    }

    #[neo_method(name = "lastRequestId")]
    pub fn last_request_id() -> i64 {
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return 0,
        };
        storage_get_i64(&ctx, REQUEST_COUNTER_KEY).unwrap_or(0)
    }

    /// Return config via notify: [owner_hex, oracle_hex]
    #[neo_method(name = "getConfig")]
    pub fn get_config() {
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return,
        };
        let owner = match storage_get_bytes(&ctx, CONFIG_OWNER_KEY) {
            Some(b) => b,
            None => return,
        };
        let oracle = match storage_get_bytes(&ctx, CONFIG_ORACLE_KEY) {
            Some(b) => b,
            None => return,
        };
        let label = NeoString::from_str("getConfig");
        let mut state = NeoArray::new();
        state.push(NeoValue::from(NeoByteString::from_slice(&owner)));
        state.push(NeoValue::from(NeoByteString::from_slice(&oracle)));
        let _ = NeoRuntime::notify(&label, &state);
    }

    /// Return response via notify: [status, data]
    #[neo_method(name = "getResponse")]
    pub fn get_response(request_id: i64) {
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return,
        };
        let status = match storage_get_i64(&ctx, &response_key(request_id, RESPONSE_STATUS_SUFFIX)) {
            Some(s) => s,
            None => return,
        };
        let data = storage_get_bytes(&ctx, &response_key(request_id, RESPONSE_DATA_SUFFIX)).unwrap_or_default();
        let label = NeoString::from_str("getResponse");
        let mut state = NeoArray::new();
        state.push(NeoValue::from(NeoInteger::new(status)));
        state.push(NeoValue::from(NeoByteString::from_slice(&data)));
        let _ = NeoRuntime::notify(&label, &state);
    }
}

impl Default for NeoOracleConsumerContract {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    // Integration tests require NeoVM runtime stubs.
}
