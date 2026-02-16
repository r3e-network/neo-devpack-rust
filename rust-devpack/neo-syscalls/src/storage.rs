// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

//! Storage state management for Neo N3 syscall simulation.

use once_cell::sync::Lazy;

#[cfg(not(target_arch = "wasm32"))]
use neo_types::{NeoError, NeoResult, NeoStorageContext};
#[cfg(not(target_arch = "wasm32"))]
use std::collections::{HashMap, HashSet};
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
pub(crate) static ACTIVE_CONTRACT_HASH: Lazy<RwLock<[u8; 20]>> =
    Lazy::new(|| RwLock::new(DEFAULT_CONTRACT_HASH));
#[cfg(not(target_arch = "wasm32"))]
pub(crate) static ACTIVE_WITNESSES: Lazy<RwLock<HashSet<Vec<u8>>>> =
    Lazy::new(|| RwLock::new(HashSet::new()));
#[cfg(not(target_arch = "wasm32"))]
pub(crate) static ACTIVE_CRYPTO_RESULTS: Lazy<RwLock<CryptoVerificationResults>> =
    Lazy::new(|| RwLock::new(CryptoVerificationResults::default()));

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy, Default)]
pub(crate) struct CryptoVerificationResults {
    pub(crate) check_sig: bool,
    pub(crate) check_multisig: bool,
    pub(crate) verify_with_ecdsa: bool,
}

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

    pub(crate) fn reset(&self) -> NeoResult<()> {
        self.contexts
            .write()
            .map_err(|_| NeoError::InvalidState)?
            .clear();
        self.contract_stores
            .write()
            .map_err(|_| NeoError::InvalidState)?
            .clear();
        self.next_context.store(1, Ordering::SeqCst);
        Ok(())
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

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn current_contract_hash() -> [u8; 20] {
    match ACTIVE_CONTRACT_HASH.read() {
        Ok(hash) => *hash,
        Err(poisoned) => *poisoned.into_inner(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn set_current_contract_hash(hash: [u8; 20]) {
    match ACTIVE_CONTRACT_HASH.write() {
        Ok(mut active) => *active = hash,
        Err(poisoned) => *poisoned.into_inner() = hash,
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn reset_current_contract_hash() {
    set_current_contract_hash(DEFAULT_CONTRACT_HASH);
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn set_active_witnesses<I>(witnesses: I)
where
    I: IntoIterator<Item = Vec<u8>>,
{
    let updated: HashSet<Vec<u8>> = witnesses.into_iter().collect();
    match ACTIVE_WITNESSES.write() {
        Ok(mut active) => *active = updated,
        Err(poisoned) => *poisoned.into_inner() = updated,
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn has_active_witness(account: &[u8]) -> bool {
    match ACTIVE_WITNESSES.read() {
        Ok(active) => active.contains(account),
        Err(poisoned) => poisoned.into_inner().contains(account),
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn clear_active_witnesses() {
    match ACTIVE_WITNESSES.write() {
        Ok(mut active) => active.clear(),
        Err(poisoned) => poisoned.into_inner().clear(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn set_crypto_verification_results(results: CryptoVerificationResults) {
    match ACTIVE_CRYPTO_RESULTS.write() {
        Ok(mut active) => *active = results,
        Err(poisoned) => *poisoned.into_inner() = results,
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn active_crypto_verification_results() -> CryptoVerificationResults {
    match ACTIVE_CRYPTO_RESULTS.read() {
        Ok(active) => *active,
        Err(poisoned) => *poisoned.into_inner(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn reset_crypto_verification_results() {
    set_crypto_verification_results(CryptoVerificationResults::default());
}

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
