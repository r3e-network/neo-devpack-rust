// Copyright (c) 2025-2026 R3E Network
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
pub(crate) const MAX_SCRIPT_HASH_STACK_DEPTH: usize = 1024;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) const DEFAULT_CALL_FLAGS: i32 = 0x0F;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) type ContractStore = Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy)]
pub(crate) struct ActiveScriptHashes {
    pub(crate) calling: [u8; 20],
    pub(crate) entry: [u8; 20],
    pub(crate) executing: [u8; 20],
    pub(crate) call_flags: i32,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for ActiveScriptHashes {
    fn default() -> Self {
        Self {
            calling: DEFAULT_CONTRACT_HASH,
            entry: DEFAULT_CONTRACT_HASH,
            executing: DEFAULT_CONTRACT_HASH,
            call_flags: DEFAULT_CALL_FLAGS,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) static ACTIVE_SCRIPT_HASHES: Lazy<RwLock<ActiveScriptHashes>> =
    Lazy::new(|| RwLock::new(ActiveScriptHashes::default()));
#[cfg(not(target_arch = "wasm32"))]
pub(crate) static ACTIVE_SCRIPT_HASH_STACK: Lazy<RwLock<Vec<ActiveScriptHashes>>> =
    Lazy::new(|| RwLock::new(Vec::new()));
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
pub(crate) fn current_calling_script_hash() -> [u8; 20] {
    match ACTIVE_SCRIPT_HASHES.read() {
        Ok(active) => active.calling,
        Err(poisoned) => poisoned.into_inner().calling,
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn current_entry_script_hash() -> [u8; 20] {
    match ACTIVE_SCRIPT_HASHES.read() {
        Ok(active) => active.entry,
        Err(poisoned) => poisoned.into_inner().entry,
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn current_executing_script_hash() -> [u8; 20] {
    match ACTIVE_SCRIPT_HASHES.read() {
        Ok(active) => active.executing,
        Err(poisoned) => poisoned.into_inner().executing,
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn set_current_contract_hash(hash: [u8; 20]) {
    set_current_script_hashes(hash, hash, hash);
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn set_current_script_hashes(calling: [u8; 20], entry: [u8; 20], executing: [u8; 20]) {
    match ACTIVE_SCRIPT_HASHES.write() {
        Ok(mut active) => {
            active.calling = calling;
            active.entry = entry;
            active.executing = executing;
        }
        Err(poisoned) => {
            let mut active = poisoned.into_inner();
            active.calling = calling;
            active.entry = entry;
            active.executing = executing;
        }
    }
    clear_script_hash_stack();
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn set_current_calling_script_hash(hash: [u8; 20]) {
    match ACTIVE_SCRIPT_HASHES.write() {
        Ok(mut active) => active.calling = hash,
        Err(poisoned) => poisoned.into_inner().calling = hash,
    }
    clear_script_hash_stack();
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn set_current_entry_script_hash(hash: [u8; 20]) {
    match ACTIVE_SCRIPT_HASHES.write() {
        Ok(mut active) => active.entry = hash,
        Err(poisoned) => poisoned.into_inner().entry = hash,
    }
    clear_script_hash_stack();
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn set_current_executing_script_hash(hash: [u8; 20]) {
    match ACTIVE_SCRIPT_HASHES.write() {
        Ok(mut active) => active.executing = hash,
        Err(poisoned) => poisoned.into_inner().executing = hash,
    }
    clear_script_hash_stack();
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn current_call_flags() -> i32 {
    match ACTIVE_SCRIPT_HASHES.read() {
        Ok(active) => active.call_flags,
        Err(poisoned) => poisoned.into_inner().call_flags,
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn set_current_call_flags(flags: i32) {
    match ACTIVE_SCRIPT_HASHES.write() {
        Ok(mut active) => active.call_flags = flags,
        Err(poisoned) => poisoned.into_inner().call_flags = flags,
    }
    clear_script_hash_stack();
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn push_current_executing_script_hash(hash: [u8; 20], call_flags: i32) -> NeoResult<()> {
    let mut active = match ACTIVE_SCRIPT_HASHES.write() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let mut stack = match ACTIVE_SCRIPT_HASH_STACK.write() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    if stack.len() >= MAX_SCRIPT_HASH_STACK_DEPTH {
        return Err(NeoError::InvalidOperation);
    }

    stack.push(*active);
    active.calling = active.executing;
    active.executing = hash;
    active.call_flags = call_flags;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn pop_current_script_hash_frame() -> NeoResult<()> {
    let mut active = match ACTIVE_SCRIPT_HASHES.write() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    let mut stack = match ACTIVE_SCRIPT_HASH_STACK.write() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    if let Some(previous) = stack.pop() {
        *active = previous;
        Ok(())
    } else {
        Err(NeoError::InvalidState)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn clear_script_hash_stack() {
    match ACTIVE_SCRIPT_HASH_STACK.write() {
        Ok(mut stack) => stack.clear(),
        Err(poisoned) => poisoned.into_inner().clear(),
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn reset_current_contract_hash() {
    set_current_script_hashes(
        DEFAULT_CONTRACT_HASH,
        DEFAULT_CONTRACT_HASH,
        DEFAULT_CONTRACT_HASH,
    );
    set_current_call_flags(DEFAULT_CALL_FLAGS);
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
