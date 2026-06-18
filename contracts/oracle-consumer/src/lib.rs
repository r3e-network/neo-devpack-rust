// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use neo_devpack::prelude::*;

neo_manifest_overlay!(
    r#"{
    "name": "NeoOracleConsumer"
}"#
);

const KEY_CONFIG_OWNER: i64 = -1;
const KEY_CONFIG_ORACLE: i64 = -2;
const KEY_REQUEST_COUNTER: i64 = -3;
const KEY_STRIDE: i64 = 16;
const FIELD_RESPONSE_STATUS: i64 = 1;
const FIELD_RESPONSE_DATA: i64 = 2;
const FIELD_RESPONSE_EXISTS: i64 = 3;

fn response_key(id: i64, field: i64) -> i64 {
    id * KEY_STRIDE + field
}

fn response_status_key(id: i64) -> i64 {
    response_key(id, FIELD_RESPONSE_STATUS)
}

fn response_data_key(id: i64) -> i64 {
    response_key(id, FIELD_RESPONSE_DATA)
}

fn response_exists_key(id: i64) -> i64 {
    response_key(id, FIELD_RESPONSE_EXISTS)
}

fn storage_put_i64(key: i64, value: i64) -> bool {
    RawStorage::put_i64_key(key, value);
    true
}

fn storage_get_i64(key: i64) -> i64 {
    RawStorage::get_i64_key_or_zero(key)
}

fn ensure_witness_i64(account: i64) -> bool {
    NeoRuntime::check_witness_i64(account)
        .map(|flag| flag.as_bool())
        .unwrap_or(false)
}

#[cfg(test)]
fn script_hash_to_i64(hash: &NeoByteString) -> i64 {
    let bytes = hash.as_slice();
    if bytes.len() < 8 {
        return 0;
    }
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&bytes[..8]);
    i64::from_le_bytes(buf)
}

fn calling_contract_id() -> i64 {
    NeoRuntime::get_calling_script_hash_i64().unwrap_or(0)
}

// Events
#[neo_event]
pub struct OracleConfigured {
    pub owner: i64,
    pub oracle: i64,
}

#[neo_event]
pub struct OracleRequestSent {
    pub request_id: i64,
}

#[neo_event]
pub struct OracleResponseReceived {
    pub request_id: i64,
    pub status_code: i64,
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
        if !ensure_witness_i64(owner_id) {
            return false;
        }
        if storage_get_i64(KEY_CONFIG_OWNER) != 0 {
            return false;
        }
        storage_put_i64(KEY_CONFIG_OWNER, owner_id);
        storage_put_i64(KEY_CONFIG_ORACLE, oracle_id);
        let _ = (OracleConfigured {
            owner: owner_id,
            oracle: oracle_id,
        })
        .emit();
        true
    }

    #[neo_method]
    pub fn request(url_id: i64, filter_id: i64, user_data_id: i64) -> i64 {
        if url_id == 0 || filter_id == 0 || user_data_id == 0 {
            return 0;
        }
        let owner_id = storage_get_i64(KEY_CONFIG_OWNER);
        if owner_id == 0 {
            return 0;
        }
        if !ensure_witness_i64(owner_id) {
            return 0;
        }
        let current = storage_get_i64(KEY_REQUEST_COUNTER);
        let next = match current.checked_add(1) {
            Some(id) if id > 0 && id <= i64::MAX / KEY_STRIDE => id,
            _ => return 0,
        };
        storage_put_i64(KEY_REQUEST_COUNTER, next);
        let _ = (OracleRequestSent { request_id: next }).emit();
        next
    }

    #[neo_method(name = "onOracleResponse")]
    pub fn on_oracle_response(request_id: i64, status_code: i64, data_id: i64) -> bool {
        if request_id <= 0 {
            return false;
        }
        let oracle_id = storage_get_i64(KEY_CONFIG_ORACLE);
        if oracle_id == 0 {
            return false;
        }
        if calling_contract_id() != oracle_id {
            return false;
        }
        if request_id > i64::MAX / KEY_STRIDE {
            return false;
        }
        storage_put_i64(response_status_key(request_id), status_code);
        storage_put_i64(response_data_key(request_id), data_id);
        storage_put_i64(response_exists_key(request_id), 1);
        let _ = (OracleResponseReceived {
            request_id,
            status_code,
        })
        .emit();
        true
    }

    #[neo_method(safe, name = "lastRequestId")]
    pub fn last_request_id() -> i64 {
        storage_get_i64(KEY_REQUEST_COUNTER)
    }

    /// Return config via notify: [owner_id, oracle_id]
    #[neo_method(safe, name = "getConfig")]
    pub fn get_config() {
        let owner = storage_get_i64(KEY_CONFIG_OWNER);
        if owner == 0 {
            return;
        }
        let oracle = storage_get_i64(KEY_CONFIG_ORACLE);
        if oracle == 0 {
            return;
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = (owner, oracle);
            let _ = NeoRuntime::notify_event("getConfig");
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let label = NeoString::from_str("getConfig");
            let mut state = NeoArray::new();
            state.push(NeoValue::from(owner));
            state.push(NeoValue::from(oracle));
            let _ = NeoRuntime::notify(&label, &state);
        }
    }

    /// Return response via notify: [status, data_id]
    #[neo_method(safe, name = "getResponse")]
    pub fn get_response(request_id: i64) {
        if request_id <= 0 || request_id > i64::MAX / KEY_STRIDE {
            return;
        }
        if storage_get_i64(response_exists_key(request_id)) == 0 {
            return;
        }
        let status = storage_get_i64(response_status_key(request_id));
        let data_id = storage_get_i64(response_data_key(request_id));

        #[cfg(target_arch = "wasm32")]
        {
            let _ = (status, data_id);
            let _ = NeoRuntime::notify_event("getResponse");
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let label = NeoString::from_str("getResponse");
            let mut state = NeoArray::new();
            state.push(NeoValue::from(status));
            state.push(NeoValue::from(data_id));
            let _ = NeoRuntime::notify(&label, &state);
        }
    }
}

impl Default for NeoOracleConsumerContract {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::{calling_contract_id, script_hash_to_i64};
    use neo_devpack::{prelude::NeoByteString, NeoVMSyscall};

    #[test]
    fn contract_compiles() {
        // Compilation test - verifies contract module parses correctly
    }

    #[test]
    fn script_hash_id_conversion_uses_first_eight_bytes() {
        NeoVMSyscall::reset_host_state().unwrap();
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&7_i64.to_le_bytes());
        bytes.extend_from_slice(&[9_u8; 12]);
        assert_eq!(script_hash_to_i64(&NeoByteString::from_slice(&bytes)), 7);
        NeoVMSyscall::set_active_calling_script_hash(&NeoByteString::from_slice(&bytes)).unwrap();
        assert_eq!(calling_contract_id(), 7);
        NeoVMSyscall::reset_host_state().unwrap();
        assert_eq!(calling_contract_id(), 0);
    }
}
