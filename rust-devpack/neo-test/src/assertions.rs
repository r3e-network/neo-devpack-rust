// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! Test Assertions

use crate::mock_runtime::{MockRuntime, MockStorage};
use neo_types::*;
use num_traits::{Signed, ToPrimitive};

fn saturating_i64(value: &NeoInteger) -> i64 {
    value.as_bigint().to_i64().unwrap_or_else(|| {
        if value.as_bigint().is_negative() {
            i64::MIN
        } else {
            i64::MAX
        }
    })
}

/// Result of a contract method call for assertions
pub struct MethodCallResult {
    result: Result<NeoValue, NeoError>,
}

impl MethodCallResult {
    pub fn new(result: Result<NeoValue, NeoError>) -> Self {
        Self { result }
    }

    pub fn ok(result: NeoValue) -> Self {
        Self { result: Ok(result) }
    }

    pub fn err(error: NeoError) -> Self {
        Self { result: Err(error) }
    }

    pub fn assert_ok(&self) {
        assert!(
            self.result.is_ok(),
            "Expected Ok, got Err: {:?}",
            self.result.as_ref().err()
        );
    }

    pub fn assert_err(&self) {
        assert!(
            self.result.is_err(),
            "Expected Err, got Ok: {:?}",
            self.result.as_ref().ok()
        );
    }

    pub fn assert_returns(&self, expected: i64) {
        let value = self.expect_ok_value();
        let integer = value
            .as_integer()
            .unwrap_or_else(|| panic!("Expected integer return value, got {:?}", value));
        let actual = saturating_i64(integer);
        assert_eq!(
            actual, expected,
            "Expected return value {}, got {}",
            expected, actual
        );
    }

    pub fn assert_returns_bool(&self, expected: bool) {
        let value = self.expect_ok_value();
        let actual = value
            .as_boolean()
            .unwrap_or_else(|| panic!("Expected boolean return value, got {:?}", value))
            .as_bool();
        assert_eq!(
            actual, expected,
            "Expected return value {}, got {}",
            expected, actual
        );
    }

    pub fn assert_returns_slice(&self, expected: &[u8]) {
        let value = self.expect_ok_value();
        let actual = value
            .as_byte_string()
            .unwrap_or_else(|| panic!("Expected byte string return value, got {:?}", value))
            .as_slice();
        assert_eq!(
            actual, expected,
            "Expected return value {:?}, got {:?}",
            expected, actual
        );
    }

    pub fn assert_returns_string(&self, expected: &str) {
        let value = self.expect_ok_value();
        let actual = value
            .as_string()
            .unwrap_or_else(|| panic!("Expected string return value, got {:?}", value))
            .as_str();
        assert_eq!(
            actual, expected,
            "Expected return value '{}', got '{}'",
            expected, actual
        );
    }

    pub fn assert_error(&self, expected: NeoError) {
        match self.result.as_ref() {
            Ok(value) => panic!("Expected Err({:?}), got Ok({:?})", expected, value),
            Err(actual) => {
                assert_eq!(
                    actual, &expected,
                    "Expected error {:?}, got {:?}",
                    expected, actual
                )
            }
        }
    }

    pub fn value(&self) -> &NeoValue {
        self.expect_ok_value()
    }

    pub fn error(&self) -> Option<&NeoError> {
        self.result.as_ref().err()
    }

    fn expect_ok_value(&self) -> &NeoValue {
        match self.result.as_ref() {
            Ok(value) => value,
            Err(error) => panic!("Expected Ok result, got Err: {:?}", error),
        }
    }
}

impl<T: Into<NeoValue>> From<Result<T, NeoError>> for MethodCallResult {
    fn from(result: Result<T, NeoError>) -> Self {
        Self::new(result.map(|v| v.into()))
    }
}

/// Storage assertions
pub struct StorageAssertions<'a> {
    storage: &'a MockStorage,
}

impl<'a> StorageAssertions<'a> {
    pub fn new(storage: &'a MockStorage) -> Self {
        Self { storage }
    }

    pub fn contains(&self, key: &[u8]) -> bool {
        self.storage.contains(key)
    }

    pub fn assert_contains(&self, key: &[u8]) {
        assert!(
            self.storage.contains(key),
            "Expected storage to contain key {:?}",
            key
        );
    }

    pub fn assert_not_contains(&self, key: &[u8]) {
        assert!(
            !self.storage.contains(key),
            "Expected storage to NOT contain key {:?}",
            key
        );
    }

    pub fn assert_value(&self, key: &[u8], expected: &[u8]) {
        let actual = self.storage.get(key);
        assert_eq!(
            actual.as_deref(),
            Some(expected),
            "Expected storage key {:?} to have value {:?}, got {:?}",
            key,
            expected,
            actual
        );
    }

    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.storage.get(key)
    }
}

/// Runtime assertions
pub struct RuntimeAssertions<'a> {
    runtime: &'a MockRuntime,
}

impl<'a> RuntimeAssertions<'a> {
    pub fn new(runtime: &'a MockRuntime) -> Self {
        Self { runtime }
    }

    pub fn assert_log_contains(&self, message: &str) {
        let logs: &[String] = self.runtime.logs();
        assert!(
            logs.iter().any(|l: &String| l.contains(message)),
            "Expected log to contain '{}', got {:?}",
            message,
            logs
        );
    }

    pub fn assert_log_count(&self, count: usize) {
        assert_eq!(
            self.runtime.logs().len(),
            count,
            "Expected {} logs, got {}",
            count,
            self.runtime.logs().len()
        );
    }

    pub fn assert_notification_count(&self, count: usize) {
        assert_eq!(
            self.runtime.notifications().len(),
            count,
            "Expected {} notifications, got {}",
            count,
            self.runtime.notifications().len()
        );
    }

    pub fn assert_witness(&self, address: &[u8]) {
        assert!(
            self.runtime.check_witness(address),
            "Expected witness for address {:?}",
            address
        );
    }

    pub fn logs(&self) -> &[String] {
        self.runtime.logs()
    }

    pub fn notifications(&self) -> &[(NeoString, NeoArray<NeoValue>)] {
        self.runtime.notifications()
    }
}
