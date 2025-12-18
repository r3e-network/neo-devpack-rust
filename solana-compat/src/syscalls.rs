//! Syscall bindings for Solana compatibility
//!
//! Maps Solana syscalls to Neo interop services.

use crate::pubkey::Pubkey;

// ============================================================================
// Neo Syscall Imports
// ============================================================================

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "neo")]
extern "C" {
    // Logging - maps to System.Runtime.Log
    #[link_name = "runtime_log"]
    fn neo_log(message: i32, len: i32);

    // Storage - maps to System.Storage.Get/Put/Delete
    #[link_name = "storage_get"]
    fn neo_storage_get(key: i32, key_len: i32) -> i64;

    #[link_name = "storage_put"]
    fn neo_storage_put(key: i32, key_len: i32, value: i32, value_len: i32);

    #[link_name = "storage_delete"]
    fn neo_storage_delete(key: i32, key_len: i32);

    // Runtime - maps to System.Runtime.*
    #[link_name = "runtime_get_time"]
    fn neo_get_time() -> i64;

    #[link_name = "runtime_check_witness"]
    fn neo_check_witness(hash: i32) -> i32;

    // Crypto - maps to Neo.Crypto.*
    #[link_name = "crypto_sha256"]
    fn neo_sha256(data: i32, len: i32, out: i32);

    #[link_name = "crypto_hash160"]
    fn neo_hash160(data: i32, len: i32, out: i32);

    // Contract calls - maps to System.Contract.Call
    #[link_name = "contract_call"]
    fn neo_contract_call(hash: i32, method: i32, method_len: i32, args: i32, args_len: i32) -> i64;
}

// ============================================================================
// Solana-Compatible Syscall Wrappers
// ============================================================================

/// Log a message (sol_log equivalent)
///
/// Maps to: System.Runtime.Log
pub fn sol_log(message: &str) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        neo_log(message.as_ptr() as i32, message.len() as i32);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        // For testing on native target
        let _ = message;
    }
}

/// Log a 64-bit value
pub fn sol_log_64(arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64) {
    // Format as hex and log
    let _ = (arg1, arg2, arg3, arg4, arg5);
    sol_log("sol_log_64 called");
}

/// Log compute units (no-op in Neo)
pub fn sol_log_compute_units() {
    sol_log("compute_units: N/A in Neo");
}

/// Get current Unix timestamp
///
/// Maps to: System.Runtime.GetTime
pub fn sol_get_clock_sysvar() -> i64 {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        neo_get_time()
    }
    #[cfg(not(target_arch = "wasm32"))]
    0
}

/// SHA256 hash
///
/// Maps to: Neo.Crypto.SHA256
pub fn sol_sha256(data: &[u8], output: &mut [u8; 32]) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        neo_sha256(
            data.as_ptr() as i32,
            data.len() as i32,
            output.as_mut_ptr() as i32,
        );
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (data, output);
    }
}

/// Keccak256 hash (mapped to available Neo hash)
pub fn sol_keccak256(data: &[u8], output: &mut [u8; 32]) {
    use tiny_keccak::{Hasher, Keccak};

    let mut hasher = Keccak::v256();
    hasher.update(data);
    hasher.finalize(output);
}

/// Verify Ed25519 signature
///
/// Note: Neo uses different signature schemes (secp256r1, secp256k1)
/// This is a compatibility stub that uses CheckWitness
pub fn sol_verify_signature(signature: &[u8; 64], pubkey: &Pubkey, message: &[u8]) -> bool {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        // Derive script hash from the pubkey to check witness against the account identity.
        let mut hash160 = [0u8; 20];
        neo_hash160(
            pubkey.as_ref().as_ptr() as i32,
            pubkey.as_ref().len() as i32,
            hash160.as_mut_ptr() as i32,
        );
        let _ = signature;
        let _ = message;
        neo_check_witness(hash160.as_ptr() as i32) != 0
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (signature, pubkey, message);
        false
    }
}

/// Invoke another program (CPI)
///
/// Maps to: System.Contract.Call
pub fn sol_invoke(program_id: &Pubkey, method: &str, args: &[u8]) -> Result<(), u64> {
    #[cfg(target_arch = "wasm32")]
    {
        let result = unsafe {
            neo_contract_call(
                program_id.as_ref().as_ptr() as i32,
                method.as_ptr() as i32,
                method.len() as i32,
                args.as_ptr() as i32,
                args.len() as i32,
            )
        };
        if result == 0 {
            Ok(())
        } else {
            Err(result as u64)
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (program_id, method, args);
        Ok(())
    }
}

// ============================================================================
// Storage Operations (Account Data Simulation)
// ============================================================================

/// Read from storage (simulates account data read)
pub fn storage_read(key: &[u8], buffer: &mut [u8]) -> Option<usize> {
    #[cfg(target_arch = "wasm32")]
    {
        let result = unsafe { neo_storage_get(key.as_ptr() as i32, key.len() as i32) };
        let _ = buffer;
        // Without a concrete storage bridge we cannot safely populate the buffer yet.
        if result < 0 {
            None
        } else {
            None
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (key, buffer);
        None
    }
}

/// Write to storage (simulates account data write)
pub fn storage_write(key: &[u8], data: &[u8]) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        neo_storage_put(
            key.as_ptr() as i32,
            key.len() as i32,
            data.as_ptr() as i32,
            data.len() as i32,
        );
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (key, data);
    }
}

/// Delete from storage
pub fn storage_delete(key: &[u8]) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        neo_storage_delete(key.as_ptr() as i32, key.len() as i32);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = key;
    }
}
