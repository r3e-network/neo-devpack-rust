//! Integration-style checks for the devpack facade.

use neo_devpack::prelude::*;
use neo_syscalls::*;

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
    assert!(ctx.trigger().unwrap().as_i32() >= 0);
    assert!(ctx.gas_left().unwrap().as_i32() >= 0);

    let time = NeoRuntime::get_time().unwrap();
    assert!(time.as_i32() >= 0);

    let random = NeoRuntime::get_random().unwrap();
    assert!(random.as_i32() >= 0);

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
