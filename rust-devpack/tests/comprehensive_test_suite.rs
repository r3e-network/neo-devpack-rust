// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

//! Integration-style checks for the devpack facade.

use neo_devpack::prelude::*;
use neo_syscalls::*;

struct FailingSerialize;

impl serde::Serialize for FailingSerialize {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Err(serde::ser::Error::custom("boom"))
    }
}

#[test]
fn syscall_registry_matches_canonical_list() {
    let registry = NeoVMSyscallRegistry::get_instance();
    assert_eq!(registry.iter().count(), 37);
    assert!(registry.has_syscall("System.Runtime.GetTime"));
    assert!(registry.has_syscall("System.Storage.Get"));
    assert!(registry.has_syscall("System.Contract.Call"));
}

#[test]
fn runtime_surface_is_coherent() {
    let ctx = NeoRuntimeContext::new();
    assert!(ctx.trigger().unwrap().as_i32_saturating() >= 0);
    assert!(ctx.gas_left().unwrap().as_i32_saturating() >= 0);

    let time = NeoRuntime::get_time().unwrap();
    assert!(time.as_i32_saturating() >= 0);

    let random = NeoRuntime::get_random().unwrap();
    assert!(random.as_i32_saturating() >= 0);

    let platform = NeoRuntime::platform().unwrap();
    assert!(!platform.as_str().is_empty());
}

#[test]
fn storage_facade_obeys_read_only_contexts() {
    let writable = NeoStorage::get_context().unwrap();
    let read_only = NeoStorage::get_read_only_context().unwrap();
    assert!(read_only.is_read_only());

    let key = NeoByteString::from_slice(b"key");
    let value = NeoByteString::from_slice(b"value");

    assert!(NeoStorage::put(&writable, &key, &value).is_ok());
    assert!(NeoStorage::delete(&writable, &key).is_ok());

    let err = NeoStorage::put(&read_only, &key, &value).unwrap_err();
    assert_eq!(err, NeoError::InvalidOperation);
}

#[test]
fn contract_crypto_and_json_helpers_behave_consistently() {
    let script = NeoByteString::from_slice(b"contract");
    let manifest = NeoContractManifest {
        name: "Demo".to_string(),
        version: "1.0.0".to_string(),
        author: "Neo".to_string(),
        email: "dev@neo.org".to_string(),
        description: "Placeholder".to_string(),
        abi: NeoContractABI {
            hash: "0x00".to_string(),
            methods: Vec::new(),
            events: Vec::new(),
        },
        permissions: Vec::new(),
        trusts: Vec::new(),
        supported_standards: Vec::new(),
    };

    let contract_hash = NeoContractRuntime::create(&script, &manifest).unwrap();
    assert!(!contract_hash.is_empty());

    let data = NeoByteString::from_slice(b"neo");
    assert_eq!(NeoCrypto::sha256(&data).unwrap().len(), 32);

    let json = NeoJSON::serialize(&NeoValue::from(NeoInteger::new(7))).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(json.as_str()).unwrap();
    assert_eq!(parsed["type"].as_str(), Some("Integer"));
    let value = NeoJSON::deserialize(&json).unwrap();
    assert!(value.as_integer().is_some());
}

#[test]
fn codec_rejects_oversized_payloads() {
    let large = vec![0xAB; 1_048_577];
    let encoded = neo_devpack::codec::serialize(&large);
    assert!(encoded.is_err());
}

#[test]
fn json_helpers_propagate_serialization_errors() {
    let utils_err = neo_devpack::utils::json_to_bytes(&FailingSerialize).unwrap_err();
    assert!(utils_err
        .message()
        .contains("failed to serialize JSON bytes"));

    let storage_err = neo_devpack::storage::write_json(&FailingSerialize).unwrap_err();
    assert!(storage_err
        .message()
        .contains("failed to serialize storage JSON"));
}

#[test]
fn storage_store_propagates_serialization_errors() {
    let context = NeoStorage::get_context().expect("writable context");
    let err = neo_devpack::storage::store(&context, b"failing", &FailingSerialize).unwrap_err();
    assert!(err.message().contains("failed to serialize storage JSON"));
}

#[test]
fn json_deserialize_rejects_out_of_range_bytestring_values() {
    let json = NeoString::from_str(r#"{"type":"ByteString","value":[256]}"#);
    let parsed = NeoJSON::deserialize(&json);
    assert!(parsed.is_err());
}

#[test]
fn neo_manifest_serde_accepts_translator_supportedstandards_alias() {
    let manifest_json = serde_json::json!({
        "name": "Demo",
        "version": "1.0.0",
        "author": "Neo",
        "email": "dev@neo.org",
        "description": "Compatibility check",
        "abi": {
            "hash": "0x00",
            "methods": [],
            "events": []
        },
        "permissions": [],
        "trusts": [],
        "supportedstandards": ["NEP-17"]
    });

    let parsed: NeoContractManifest =
        serde_json::from_value(manifest_json).expect("manifest should deserialize");
    assert_eq!(parsed.supported_standards, vec!["NEP-17"]);
}

#[test]
fn neo_manifest_serde_accepts_translator_extra_metadata() {
    let manifest_json = serde_json::json!({
        "name": "Demo",
        "abi": {
            "hash": "0x00",
            "methods": [],
            "events": []
        },
        "permissions": [],
        "trusts": [],
        "supportedstandards": [],
        "extra": {
            "version": "1.2.3",
            "author": "Neo",
            "email": "dev@neo.org",
            "description": "Generated"
        }
    });

    let parsed: NeoContractManifest =
        serde_json::from_value(manifest_json).expect("manifest should deserialize");
    assert_eq!(parsed.version, "1.2.3");
    assert_eq!(parsed.author, "Neo");
    assert_eq!(parsed.email, "dev@neo.org");
    assert_eq!(parsed.description, "Generated");
}
