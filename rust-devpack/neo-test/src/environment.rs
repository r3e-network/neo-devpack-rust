// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

//! Test Environment

use crate::assertions::{RuntimeAssertions, StorageAssertions};
use crate::mock_runtime::{MockRuntime, MockStorageContext};
use neo_types::NeoByteString;

pub type TestResult<T = ()> = Result<T, TestError>;

#[derive(Debug, Clone)]
pub struct TestError {
    pub message: String,
    pub context: String,
}

impl TestError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            context: String::new(),
        }
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = context.into();
        self
    }
}

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.context.is_empty() {
            write!(f, "{}", self.message)
        } else {
            write!(f, "{}: {}", self.context, self.message)
        }
    }
}

impl std::error::Error for TestError {}

/// Test environment for Neo N3 contract testing
pub struct TestEnvironment {
    runtime: MockRuntime,
    deployment: Option<DeploymentState>,
}

#[derive(Debug, Clone)]
struct DeploymentState {
    script: Vec<u8>,
    manifest: Vec<u8>,
}

impl TestEnvironment {
    pub fn new() -> Self {
        Self {
            runtime: MockRuntime::new(),
            deployment: None,
        }
    }

    pub fn with_runtime(mut self, runtime: MockRuntime) -> Self {
        self.runtime = runtime;
        self
    }

    pub fn runtime(&self) -> &MockRuntime {
        &self.runtime
    }

    pub fn runtime_mut(&mut self) -> &mut MockRuntime {
        &mut self.runtime
    }

    pub fn set_storage(&mut self, key: &[u8], value: &[u8]) {
        self.runtime.storage_mut().put(key, value);
    }

    pub fn get_storage_context(&mut self) -> MockStorageContext {
        self.runtime.get_storage_context()
    }

    pub fn get_read_only_storage_context(&mut self) -> MockStorageContext {
        self.runtime.get_read_only_storage_context()
    }

    pub fn put_storage_with_context(
        &mut self,
        context: &MockStorageContext,
        key: &[u8],
        value: &[u8],
    ) -> Result<(), neo_types::NeoError> {
        self.runtime.storage_put_with_context(context, key, value)
    }

    pub fn delete_storage_with_context(
        &mut self,
        context: &MockStorageContext,
        key: &[u8],
    ) -> Result<(), neo_types::NeoError> {
        self.runtime.storage_delete_with_context(context, key)
    }

    pub fn get_storage(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.runtime.storage_ref().get(key)
    }

    pub fn delete_storage(&mut self, key: &[u8]) {
        self.runtime.storage_mut().delete(key);
    }

    pub fn set_trigger(&mut self, trigger: i32) {
        self.runtime.trigger = trigger;
    }

    pub fn set_time(&mut self, time: i64) {
        self.runtime.time = time;
    }

    pub fn set_network(&mut self, network: i64) {
        self.runtime.network = network;
    }

    pub fn add_witness(&mut self, address: &[u8]) {
        self.runtime
            .witnesses_mut()
            .push(NeoByteString::from_slice(address));
    }

    pub fn check_witness(&self, address: &[u8]) -> bool {
        self.runtime.check_witness(address)
    }

    pub fn add_log(&mut self, message: &str) {
        self.runtime.add_log(message);
    }

    pub fn logs(&self) -> &[String] {
        self.runtime.logs()
    }

    pub fn clear_logs(&mut self) {
        self.runtime.clear_logs();
    }

    pub fn assert_runtime(&self) -> RuntimeAssertions<'_> {
        RuntimeAssertions::new(&self.runtime)
    }

    pub fn assert_storage(&self) -> StorageAssertions<'_> {
        StorageAssertions::new(self.runtime.storage_ref())
    }

    pub fn call_method<F, R>(&self, _name: &str, _args: &[neo_types::NeoValue], _f: F) -> R
    where
        F: FnOnce() -> R,
    {
        _f()
    }

    pub fn deploy(&mut self, script: &[u8], manifest: &[u8]) -> TestResult {
        if self.deployment.is_some() {
            return Err(TestError::new("contract is already deployed").with_context("deploy"));
        }
        if script.is_empty() {
            return Err(TestError::new("script cannot be empty").with_context("deploy"));
        }
        if manifest.is_empty() {
            return Err(TestError::new("manifest cannot be empty").with_context("deploy"));
        }

        self.deployment = Some(DeploymentState {
            script: script.to_vec(),
            manifest: manifest.to_vec(),
        });
        Ok(())
    }

    pub fn update(&mut self, script: &[u8]) -> TestResult {
        let existing_manifest = self
            .deployment
            .as_ref()
            .ok_or_else(|| TestError::new("contract is not deployed").with_context("update"))?
            .manifest
            .clone();
        self.update_with_manifest(script, &existing_manifest)
    }

    pub fn update_with_manifest(&mut self, script: &[u8], manifest: &[u8]) -> TestResult {
        if script.is_empty() {
            return Err(TestError::new("script cannot be empty").with_context("update"));
        }
        if manifest.is_empty() {
            return Err(TestError::new("manifest cannot be empty").with_context("update"));
        }

        let deployment = self
            .deployment
            .as_mut()
            .ok_or_else(|| TestError::new("contract is not deployed").with_context("update"))?;
        deployment.script.clear();
        deployment.script.extend_from_slice(script);
        deployment.manifest.clear();
        deployment.manifest.extend_from_slice(manifest);
        Ok(())
    }

    pub fn destroy(&mut self) -> TestResult {
        if self.deployment.take().is_none() {
            return Err(TestError::new("contract is not deployed").with_context("destroy"));
        }

        // Simulate ContractManagement.Destroy semantics by wiping contract storage state.
        self.runtime.storage_mut().clear();
        self.runtime.clear_storage_contexts();

        Ok(())
    }

    pub fn is_deployed(&self) -> bool {
        self.deployment.is_some()
    }

    pub fn deployed_script(&self) -> Option<&[u8]> {
        self.deployment
            .as_ref()
            .map(|deployment| deployment.script.as_slice())
    }

    pub fn deployed_manifest(&self) -> Option<&[u8]> {
        self.deployment
            .as_ref()
            .map(|deployment| deployment.manifest.as_slice())
    }

    pub fn reset(&mut self) {
        self.runtime = MockRuntime::new();
        self.deployment = None;
    }
}

impl Default for TestEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ContractTest<T> {
    env: TestEnvironment,
    contract: T,
}

impl<T> ContractTest<T> {
    pub fn new(contract: T) -> Self {
        Self {
            env: TestEnvironment::new(),
            contract,
        }
    }

    pub fn with_env(mut self, env: TestEnvironment) -> Self {
        self.env = env;
        self
    }

    pub fn env(&self) -> &TestEnvironment {
        &self.env
    }

    pub fn env_mut(&mut self) -> &mut TestEnvironment {
        &mut self.env
    }

    pub fn contract(&self) -> &T {
        &self.contract
    }

    pub fn contract_mut(&mut self) -> &mut T {
        &mut self.contract
    }
}

pub struct TestBuilder {
    env: TestEnvironment,
}

impl TestBuilder {
    pub fn new() -> Self {
        Self {
            env: TestEnvironment::new(),
        }
    }

    pub fn storage(mut self, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> Self {
        self.env.set_storage(key.as_ref(), value.as_ref());
        self
    }

    pub fn time(mut self, time: i64) -> Self {
        self.env.set_time(time);
        self
    }

    pub fn network(mut self, network: i64) -> Self {
        self.env.set_network(network);
        self
    }

    pub fn witness(mut self, address: impl AsRef<[u8]>) -> Self {
        self.env.add_witness(address.as_ref());
        self
    }

    pub fn trigger(mut self, trigger: i32) -> Self {
        self.env.set_trigger(trigger);
        self
    }

    pub fn build(self) -> TestEnvironment {
        self.env
    }
}

impl Default for TestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[macro_export]
macro_rules! assert_neo {
    ($expr:expr, $expected:expr) => {
        assert_eq!($expr.as_i32_saturating(), $expected, "Assertion failed")
    };
}

#[macro_export]
macro_rules! test_env {
    () => {{
        neo_test::TestEnvironment::new()
    }};
}
