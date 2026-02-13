// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

//! Mock Runtime for Testing

use neo_types::*;
use std::collections::HashMap;

/// Mock storage for testing
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
}

/// Mock runtime for testing contract execution
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
            network: 860905102,
            address_version: 53,
            witnesses: Vec::new(),
            notifications: Vec::new(),
            logs: Vec::new(),
            script_container: None,
            calling_script_hash: None,
            executing_script_hash: None,
            entry_script_hash: None,
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

    pub fn build(self) -> MockRuntime {
        self.runtime
    }
}

impl Default for MockRuntimeBuilder {
    fn default() -> Self {
        Self::new()
    }
}
