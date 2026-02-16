// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

use neo_devpack::{prelude::*, NeoVMSyscall};
use std::sync::{Mutex, MutexGuard, OnceLock};

fn runtime_test_lock() -> MutexGuard<'static, ()> {
    static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    TEST_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("neo_runtime test lock poisoned")
}

fn setup_runtime_test() -> MutexGuard<'static, ()> {
    let guard = runtime_test_lock();
    NeoVMSyscall::reset_host_state().expect("host syscall state should reset");
    guard
}

#[test]
fn runtime_core_syscalls_return_expected_types() {
    let _guard = setup_runtime_test();
    let timestamp = NeoRuntime::get_time().unwrap();
    assert!(timestamp.as_i32_saturating() >= 0);

    let network = NeoRuntime::get_network().unwrap();
    assert!(network.as_i32_saturating() >= 0);

    let addr_version = NeoRuntime::get_address_version().unwrap();
    assert!(addr_version.as_i32_saturating() >= 0);

    let gas_left = NeoRuntime::get_gas_left().unwrap();
    assert!(gas_left.as_i32_saturating() >= 0);

    let trigger = NeoRuntime::get_trigger().unwrap();
    assert!(trigger.as_i32_saturating() >= 0);
}

#[test]
fn runtime_script_hash_helpers_produce_bytes() {
    let _guard = setup_runtime_test();
    let calling = NeoRuntime::get_calling_script_hash().unwrap();
    let entry = NeoRuntime::get_entry_script_hash().unwrap();
    let executing = NeoRuntime::get_executing_script_hash().unwrap();

    assert_eq!(calling.len(), 20);
    assert_eq!(entry.len(), 20);
    assert_eq!(executing.len(), 20);
}

#[test]
fn runtime_notifications_and_container_are_arrays() {
    let _guard = setup_runtime_test();
    let notifications = NeoRuntime::get_notifications(None).unwrap();
    assert!(notifications.is_empty());

    let container = NeoRuntime::get_script_container().unwrap();
    assert!(container.is_empty());
}

#[test]
fn storage_context_round_trip() {
    let _guard = setup_runtime_test();
    let ctx = NeoStorage::get_context().unwrap();
    assert!(!ctx.is_read_only());

    let read_only = NeoStorage::get_read_only_context().unwrap();
    assert!(read_only.is_read_only());

    let converted = NeoStorage::as_read_only(&ctx).unwrap();
    assert!(converted.is_read_only());
}

#[test]
fn storage_operations_succeed_for_writable_context() {
    let _guard = setup_runtime_test();
    let ctx = NeoStorage::get_context().unwrap();
    let key = NeoByteString::from_slice(b"demo");
    let value = NeoByteString::from_slice(b"value");

    let stored = NeoStorage::get(&ctx, &key).unwrap();
    assert_eq!(stored.len(), 0);

    assert!(NeoStorage::put(&ctx, &key, &value).is_ok());
    assert!(NeoStorage::delete(&ctx, &key).is_ok());

    let iter = NeoStorage::find(&ctx, &key).unwrap();
    assert!(!iter.has_next());
}

#[test]
fn storage_find_returns_struct_entries() {
    let _guard = setup_runtime_test();
    let ctx = NeoStorage::get_context().unwrap();
    let prefix = NeoByteString::from_slice(b"market:");
    let key_a = NeoByteString::from_slice(b"market:alpha");
    let key_b = NeoByteString::from_slice(b"market:beta");
    let val_a = NeoByteString::from_slice(b"one");
    let val_b = NeoByteString::from_slice(b"two");

    NeoStorage::put(&ctx, &key_a, &val_a).unwrap();
    NeoStorage::put(&ctx, &key_b, &val_b).unwrap();

    let mut iter = NeoStorage::find(&ctx, &prefix).unwrap();
    let mut seen = Vec::new();
    while iter.has_next() {
        if let Some(entry) = iter.next() {
            let st = entry.as_struct().expect("expected key/value struct");
            let key_field = st
                .get_field("key")
                .and_then(NeoValue::as_byte_string)
                .expect("missing key field");
            let value_field = st
                .get_field("value")
                .and_then(NeoValue::as_byte_string)
                .expect("missing value field");
            seen.push((key_field.clone(), value_field.clone()));
        }
    }

    assert_eq!(seen.len(), 2);
    assert!(seen
        .iter()
        .any(|(k, v)| k.as_slice() == key_a.as_slice() && v.as_slice() == val_a.as_slice()));
    assert!(seen
        .iter()
        .any(|(k, v)| k.as_slice() == key_b.as_slice() && v.as_slice() == val_b.as_slice()));

    NeoStorage::delete(&ctx, &key_a).unwrap();
    NeoStorage::delete(&ctx, &key_b).unwrap();
}

#[test]
fn storage_put_fails_for_read_only_context() {
    let _guard = setup_runtime_test();
    let ctx = NeoStorage::get_read_only_context().unwrap();
    let key = NeoByteString::from_slice(b"demo");
    let value = NeoByteString::from_slice(b"value");

    let err = NeoStorage::put(&ctx, &key, &value).unwrap_err();
    assert_eq!(err, NeoError::InvalidOperation);
}

#[test]
fn runtime_misc_helpers_work() {
    let _guard = setup_runtime_test();
    let account = NeoByteString::from_slice(b"account");
    assert!(NeoRuntime::check_witness(&account).unwrap().as_bool());

    let event = NeoString::from_str("event");
    let state = NeoArray::<NeoValue>::new();
    assert!(NeoRuntime::notify(&event, &state).is_ok());

    let message = NeoString::from_str("log");
    assert!(NeoRuntime::log(&message).is_ok());

    let platform = NeoRuntime::platform().unwrap();
    assert!(!platform.as_str().is_empty());
}

#[test]
fn crypto_helpers_produce_deterministic_lengths() {
    let _guard = setup_runtime_test();
    let data = NeoByteString::from_slice(b"neo");
    assert_eq!(NeoCrypto::sha256(&data).unwrap().len(), 32);
    assert_eq!(NeoCrypto::ripemd160(&data).unwrap().len(), 20);
    assert_eq!(NeoCrypto::keccak256(&data).unwrap().len(), 32);
    assert_eq!(NeoCrypto::keccak512(&data).unwrap().len(), 64);

    let seed = NeoInteger::new(42);
    let murmur = NeoCrypto::murmur32(&data, seed).unwrap();
    assert!(murmur.as_i32_saturating() != 0);

    let signature = NeoByteString::from_slice(&[0x42; 64]);
    let public_key = NeoByteString::from_slice(&[0x02; 33]);
    assert!(NeoCrypto::verify_signature(&data, &signature, &public_key)
        .unwrap()
        .as_bool());
}

#[test]
fn host_active_contract_hash_controls_storage_partitioning() {
    let _guard = setup_runtime_test();

    let hash_a = NeoByteString::from_slice(&[0x11; 20]);
    let hash_b = NeoByteString::from_slice(&[0x22; 20]);
    let key = NeoByteString::from_slice(b"shared:key");
    let val_a = NeoByteString::from_slice(b"value-a");
    let val_b = NeoByteString::from_slice(b"value-b");

    NeoVMSyscall::set_active_contract_hash(&hash_a).unwrap();
    let ctx_a = NeoStorage::get_context().unwrap();
    NeoStorage::put(&ctx_a, &key, &val_a).unwrap();
    assert_eq!(NeoRuntime::get_executing_script_hash().unwrap(), hash_a);

    NeoVMSyscall::set_active_contract_hash(&hash_b).unwrap();
    let ctx_b = NeoStorage::get_context().unwrap();
    assert_eq!(NeoStorage::get(&ctx_b, &key).unwrap().len(), 0);
    NeoStorage::put(&ctx_b, &key, &val_b).unwrap();
    assert_eq!(NeoRuntime::get_executing_script_hash().unwrap(), hash_b);

    NeoVMSyscall::set_active_contract_hash(&hash_a).unwrap();
    let ctx_a_again = NeoStorage::get_context().unwrap();
    assert_eq!(NeoStorage::get(&ctx_a_again, &key).unwrap(), val_a);

    NeoVMSyscall::set_active_contract_hash(&hash_b).unwrap();
    let ctx_b_again = NeoStorage::get_context().unwrap();
    assert_eq!(NeoStorage::get(&ctx_b_again, &key).unwrap(), val_b);
}
