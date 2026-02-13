// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

//! Mock Runtime for Testing Neo N3 Smart Contracts
//!
//! This module provides a comprehensive mock runtime environment for testing
//! Neo N3 smart contracts without requiring a full Neo node.

use neo_types::*;
use std::collections::HashMap;

/// Mock storage that simulates Neo blockchain storage
#[derive(Debug, Clone, Default)]
pub struct MockStorage {
    data: HashMap<Vec<u8>, Vec<u8>>,
}

impl MockStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.data.get(key).cloned()
    }

    pub fn put(&mut self, key: &[u8], value: &[u8]) {
        self.data.insert(key.to_vec(), value.to_vec());
    }

    pub fn delete(&mut self, key: &[u8]) {
        self.data.remove(key);
    }

    pub fn contains(&self, key: &[u8]) -> bool {
        self.data.contains_key(key)
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn keys(&self) -> Vec<Vec<u8>> {
        self.data.keys().cloned().collect()
    }

    pub fn find(&self, prefix: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> {
        self.data
            .iter()
            .filter(|(k, _)| k.starts_with(prefix))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

/// Mock storage context for simulating storage operations
#[derive(Debug, Clone)]
pub struct MockStorageContext {
    pub id: u32,
    is_read_only: bool,
}

impl MockStorageContext {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            is_read_only: false,
        }
    }

    pub fn read_only(id: u32) -> Self {
        Self {
            id,
            is_read_only: true,
        }
    }

    pub fn is_read_only(&self) -> bool {
        self.is_read_only
    }
}

/// Mock runtime for testing contract execution
///
/// This provides a complete simulation of the Neo N3 runtime environment
/// for testing smart contracts.
#[derive(Debug, Clone)]
pub struct MockRuntime {
    pub storage: MockStorage,
    pub trigger: i32,
    pub time: i64,
    pub network: i64,
    pub address_version: i32,
    witnesses: Vec<NeoByteString>,
    pub notifications: Vec<(NeoString, NeoArray<NeoValue>)>,
    pub logs: Vec<String>,
    pub script_container: Option<NeoArray<NeoValue>>,
    pub calling_script_hash: Option<NeoByteString>,
    pub executing_script_hash: Option<NeoByteString>,
    pub entry_script_hash: Option<NeoByteString>,
    pub gas_left: i64,
    pub invocation_counter: i32,
    storage_contexts: Vec<MockStorageContext>,
}

impl Default for MockRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl MockRuntime {
    pub fn new() -> Self {
        Self {
            storage: MockStorage::new(),
            trigger: 0,
            time: 0,
            network: 860905102, // MainNet
            address_version: 53,
            witnesses: Vec::new(),
            notifications: Vec::new(),
            logs: Vec::new(),
            script_container: None,
            calling_script_hash: None,
            executing_script_hash: None,
            entry_script_hash: None,
            gas_left: 100_000_000, // 100 GAS
            invocation_counter: 0,
            storage_contexts: vec![MockStorageContext::new(0)],
        }
    }

    pub fn with_storage(mut self, storage: MockStorage) -> Self {
        self.storage = storage;
        self
    }

    pub fn with_trigger(mut self, trigger: i32) -> Self {
        self.trigger = trigger;
        self
    }

    pub fn with_time(mut self, time: i64) -> Self {
        self.time = time;
        self
    }

    pub fn with_network(mut self, network: i64) -> Self {
        self.network = network;
        self
    }

    pub fn with_witness(mut self, address: &[u8]) -> Self {
        self.witnesses.push(NeoByteString::from_slice(address));
        self
    }

    pub fn with_script_container(mut self, container: NeoArray<NeoValue>) -> Self {
        self.script_container = Some(container);
        self
    }

    pub fn with_calling_script_hash(mut self, hash: &[u8]) -> Self {
        self.calling_script_hash = Some(NeoByteString::from_slice(hash));
        self
    }

    pub fn with_executing_script_hash(mut self, hash: &[u8]) -> Self {
        self.executing_script_hash = Some(NeoByteString::from_slice(hash));
        self
    }

    pub fn with_entry_script_hash(mut self, hash: &[u8]) -> Self {
        self.entry_script_hash = Some(NeoByteString::from_slice(hash));
        self
    }

    pub fn with_gas_left(mut self, gas: i64) -> Self {
        self.gas_left = gas;
        self
    }

    pub fn storage_ref(&self) -> &MockStorage {
        &self.storage
    }

    pub fn storage_mut(&mut self) -> &mut MockStorage {
        &mut self.storage
    }

    pub fn trigger_value(&self) -> i32 {
        self.trigger
    }

    pub fn time_value(&self) -> i64 {
        self.time
    }

    pub fn network_value(&self) -> i64 {
        self.network
    }

    pub fn address_version_value(&self) -> i32 {
        self.address_version
    }

    pub fn witnesses(&self) -> &[NeoByteString] {
        &self.witnesses
    }

    pub fn witnesses_mut(&mut self) -> &mut Vec<NeoByteString> {
        &mut self.witnesses
    }

    pub fn notifications(&self) -> &[(NeoString, NeoArray<NeoValue>)] {
        &self.notifications
    }

    pub fn logs(&self) -> &[String] {
        &self.logs
    }

    pub fn clear_notifications(&mut self) {
        self.notifications.clear();
    }

    pub fn clear_logs(&mut self) {
        self.logs.clear();
    }

    pub fn add_log(&mut self, message: &str) {
        self.logs.push(message.to_string());
    }

    pub fn add_notification(&mut self, event: NeoString, state: NeoArray<NeoValue>) {
        self.notifications.push((event, state));
    }

    pub fn check_witness(&self, hash: &[u8]) -> bool {
        self.witnesses.iter().any(|w| w.as_slice() == hash)
    }

    pub fn calling_script_hash(&self) -> Option<&NeoByteString> {
        self.calling_script_hash.as_ref()
    }

    pub fn executing_script_hash(&self) -> Option<&NeoByteString> {
        self.executing_script_hash.as_ref()
    }

    pub fn entry_script_hash(&self) -> Option<&NeoByteString> {
        self.entry_script_hash.as_ref()
    }

    pub fn gas_left(&self) -> i64 {
        self.gas_left
    }

    pub fn consume_gas(&mut self, amount: i64) {
        if amount <= 0 {
            return;
        }

        self.gas_left = self.gas_left.checked_sub(amount).unwrap_or(0).max(0);
    }

    pub fn invocation_counter(&self) -> i32 {
        self.invocation_counter
    }

    pub fn increment_invocation_counter(&mut self) {
        self.invocation_counter += 1;
    }

    pub fn get_storage_context(&mut self) -> MockStorageContext {
        let id = self.storage_contexts.len() as u32;
        self.storage_contexts.push(MockStorageContext::new(id));
        MockStorageContext::new(id)
    }

    pub fn get_read_only_storage_context(&mut self) -> MockStorageContext {
        let id = self.storage_contexts.len() as u32;
        self.storage_contexts
            .push(MockStorageContext::read_only(id));
        MockStorageContext::read_only(id)
    }

    /// Simulate storage get operation
    pub fn storage_get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.storage.get(key)
    }

    /// Simulate storage put operation
    pub fn storage_put(&mut self, key: &[u8], value: &[u8]) {
        self.storage.put(key, value);
    }

    /// Simulate storage put operation with storage context validation.
    pub fn storage_put_with_context(
        &mut self,
        context: &MockStorageContext,
        key: &[u8],
        value: &[u8],
    ) -> NeoResult<()> {
        self.ensure_context_valid(context)?;
        if context.is_read_only() {
            return Err(NeoError::InvalidOperation);
        }

        self.storage.put(key, value);
        Ok(())
    }

    /// Simulate storage delete operation
    pub fn storage_delete(&mut self, key: &[u8]) {
        self.storage.delete(key);
    }

    /// Simulate storage delete operation with storage context validation.
    pub fn storage_delete_with_context(
        &mut self,
        context: &MockStorageContext,
        key: &[u8],
    ) -> NeoResult<()> {
        self.ensure_context_valid(context)?;
        if context.is_read_only() {
            return Err(NeoError::InvalidOperation);
        }

        self.storage.delete(key);
        Ok(())
    }

    /// Simulate storage find operation
    pub fn storage_find(&self, prefix: &[u8]) -> Vec<(Vec<u8>, Vec<u8>)> {
        self.storage.find(prefix)
    }

    /// Reset the runtime state
    pub fn reset(&mut self) {
        self.notifications.clear();
        self.logs.clear();
        self.invocation_counter = 0;
    }

    /// Reset known storage contexts to the runtime default context.
    pub fn clear_storage_contexts(&mut self) {
        self.storage_contexts.clear();
        self.storage_contexts.push(MockStorageContext::new(0));
    }

    fn ensure_context_valid(&self, context: &MockStorageContext) -> NeoResult<()> {
        let is_known_context = self
            .storage_contexts
            .iter()
            .any(|ctx| ctx.id == context.id && ctx.is_read_only == context.is_read_only);
        if !is_known_context {
            return Err(NeoError::InvalidArgument);
        }

        Ok(())
    }
}

/// Builder for creating mock runtime
pub struct MockRuntimeBuilder {
    runtime: MockRuntime,
}

impl MockRuntimeBuilder {
    pub fn new() -> Self {
        Self {
            runtime: MockRuntime::new(),
        }
    }

    pub fn storage(mut self, storage: MockStorage) -> Self {
        self.runtime.storage = storage;
        self
    }

    pub fn trigger(mut self, trigger: i32) -> Self {
        self.runtime.trigger = trigger;
        self
    }

    pub fn time(mut self, time: i64) -> Self {
        self.runtime.time = time;
        self
    }

    pub fn network(mut self, network: i64) -> Self {
        self.runtime.network = network;
        self
    }

    pub fn witness(mut self, address: &[u8]) -> Self {
        self.runtime
            .witnesses
            .push(NeoByteString::from_slice(address));
        self
    }

    pub fn script_hash(mut self, hash: &[u8]) -> Self {
        self.runtime.executing_script_hash = Some(NeoByteString::from_slice(hash));
        self
    }

    pub fn gas(mut self, gas: i64) -> Self {
        self.runtime.gas_left = gas;
        self
    }

    pub fn build(self) -> MockRuntime {
        self.runtime
    }
}

impl Default for MockRuntimeBuilder {
    fn default() -> Self {
        Self::new()
    }
}
