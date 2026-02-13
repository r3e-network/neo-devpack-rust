// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

//! Storage state management for Neo N3 syscall simulation.

use once_cell::sync::Lazy;

#[cfg(not(target_arch = "wasm32"))]
use neo_types::{NeoError, NeoResult, NeoStorageContext};
#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::atomic::{AtomicU32, Ordering};
#[cfg(target_arch = "wasm32")]
use std::sync::Mutex;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Arc, RwLock};

#[cfg(not(target_arch = "wasm32"))]
pub(crate) const DEFAULT_CONTRACT_HASH: [u8; 20] = [0u8; 20];

#[cfg(not(target_arch = "wasm32"))]
pub(crate) type ContractStore = Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone)]
pub(crate) struct ContextHandle {
    pub(crate) read_only: bool,
    pub(crate) contract: [u8; 20],
    pub(crate) store: ContractStore,
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::type_complexity)]
pub(crate) struct StorageState {
    next_context: AtomicU32,
    contexts: RwLock<HashMap<u32, ContextHandle>>,
    contract_stores: RwLock<HashMap<[u8; 20], ContractStore>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl StorageState {
    pub(crate) fn new() -> Self {
        Self {
            next_context: AtomicU32::new(1),
            contexts: RwLock::new(HashMap::new()),
            contract_stores: RwLock::new(HashMap::new()),
        }
    }

    pub(crate) fn create_context(
        &self,
        contract: [u8; 20],
        read_only: bool,
    ) -> NeoResult<NeoStorageContext> {
        let store = self.get_or_create_store(contract);
        let id = self.next_context.fetch_add(1, Ordering::SeqCst);
        let handle = ContextHandle {
            read_only,
            contract,
            store,
        };
        self.contexts
            .write()
            .map_err(|_| NeoError::InvalidState)?
            .insert(id, handle);
        Ok(if read_only {
            NeoStorageContext::read_only(id)
        } else {
            NeoStorageContext::new(id)
        })
    }

    pub(crate) fn clone_as_read_only(
        &self,
        context: &NeoStorageContext,
    ) -> NeoResult<NeoStorageContext> {
        let handle = self
            .contexts
            .read()
            .map_err(|_| NeoError::InvalidState)?
            .get(&context.id())
            .cloned()
            .ok_or(NeoError::InvalidState)?;
        let id = self.next_context.fetch_add(1, Ordering::SeqCst);
        let ro_handle = ContextHandle {
            read_only: true,
            contract: handle.contract,
            store: handle.store,
        };
        self.contexts
            .write()
            .map_err(|_| NeoError::InvalidState)?
            .insert(id, ro_handle);
        Ok(NeoStorageContext::read_only(id))
    }

    pub(crate) fn get_handle(&self, context: &NeoStorageContext) -> NeoResult<ContextHandle> {
        self.contexts
            .read()
            .map_err(|_| NeoError::InvalidState)?
            .get(&context.id())
            .cloned()
            .ok_or(NeoError::InvalidState)
    }

    fn get_or_create_store(&self, contract: [u8; 20]) -> ContractStore {
        let mut stores = match self.contract_stores.write() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        stores
            .entry(contract)
            .or_insert_with(|| Arc::new(RwLock::new(HashMap::new())))
            .clone()
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) static STORAGE_STATE: Lazy<StorageState> = Lazy::new(StorageState::new);

#[cfg(target_arch = "wasm32")]
pub(crate) type StorageEntry = (Vec<u8>, Vec<u8>);

#[cfg(target_arch = "wasm32")]
pub(crate) static STORAGE_ENTRIES: Lazy<Mutex<Vec<StorageEntry>>> =
    Lazy::new(|| Mutex::new(Vec::new()));

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn create_context_recovers_when_store_lock_is_poisoned() {
        let state = StorageState::new();
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = state.contract_stores.write().unwrap();
            panic!("poison contract stores lock");
        }));

        let result = state.create_context(DEFAULT_CONTRACT_HASH, false);
        assert!(result.is_ok());
    }
}
