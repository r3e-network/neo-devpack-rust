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

fn response_status_key(id: i64) -> Vec<u8> {
    let mut key = RESPONSE_PREFIX.to_vec();
    key.extend_from_slice(&id.to_le_bytes());
    key.extend_from_slice(RESPONSE_STATUS_SUFFIX);
    key
}

fn response_data_key(id: i64) -> Vec<u8> {
    let mut key = RESPONSE_PREFIX.to_vec();
    key.extend_from_slice(&id.to_le_bytes());
    key.extend_from_slice(RESPONSE_DATA_SUFFIX);
    key
}

fn storage_put_i64(ctx: &NeoStorageContext, key: &[u8], value: i64) -> bool {
    NeoStorage::put(
        ctx,
        &NeoByteString::from_slice(key),
        &NeoByteString::from_slice(&value.to_le_bytes()),
    )
    .is_ok()
}

fn storage_get_i64(ctx: &NeoStorageContext, key: &[u8]) -> Option<i64> {
    let data = NeoStorage::get(ctx, &NeoByteString::from_slice(key)).ok()?;
    if data.len() != 8 {
        return None;
    }
    let s = data.as_slice();
    let mut buf = [0u8; 8];
    buf.copy_from_slice(s);
    Some(i64::from_le_bytes(buf))
}

fn storage_has_key(ctx: &NeoStorageContext, key: &[u8]) -> bool {
    NeoStorage::get(ctx, &NeoByteString::from_slice(key))
        .map(|d| !d.is_empty())
        .unwrap_or(false)
}

// Events
#[neo_event]
pub struct OracleConfigured {
    pub owner: NeoInteger,
    pub oracle: NeoInteger,
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
    pub fn configure(owner_id: i64, oracle_id: i64) -> bool {
        if owner_id == 0 || oracle_id == 0 {
            return false;
        }
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return false,
        };
        if storage_has_key(&ctx, CONFIG_OWNER_KEY) {
            return false;
        }
        storage_put_i64(&ctx, CONFIG_OWNER_KEY, owner_id);
        storage_put_i64(&ctx, CONFIG_ORACLE_KEY, oracle_id);
        let _ = (OracleConfigured {
            owner: NeoInteger::new(owner_id),
            oracle: NeoInteger::new(oracle_id),
        })
        .emit();
        true
    }

    #[neo_method]
    pub fn request(url_id: i64, filter_id: i64, user_data_id: i64) -> i64 {
        if url_id == 0 || filter_id == 0 || user_data_id == 0 {
            return 0;
        }
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
    pub fn on_oracle_response(request_id: i64, status_code: i64, data_id: i64) -> bool {
        if request_id <= 0 {
            return false;
        }
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return false,
        };
        storage_put_i64(&ctx, &response_status_key(request_id), status_code);
        storage_put_i64(&ctx, &response_data_key(request_id), data_id);
        let _ = (OracleResponseReceived {
            request_id: NeoInteger::new(request_id),
            status_code: NeoInteger::new(status_code),
        })
        .emit();
        true
    }

    #[neo_method(safe, name = "lastRequestId")]
    pub fn last_request_id() -> i64 {
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return 0,
        };
        storage_get_i64(&ctx, REQUEST_COUNTER_KEY).unwrap_or(0)
    }

    /// Return config via notify: [owner_id, oracle_id]
    #[neo_method(safe, name = "getConfig")]
    pub fn get_config() {
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return,
        };
        let owner = match storage_get_i64(&ctx, CONFIG_OWNER_KEY) {
            Some(v) => v,
            None => return,
        };
        let oracle = match storage_get_i64(&ctx, CONFIG_ORACLE_KEY) {
            Some(v) => v,
            None => return,
        };
        let label = NeoString::from_str("getConfig");
        let mut state = NeoArray::new();
        state.push(NeoValue::from(NeoInteger::new(owner)));
        state.push(NeoValue::from(NeoInteger::new(oracle)));
        let _ = NeoRuntime::notify(&label, &state);
    }

    /// Return response via notify: [status, data_id]
    #[neo_method(safe, name = "getResponse")]
    pub fn get_response(request_id: i64) {
        let ctx = match NeoStorage::get_context().ok() {
            Some(c) => c,
            None => return,
        };
        let status = match storage_get_i64(&ctx, &response_status_key(request_id)) {
            Some(s) => s,
            None => return,
        };
        let data_id = storage_get_i64(&ctx, &response_data_key(request_id)).unwrap_or(0);
        let label = NeoString::from_str("getResponse");
        let mut state = NeoArray::new();
        state.push(NeoValue::from(NeoInteger::new(status)));
        state.push(NeoValue::from(NeoInteger::new(data_id)));
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
    #[test]
    fn contract_compiles() {
        // Compilation test - verifies contract module parses correctly
    }
}
