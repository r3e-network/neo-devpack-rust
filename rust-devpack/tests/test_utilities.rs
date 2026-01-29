//! Shared Test Utilities for rust-devpack
//!
//! This module provides common testing helpers, mock objects, and assertion
//! macros used across all rust-devpack test modules.
//!
//! # Examples
//!
//! ```rust
//! use rust_devpack_tests::test_utilities::*;
//!
//! let mut mock = MockNeoRuntime::new()
//!     .with_balance("NEO", 100)
//!     .with_block_height(1000);
//!
//! assert_eq!(mock.get_balance("NEO"), 100);
//! ```

use std::collections::HashMap;

/// Mock implementation of Neo N3 Runtime for testing
///
/// This mock provides deterministic responses for all NeoRuntime syscalls
/// without requiring a full Neo node or VM instance.
pub struct MockNeoRuntime {
    block_height: u32,
    block_timestamp: u64,
    network: u32,
    gas_left: i64,
    calling_script_hash: Vec<u8>,
    entry_script_hash: Vec<u8>,
    executing_script_hash: Vec<u8>,
    storage: HashMap<Vec<u8>, Vec<u8>>,
    balances: HashMap<String, i64>,
    witnesses: Vec<Vec<u8>>,
    notifications: Vec<MockNotification>,
    trigger: TriggerType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TriggerType {
    Application,
    Verification,
    System,
}

#[derive(Debug, Clone)]
pub struct MockNotification {
    pub script_hash: Vec<u8>,
    pub event_name: String,
    pub state: Vec<u8>,
}

impl MockNeoRuntime {
    /// Creates a new mock runtime with default values
    pub fn new() -> Self {
        Self {
            block_height: 0,
            block_timestamp: 0,
            network: 860833102, // Neo N3 MainNet magic
            gas_left: 1000000000,
            calling_script_hash: vec![0u8; 20],
            entry_script_hash: vec![0u8; 20],
            executing_script_hash: vec![0u8; 20],
            storage: HashMap::new(),
            balances: HashMap::new(),
            witnesses: Vec::new(),
            notifications: Vec::new(),
            trigger: TriggerType::Application,
        }
    }

    /// Sets the current block height
    pub fn with_block_height(mut self, height: u32) -> Self {
        self.block_height = height;
        self
    }

    /// Sets the current block timestamp
    pub fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.block_timestamp = timestamp;
        self
    }

    /// Sets the network magic
    pub fn with_network(mut self, network: u32) -> Self {
        self.network = network;
        self
    }

    /// Sets the available gas
    pub fn with_gas(mut self, gas: i64) -> Self {
        self.gas_left = gas;
        self
    }

    /// Sets a token balance for the mock
    pub fn with_balance(mut self, token: &str, amount: i64) -> Self {
        self.balances.insert(token.to_string(), amount);
        self
    }

    /// Adds a witness that will pass check_witness
    pub fn with_witness(mut self, witness: &[u8]) -> Self {
        self.witnesses.push(witness.to_vec());
        self
    }

    /// Sets the trigger type
    pub fn with_trigger(mut self, trigger: TriggerType) -> Self {
        self.trigger = trigger;
        self
    }

    // Runtime getters

    pub fn get_time(&self) -> u64 {
        self.block_timestamp
    }

    pub fn get_network(&self) -> u32 {
        self.network
    }

    pub fn get_gas_left(&self) -> i64 {
        self.gas_left
    }

    pub fn get_trigger(&self) -> &TriggerType {
        &self.trigger
    }

    pub fn get_block_height(&self) -> u32 {
        self.block_height
    }

    // Storage operations

    pub fn storage_put(&mut self, key: &[u8], value: &[u8]) {
        self.storage.insert(key.to_vec(), value.to_vec());
    }

    pub fn storage_get(&self, key: &[u8]) -> Option<&Vec<u8>> {
        self.storage.get(key)
    }

    pub fn storage_delete(&mut self, key: &[u8]) {
        self.storage.remove(key);
    }

    pub fn storage_find(&self, prefix: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> {
        self.storage
            .iter()
            .filter(|(k, _)| k.starts_with(prefix))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    // Witness check

    pub fn check_witness(&self, hash: &[u8]) -> bool {
        self.witnesses.iter().any(|w| w == hash)
    }

    // Balance

    pub fn get_balance(&self, token: &str) -> i64 {
        *self.balances.get(token).unwrap_or(&0)
    }

    // Notifications

    pub fn notify(&mut self, script_hash: &[u8], event_name: &str, state: &[u8]) {
        self.notifications.push(MockNotification {
            script_hash: script_hash.to_vec(),
            event_name: event_name.to_string(),
            state: state.to_vec(),
        });
    }

    pub fn get_notifications(&self) -> &[MockNotification] {
        &self.notifications
    }
}

impl Default for MockNeoRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// Test data generators for property-based testing
pub struct TestDataGenerators;

impl TestDataGenerators {
    /// Generates valid Neo addresses (20 bytes)
    pub fn valid_address() -> Vec<u8> {
        (0..20).map(|i| i as u8).collect()
    }

    /// Generates a range of test integers
    pub fn test_integers() -> Vec<i32> {
        vec![
            0,
            1,
            -1,
            42,
            -42,
            i32::MAX,
            i32::MIN,
            1000,
            -1000,
            0x7FFFFFFF,
            0x80000000u32 as i32,
        ]
    }

    /// Generates edge case strings
    pub fn test_strings() -> Vec<String> {
        vec![
            "".to_string(),
            "a".to_string(),
            "Hello, Neo!".to_string(),
            "日本語テスト".to_string(),
            "\x00\x01\x02".to_string(),
            "a".repeat(1000),
        ]
    }

    /// Generates test byte arrays
    pub fn test_byte_arrays() -> Vec<Vec<u8>> {
        vec![
            vec![],
            vec![0x00],
            vec![0xFF],
            vec![0x00, 0x01, 0x02, 0x03],
            (0..256).map(|i| i as u8).collect(),
            vec![0x42; 1000],
        ]
    }
}

/// Assertion helpers for Neo types
pub mod assertions {
    use neo_devpack::prelude::*;

    /// Asserts that a NeoInteger equals an expected value
    pub fn assert_int_eq(actual: &NeoInteger, expected: i32) {
        assert_eq!(
            actual.as_i32(),
            expected,
            "Expected NeoInteger to equal {}, got {}",
            expected,
            actual.as_i32()
        );
    }

    /// Asserts that a NeoResult is Ok and applies a predicate
    pub fn assert_ok<T: std::fmt::Debug>(result: &NeoResult<T>) {
        assert!(result.is_ok(), "Expected Ok result, got Err: {:?}", result);
    }

    /// Asserts that a NeoResult is Err with specific error
    pub fn assert_err_eq<T>(result: &NeoResult<T>, expected: NeoError) {
        match result {
            Ok(_) => panic!("Expected Err({:?}), got Ok", expected),
            Err(e) => assert_eq!(*e, expected, "Expected error {:?}, got {:?}", expected, e),
        }
    }

    /// Asserts that a byte string has expected length
    pub fn assert_byte_string_len(bs: &NeoByteString, expected_len: usize) {
        assert_eq!(
            bs.len(),
            expected_len,
            "Expected ByteString length {}, got {}",
            expected_len,
            bs.len()
        );
    }

    /// Asserts that a storage context is valid
    pub fn assert_valid_storage_context(ctx: &NeoStorageContext) {
        assert!(ctx.id() > 0, "Storage context ID should be positive");
    }
}

/// Property-based testing utilities
pub struct PropertyTest;

impl PropertyTest {
    /// Tests a property with multiple random inputs
    pub fn forall<F, T>(inputs: Vec<T>, property: F)
    where
        F: Fn(&T),
        T: std::fmt::Debug,
    {
        for input in &inputs {
            property(input);
        }
    }

    /// Tests that an operation is idempotent: f(f(x)) == f(x)
    pub fn assert_idempotent<T, F>(input: T, operation: F)
    where
        F: Fn(&T) -> T,
        T: PartialEq + std::fmt::Debug,
    {
        let first = operation(&input);
        let second = operation(&first);
        assert_eq!(
            first, second,
            "Operation should be idempotent: f(f({:?})) != f({:?})",
            input, input
        );
    }

    /// Tests that an operation is commutative: f(a, b) == f(b, a)
    pub fn assert_commutative<T, F>(a: T, b: T, operation: F)
    where
        F: Fn(&T, &T) -> T,
        T: PartialEq + std::fmt::Debug,
    {
        let result1 = operation(&a, &b);
        let result2 = operation(&b, &a);
        assert_eq!(
            result1, result2,
            "Operation should be commutative: f({:?}, {:?}) != f({:?}, {:?})",
            a, b, b, a
        );
    }

    /// Tests that an operation is associative: f(f(a, b), c) == f(a, f(b, c))
    pub fn assert_associative<T, F>(a: T, b: T, c: T, operation: F)
    where
        F: Fn(&T, &T) -> T,
        T: PartialEq + std::fmt::Debug + Clone,
    {
        let result1 = operation(&operation(&a, &b), &c);
        let result2 = operation(&a, &operation(&b, &c));
        assert_eq!(
            result1, result2,
            "Operation should be associative: f(f({:?}, {:?}), {:?}) != f({:?}, f({:?}, {:?}))",
            a, b, c, a, b, c
        );
    }
}

/// Fuzz testing seed corpus generators
pub struct FuzzCorpus;

impl FuzzCorpus {
    /// Generates seed inputs for integer operations fuzzing
    pub fn integer_seeds() -> Vec<Vec<u8>> {
        vec![
            vec![0x00, 0x00, 0x00, 0x00],              // Zero
            vec![0x01, 0x00, 0x00, 0x00],              // One
            vec![0xFF, 0xFF, 0xFF, 0x7F],              // i32::MAX
            vec![0x00, 0x00, 0x00, 0x80],              // i32::MIN
            vec![0xFF, 0xFF, 0xFF, 0xFF],              // -1
            (0..100).map(|i| (i * 7) as u8).collect(), // Pattern
        ]
    }

    /// Generates seed inputs for storage operations fuzzing
    pub fn storage_seeds() -> Vec<Vec<u8>> {
        vec![
            vec![],                                      // Empty
            vec![0x00],                                  // Single null
            b"key".to_vec(),                             // ASCII key
            b"prefix:test".to_vec(),                     // With colon
            (0..32).map(|i| i as u8).collect(),          // 32 bytes
            (0..256).map(|i| (i % 256) as u8).collect(), // Full byte range
        ]
    }

    /// Generates seed inputs for string operations fuzzing
    pub fn string_seeds() -> Vec<Vec<u8>> {
        vec![
            vec![],                           // Empty
            vec![0x00],                       // Null
            b"Hello".to_vec(),                // ASCII
            vec![0xE4, 0xB8, 0xAD],           // UTF-8 (中)
            (0..100).map(|_| b'a').collect(), // Repeated
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_runtime_defaults() {
        let runtime = MockNeoRuntime::new();
        assert_eq!(runtime.get_block_height(), 0);
        assert_eq!(runtime.get_network(), 860833102);
        assert_eq!(runtime.get_gas_left(), 1000000000);
    }

    #[test]
    fn test_mock_runtime_builder() {
        let runtime = MockNeoRuntime::new()
            .with_block_height(1000)
            .with_gas(500000)
            .with_balance("NEO", 100)
            .with_witness(&[0x01, 0x02, 0x03]);

        assert_eq!(runtime.get_block_height(), 1000);
        assert_eq!(runtime.get_gas_left(), 500000);
        assert_eq!(runtime.get_balance("NEO"), 100);
        assert!(runtime.check_witness(&[0x01, 0x02, 0x03]));
        assert!(!runtime.check_witness(&[0x04, 0x05, 0x06]));
    }

    #[test]
    fn test_mock_runtime_storage() {
        let mut runtime = MockNeoRuntime::new();

        runtime.storage_put(b"key1", b"value1");
        assert_eq!(runtime.storage_get(b"key1"), Some(&b"value1"[..].to_vec()));

        runtime.storage_put(b"key2", b"value2");
        assert_eq!(runtime.storage_find(b"key").len(), 2);

        runtime.storage_delete(b"key1");
        assert_eq!(runtime.storage_get(b"key1"), None);
    }

    #[test]
    fn test_mock_runtime_notifications() {
        let mut runtime = MockNeoRuntime::new();

        runtime.notify(&[0u8; 20], "Transfer", b"data");
        assert_eq!(runtime.get_notifications().len(), 1);
        assert_eq!(runtime.get_notifications()[0].event_name, "Transfer");
    }

    #[test]
    fn test_data_generators() {
        let addr = TestDataGenerators::valid_address();
        assert_eq!(addr.len(), 20);

        let integers = TestDataGenerators::test_integers();
        assert!(integers.contains(&0));
        assert!(integers.contains(&i32::MAX));
        assert!(integers.contains(&i32::MIN));
    }

    #[test]
    fn test_property_test_commutative() {
        PropertyTest::assert_commutative(5, 10, |a, b| a + b);
    }

    #[test]
    fn test_property_test_associative() {
        PropertyTest::assert_associative(5, 10, 15, |a, b| a + b);
    }

    #[test]
    fn test_fuzz_corpus() {
        assert!(!FuzzCorpus::integer_seeds().is_empty());
        assert!(!FuzzCorpus::storage_seeds().is_empty());
        assert!(!FuzzCorpus::string_seeds().is_empty());
    }
}
