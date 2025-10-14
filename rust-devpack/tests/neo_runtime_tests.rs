use neo_devpack::prelude::*;

#[test]
fn runtime_core_syscalls_return_expected_types() {
    let timestamp = NeoRuntime::get_time().unwrap();
    assert!(timestamp.as_i32() >= 0);

    let network = NeoRuntime::get_network().unwrap();
    assert!(network.as_i32() >= 0);

    let addr_version = NeoRuntime::get_address_version().unwrap();
    assert!(addr_version.as_i32() >= 0);

    let gas_left = NeoRuntime::get_gas_left().unwrap();
    assert!(gas_left.as_i32() >= 0);

    let trigger = NeoRuntime::get_trigger().unwrap();
    assert!(trigger.as_i32() >= 0);
}

#[test]
fn runtime_script_hash_helpers_produce_bytes() {
    let calling = NeoRuntime::get_calling_script_hash().unwrap();
    let entry = NeoRuntime::get_entry_script_hash().unwrap();
    let executing = NeoRuntime::get_executing_script_hash().unwrap();

    assert_eq!(calling.len(), 20);
    assert_eq!(entry.len(), 20);
    assert_eq!(executing.len(), 20);
}

#[test]
fn runtime_notifications_and_container_are_arrays() {
    let notifications = NeoRuntime::get_notifications(None).unwrap();
    assert!(notifications.is_empty());

    let container = NeoRuntime::get_script_container().unwrap();
    assert!(container.is_empty());
}

#[test]
fn storage_context_round_trip() {
    let ctx = NeoStorage::get_context().unwrap();
    assert!(!ctx.is_read_only());

    let read_only = NeoStorage::get_read_only_context().unwrap();
    assert!(read_only.is_read_only());

    let converted = NeoStorage::as_read_only(&ctx).unwrap();
    assert!(converted.is_read_only());
}

#[test]
fn storage_operations_succeed_for_writable_context() {
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
fn storage_put_fails_for_read_only_context() {
    let ctx = NeoStorage::get_read_only_context().unwrap();
    let key = NeoByteString::from_slice(b"demo");
    let value = NeoByteString::from_slice(b"value");

    let err = NeoStorage::put(&ctx, &key, &value).unwrap_err();
    assert_eq!(err, NeoError::InvalidOperation);
}

#[test]
fn runtime_misc_helpers_work() {
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
    let data = NeoByteString::from_slice(b"neo");
    assert_eq!(NeoCrypto::sha256(&data).unwrap().len(), 32);
    assert_eq!(NeoCrypto::ripemd160(&data).unwrap().len(), 20);
    assert_eq!(NeoCrypto::keccak256(&data).unwrap().len(), 32);
    assert_eq!(NeoCrypto::keccak512(&data).unwrap().len(), 64);

    let seed = NeoInteger::new(42);
    let murmur = NeoCrypto::murmur32(&data, seed).unwrap();
    assert!(murmur.as_i32() != 0);

    let signature = NeoByteString::from_slice(&[0x42; 64]);
    let public_key = NeoByteString::from_slice(&[0x02; 33]);
    assert!(NeoCrypto::verify_signature(&data, &signature, &public_key)
        .unwrap()
        .as_bool());
}
