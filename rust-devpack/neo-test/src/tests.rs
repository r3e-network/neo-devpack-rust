// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

//! Comprehensive tests for neo-test crate

use crate::assertions::MethodCallResult;
use crate::environment::{ContractTest, TestBuilder, TestEnvironment};
use crate::mock_runtime::{MockRuntime, MockRuntimeBuilder, MockStorage, MockStorageContext};
use neo_types::{NeoArray, NeoBoolean, NeoByteString, NeoError, NeoInteger, NeoString, NeoValue};

#[test]
fn test_mock_storage_basic_operations() {
    let storage = MockStorage::new();

    // Initially empty
    assert!(storage.is_empty());
    assert_eq!(storage.len(), 0);

    // Put value
    let mut storage = storage;
    storage.put(b"key1", b"value1");

    assert!(!storage.is_empty());
    assert_eq!(storage.len(), 1);
    assert!(storage.contains(b"key1"));
    assert_eq!(storage.get(b"key1"), Some(b"value1".to_vec()));

    // Update value
    storage.put(b"key1", b"value2");
    assert_eq!(storage.get(b"key1"), Some(b"value2".to_vec()));
    assert_eq!(storage.len(), 1);

    // Delete value
    storage.delete(b"key1");
    assert!(!storage.contains(b"key1"));
    assert_eq!(storage.len(), 0);
}

#[test]
fn test_mock_storage_find() {
    let mut storage = MockStorage::new();

    storage.put(b"prefix_key1", b"value1");
    storage.put(b"prefix_key2", b"value2");
    storage.put(b"other_key", b"value3");

    let results = storage.find(b"prefix_");
    assert_eq!(results.len(), 2);
}

#[test]
fn test_mock_runtime_basic() {
    let runtime = MockRuntime::new();

    assert_eq!(runtime.network_value(), 860905102); // MainNet
    assert_eq!(runtime.address_version_value(), 53);
    assert_eq!(runtime.time_value(), 0);
    assert_eq!(runtime.trigger_value(), 0);
    assert_eq!(runtime.gas_left(), 100_000_000);
}

#[test]
fn test_mock_runtime_builder() {
    let runtime = MockRuntimeBuilder::new()
        .network(894448701) // TestNet
        .time(1234567890)
        .trigger(0x01)
        .gas(50_000_000)
        .witness(b"test_witness")
        .build();

    assert_eq!(runtime.network_value(), 894448701);
    assert_eq!(runtime.time_value(), 1234567890);
    assert_eq!(runtime.trigger_value(), 0x01);
    assert_eq!(runtime.gas_left(), 50_000_000);
    assert!(runtime.check_witness(b"test_witness"));
}

#[test]
fn test_mock_runtime_witness() {
    let mut runtime = MockRuntime::new();

    // Initially no witnesses
    assert!(!runtime.check_witness(b"addr1"));

    // Add witness
    runtime
        .witnesses_mut()
        .push(NeoByteString::from_slice(b"addr1"));
    assert!(runtime.check_witness(b"addr1"));
    assert!(!runtime.check_witness(b"addr2"));
}

#[test]
fn test_mock_runtime_notifications() {
    let mut runtime = MockRuntime::new();

    // Add notification
    let event = NeoString::from_str("Transfer");
    let state = NeoArray::<NeoValue>::new();
    runtime.add_notification(event, state);

    assert_eq!(runtime.notifications().len(), 1);

    // Clear and verify
    runtime.clear_notifications();
    assert!(runtime.notifications().is_empty());
}

#[test]
fn test_mock_runtime_logs() {
    let mut runtime = MockRuntime::new();

    runtime.add_log("test log 1");
    runtime.add_log("test log 2");

    assert_eq!(runtime.logs().len(), 2);
    assert!(runtime.logs().contains(&"test log 1".to_string()));

    runtime.clear_logs();
    assert!(runtime.logs().is_empty());
}

#[test]
fn test_mock_runtime_storage_operations() {
    let mut runtime = MockRuntime::new();

    // Storage operations
    runtime.storage_put(b"balance:owner", b"1000");
    assert_eq!(
        runtime.storage_get(b"balance:owner"),
        Some(b"1000".to_vec())
    );

    runtime.storage_delete(b"balance:owner");
    assert_eq!(runtime.storage_get(b"balance:owner"), None);
}

#[test]
fn test_mock_runtime_storage_find() {
    let mut runtime = MockRuntime::new();

    runtime.storage_put(b"token_owner", b"address1");
    runtime.storage_put(b"token_balance", b"1000");
    runtime.storage_put(b"other_data", b"value");

    let results = runtime.storage_find(b"token_");
    assert_eq!(results.len(), 2);
}

#[test]
fn test_mock_runtime_gas_consumption() {
    let mut runtime = MockRuntimeBuilder::new().gas(1000).build();

    assert_eq!(runtime.gas_left(), 1000);

    runtime.consume_gas(500);
    assert_eq!(runtime.gas_left(), 500);

    // Can't go below zero
    runtime.consume_gas(1000);
    assert_eq!(runtime.gas_left(), 0);
}

#[test]
fn test_mock_runtime_invocation_counter() {
    let mut runtime = MockRuntime::new();

    assert_eq!(runtime.invocation_counter(), 0);

    runtime.increment_invocation_counter();
    runtime.increment_invocation_counter();
    assert_eq!(runtime.invocation_counter(), 2);
}

#[test]
fn test_mock_storage_context() {
    let ctx = MockStorageContext::new(42);
    assert_eq!(ctx.id, 42);
    assert!(!ctx.is_read_only());

    let ro_ctx = MockStorageContext::read_only(1);
    assert_eq!(ro_ctx.id, 1);
    assert!(ro_ctx.is_read_only());
}

#[test]
fn test_mock_runtime_storage_context_creation() {
    let mut runtime = MockRuntime::new();

    // First context is created by default in MockRuntime::new()
    let ctx = runtime.get_storage_context();
    assert_eq!(ctx.id, 1); // id starts from 1 since 0 is created by default
    assert!(!ctx.is_read_only());

    let ctx2 = runtime.get_storage_context();
    assert_eq!(ctx2.id, 2);

    let ro_ctx = runtime.get_read_only_storage_context();
    assert!(ro_ctx.is_read_only());
}

#[test]
fn test_test_environment_basic() {
    let mut env = TestEnvironment::new();

    // Set storage
    env.set_storage(b"key", b"value");
    assert_eq!(env.get_storage(b"key"), Some(b"value".to_vec()));

    // Delete storage
    env.delete_storage(b"key");
    assert_eq!(env.get_storage(b"key"), None);
}

#[test]
fn test_test_environment_witness() {
    let mut env = TestEnvironment::new();

    env.add_witness(b"test_address");
    assert!(env.check_witness(b"test_address"));
    assert!(!env.check_witness(b"other_address"));
}

#[test]
fn test_test_environment_runtime_settings() {
    let mut env = TestEnvironment::new();

    env.set_trigger(0x01);
    env.set_time(1234567890);
    env.set_network(894448701);

    assert_eq!(env.runtime().trigger_value(), 0x01);
    assert_eq!(env.runtime().time_value(), 1234567890);
    assert_eq!(env.runtime().network_value(), 894448701);
}

#[test]
fn test_test_environment_logs() {
    let mut env = TestEnvironment::new();

    env.add_log("test log");
    assert_eq!(env.logs().len(), 1);

    env.clear_logs();
    assert!(env.logs().is_empty());
}

#[test]
fn test_test_builder_fluent_api() {
    let env = TestBuilder::new()
        .storage(b"owner", b"AV4GGdKS2C7j1GqC3w5y4qX5")
        .storage(b"total_supply", b"1000000")
        .time(1234567890)
        .network(860905102)
        .witness(b"AV4GGdKS2C7j1GqC3w5y4qX5")
        .trigger(0x01)
        .build();

    assert_eq!(
        env.get_storage(b"owner"),
        Some(b"AV4GGdKS2C7j1GqC3w5y4qX5".to_vec())
    );
    assert_eq!(env.get_storage(b"total_supply"), Some(b"1000000".to_vec()));
    assert!(env.check_witness(b"AV4GGdKS2C7j1GqC3w5y4qX5"));
}

#[test]
fn test_storage_assertions() {
    let mut env = TestEnvironment::new();
    env.set_storage(b"key1", b"value1");
    env.set_storage(b"key2", b"value2");

    let assertions = env.assert_storage();

    assertions.assert_contains(b"key1");
    assertions.assert_contains(b"key2");
    assertions.assert_not_contains(b"nonexistent");
    assertions.assert_value(b"key1", b"value1");
}

#[test]
fn test_runtime_assertions() {
    let mut env = TestEnvironment::new();
    env.add_witness(b"test_address");
    env.add_log("test log");

    let assertions = env.assert_runtime();

    assertions.assert_witness(b"test_address");
    assertions.assert_log_count(1);
    assertions.assert_log_contains("test");
    assertions.assert_notification_count(0);
}

#[test]
fn test_contract_test_helper() {
    struct MyContract {
        value: i32,
    }

    impl MyContract {
        fn new() -> Self {
            Self { value: 42 }
        }

        fn get_value(&self) -> i32 {
            self.value
        }
    }

    let test = ContractTest::new(MyContract::new());

    assert_eq!(test.contract().get_value(), 42);
}

#[test]
fn test_mock_runtime_reset() {
    let mut runtime = MockRuntimeBuilder::new().witness(b"test").build();

    runtime.add_log("test");
    runtime.increment_invocation_counter();

    assert!(!runtime.witnesses().is_empty());
    assert!(!runtime.logs().is_empty());
    assert!(runtime.invocation_counter() > 0);

    runtime.reset();

    // Witnesses preserved after reset
    assert!(!runtime.witnesses().is_empty());
    // But logs and counter reset
    assert!(runtime.logs().is_empty());
    assert_eq!(runtime.invocation_counter(), 0);
}

#[test]
fn test_method_call_result_assertions() {
    // Test Ok result
    let result = MethodCallResult::ok(NeoValue::from(NeoInteger::new(42)));
    result.assert_ok();
    result.assert_returns(42);

    // Test Err result
    let result = MethodCallResult::err(NeoError::InvalidOperation);
    result.assert_err();
}

#[test]
fn test_method_call_result_boolean() {
    let result = MethodCallResult::ok(NeoValue::from(NeoBoolean::TRUE));
    result.assert_ok();
    result.assert_returns_bool(true);
}

#[test]
fn test_method_call_result_bytes() {
    let bytes = NeoByteString::from_slice(b"hello");
    let result = MethodCallResult::ok(NeoValue::from(bytes));
    result.assert_ok();
    result.assert_returns_slice(b"hello");
}

#[test]
fn test_method_call_result_string() {
    let s = NeoString::from_str("test string");
    let result = MethodCallResult::ok(NeoValue::from(s));
    result.assert_ok();
    result.assert_returns_string("test string");
}
