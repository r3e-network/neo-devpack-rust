// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! Integration-style checks for the devpack facade.

use neo_devpack::prelude::*;
use neo_syscalls::*;
use std::sync::{Mutex, MutexGuard, OnceLock};

fn host_state_lock() -> MutexGuard<'static, ()> {
    static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    match TEST_LOCK.get_or_init(|| Mutex::new(())).lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn setup_host_state() -> MutexGuard<'static, ()> {
    let guard = host_state_lock();
    NeoVMSyscall::reset_host_state().expect("host syscall state should reset");
    guard
}

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
    assert_eq!(registry.iter().count(), 38);
    assert!(registry.has_syscall("System.Runtime.GetTime"));
    assert!(registry.has_syscall("System.Storage.Get"));
    assert!(registry.has_syscall("System.Contract.Call"));
    assert!(registry.has_syscall("Neo.Crypto.VerifyWithECDsa"));
}

#[test]
fn runtime_surface_is_coherent() {
    let _guard = setup_host_state();
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
    let _guard = setup_host_state();
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
    let _guard = setup_host_state();
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
    let _guard = setup_host_state();
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
fn native_contract_hashes_are_exposed_via_prelude() {
    assert_eq!(NEO_CONTRACT, "0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5");
    assert_eq!(
        ORACLE_CONTRACT,
        "0xfe924b7cfe89ddd271abaf7210a80a7e11178758"
    );
    assert_eq!(neo_contract_hash().len(), 20);
    assert_eq!(oracle_contract_hash().len(), 20);
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

#[test]
fn nep24_royalty_helper_computes_basis_points() {
    let sale_price = NeoInteger::new(1_000_000u32);
    let royalty = compute_bps_royalty(&sale_price, 500).expect("royalty should compute");
    assert_eq!(royalty.as_i32_saturating(), 50_000);

    let err = compute_bps_royalty(&sale_price, 10_001).expect_err("bps > 10000 must fail");
    assert!(err.message().contains("bps cannot exceed 10000"));
}

struct LifecycleHarness;

impl Nep26Lifecycle for LifecycleHarness {}

#[test]
fn nep26_lifecycle_helpers_delegate_runtime_calls() {
    let harness = LifecycleHarness;
    let script_hash = NeoByteString::from_slice(b"hash");
    let nef = NeoByteString::from_slice(b"nef");
    let manifest = NeoContractManifest {
        name: "Lifecycle".to_string(),
        version: "1.0.0".to_string(),
        author: "Neo".to_string(),
        email: "dev@neo.org".to_string(),
        description: "Lifecycle test".to_string(),
        abi: NeoContractABI {
            hash: "0x00".to_string(),
            methods: Vec::new(),
            events: Vec::new(),
        },
        permissions: Vec::new(),
        trusts: Vec::new(),
        supported_standards: vec![NEP26_STANDARD.to_string()],
    };

    harness
        .update_contract(&script_hash, &nef, &manifest)
        .expect("update should succeed");
    harness
        .destroy_contract(&script_hash)
        .expect("destroy should succeed");
}

#[test]
fn common_supported_standards_include_extended_neps() {
    let standards = common_supported_standards();
    assert!(standards.contains(&NEP17_STANDARD));
    assert!(standards.contains(&NEP11_STANDARD));
    assert!(standards.contains(&NEP24_STANDARD));
    assert!(standards.contains(&NEP26_STANDARD));
}

struct StandardsHarness;

impl Nep22Update for StandardsHarness {
    fn update(
        &self,
        _nef_file: NeoByteString,
        _manifest: NeoString,
        _data: NeoValue,
    ) -> NeoResult<()> {
        Ok(())
    }
}

impl Nep26Receiver for StandardsHarness {
    fn on_nep11_payment(
        &self,
        _from: NeoByteString,
        _amount: NeoInteger,
        _token_id: NeoByteString,
        _data: NeoValue,
    ) -> NeoResult<()> {
        Ok(())
    }
}

impl Nep24Royalty for StandardsHarness {
    fn royalty_info(
        &self,
        _token_id: &NeoByteString,
        _royalty_token: &NeoByteString,
        _sale_price: &NeoInteger,
    ) -> NeoResult<Vec<Nep24RoyaltyRecipient>> {
        Ok(Vec::new())
    }
}

impl Nep24RoyaltyStack for StandardsHarness {
    fn royalty_info_stack(
        &self,
        _token_id: NeoByteString,
        _royalty_token: NeoByteString,
        _sale_price: NeoInteger,
    ) -> NeoResult<NeoArray<NeoValue>> {
        Ok(NeoArray::new())
    }
}

impl Nep27Receiver for StandardsHarness {
    fn on_nep17_payment(
        &self,
        _from: NeoByteString,
        _amount: NeoInteger,
        _data: NeoValue,
    ) -> NeoResult<()> {
        Ok(())
    }
}

impl Nep29Deploy for StandardsHarness {
    fn deploy(&self, _data: NeoValue, _update: NeoBoolean) -> NeoResult<()> {
        Ok(())
    }
}

impl Nep30Verify for StandardsHarness {
    fn verify(&self) -> NeoResult<NeoBoolean> {
        Ok(NeoBoolean::new(true))
    }
}

impl Nep31Destroy for StandardsHarness {
    fn destroy(&self) -> NeoResult<()> {
        Ok(())
    }
}

#[test]
fn standards_constants_cover_extended_neps() {
    assert_eq!(NEP_11, "NEP-11");
    assert_eq!(NEP_17, "NEP-17");
    assert_eq!(NEP_22, "NEP-22");
    assert_eq!(NEP_24, "NEP-24");
    assert_eq!(NEP_26, "NEP-26");
    assert_eq!(NEP_27, "NEP-27");
    assert_eq!(NEP_29, "NEP-29");
    assert_eq!(NEP_30, "NEP-30");
    assert_eq!(NEP_31, "NEP-31");
    assert!(LIFECYCLE_STANDARDS.contains(&"NEP-29"));
    assert!(CALLBACK_STANDARDS.contains(&"NEP-27"));
}

#[test]
fn standards_traits_are_usable() {
    let harness = StandardsHarness;
    let any = NeoValue::from(NeoInteger::new(1));
    let bytes = NeoByteString::from_slice(b"neo");

    harness
        .update(bytes.clone(), NeoString::from_str("{}"), any.clone())
        .unwrap();
    harness
        .on_nep11_payment(
            bytes.clone(),
            NeoInteger::new(1),
            bytes.clone(),
            any.clone(),
        )
        .unwrap();
    harness
        .on_nep17_payment(bytes.clone(), NeoInteger::new(1), any.clone())
        .unwrap();
    harness
        .royalty_info(&bytes.clone(), &bytes.clone(), &NeoInteger::new(100))
        .unwrap();
    harness
        .royalty_info_stack(bytes.clone(), bytes.clone(), NeoInteger::new(100))
        .unwrap();
    harness.deploy(any, NeoBoolean::new(false)).unwrap();
    assert!(harness.verify().unwrap().as_bool());
    harness.destroy().unwrap();
}
