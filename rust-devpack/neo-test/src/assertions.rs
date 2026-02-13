// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

//! Test Assertions

use crate::mock_runtime::{MockRuntime, MockStorage};
use neo_types::*;

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
        let actual = self
            .result
            .as_ref()
            .ok()
            .and_then(|v| v.as_integer())
            .map(|i| i.as_i64_saturating())
            .unwrap_or(0);
        assert_eq!(
            actual, expected,
            "Expected return value {}, got {}",
            expected, actual
        );
    }

    pub fn assert_returns_bool(&self, expected: bool) {
        let actual = self
            .result
            .as_ref()
            .ok()
            .and_then(|v| v.as_boolean())
            .map(|b| b.as_bool())
            .unwrap_or(false);
        assert_eq!(
            actual, expected,
            "Expected return value {}, got {}",
            expected, actual
        );
    }

    pub fn assert_returns_slice(&self, expected: &[u8]) {
        let actual = self
            .result
            .as_ref()
            .ok()
            .and_then(|v| v.as_byte_string())
            .map(|s| s.as_slice())
            .unwrap_or(&[]);
        assert_eq!(
            actual, expected,
            "Expected return value {:?}, got {:?}",
            expected, actual
        );
    }

    pub fn assert_returns_string(&self, expected: &str) {
        let actual = self
            .result
            .as_ref()
            .ok()
            .and_then(|v| v.as_string())
            .map(|s| s.as_str())
            .unwrap_or("");
        assert_eq!(
            actual, expected,
            "Expected return value '{}', got '{}'",
            expected, actual
        );
    }

    pub fn value(&self) -> &NeoValue {
        self.result.as_ref().unwrap()
    }

    pub fn error(&self) -> Option<&NeoError> {
        self.result.as_ref().err()
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
