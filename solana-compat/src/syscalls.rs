// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

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

    // Storage - maps to System.Storage.Get/Put/Delete.
    //
    // NOTE: these link names must match the canonical neo-syscalls ABI that the
    // wasm-neovm translator's import dispatch recognizes (`neo_storage_*`).
    // Using the bare `storage_*` names produced WASM imports the translator
    // rejected ("unknown Neo syscall import"), so any solana-compat contract
    // touching storage failed to translate. The signatures already match.
    #[link_name = "storage_get"]
    fn neo_storage_get(key: i32, key_len: i32) -> i64;

    // Storage read into a caller-supplied buffer (X9): writes the stored value
    // into `out` (capacity `out_cap`) and returns the number of bytes written,
    // or -1 if the key is absent / the value exceeds `out_cap`.
    #[link_name = "neo_storage_get_into"]
    fn neo_storage_get_into(key: i32, key_len: i32, out: i32, out_cap: i32) -> i32;

    #[link_name = "neo_storage_put_bytes"]
    fn neo_storage_put(key: i32, key_len: i32, value: i32, value_len: i32);

    #[link_name = "neo_storage_delete_bytes"]
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

/// Log a message (`sol_log` equivalent)
///
/// Maps to: System.Runtime.Log
///
/// # Safety
///
/// This function is safe because it only passes valid string references to the NeoVM runtime.
/// The `message` reference is guaranteed to be valid for the duration of the call.
pub fn sol_log(message: &str) {
    #[cfg(target_arch = "wasm32")]
    // SAFETY: We're passing a valid string slice to the NeoVM runtime.
    // The pointer and length come from a valid Rust reference.
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
///
/// Returns the clock as **seconds** since the Unix epoch (Solana's
/// `Clock::unix_timestamp` unit). `System.Runtime.GetTime` on Neo N3 returns
/// milliseconds, so we divide by 1000 (X13: was returning milliseconds, off by
/// 1000x vs Solana deadlines).
///
/// # Safety
///
/// This function wraps the NeoVM syscall which returns an i64 value.
/// It is safe because the syscall does not access any caller-provided memory.
pub fn sol_get_clock_sysvar() -> i64 {
    #[cfg(target_arch = "wasm32")]
    // SAFETY: The NeoVM syscall returns a simple i64 value without
    // accessing any caller-provided memory.
    unsafe {
        neo_get_time() / 1000
    }
    #[cfg(not(target_arch = "wasm32"))]
    0
}

/// SHA256 hash
///
/// Maps to: Neo.Crypto.SHA256
///
/// # Safety
///
/// This function is safe because:
/// - `data` is a valid byte slice reference
/// - `output` is a valid mutable reference to a 32-byte array
///
/// Both references are guaranteed valid for the duration of the call.
pub fn sol_sha256(data: &[u8], output: &mut [u8; 32]) {
    #[cfg(target_arch = "wasm32")]
    // SAFETY: We're passing valid references to the NeoVM runtime.
    // data.as_ptr() points to valid data, and output.as_mut_ptr() points to a 32-byte buffer.
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
/// **Not implemented.** Solana's `sol_verify_signature` performs Ed25519
/// verification of `signature` over `message` against `pubkey`. The Neo N3
/// CryptoLib does not expose Ed25519, so a faithful implementation requires a
/// host bridge that is not yet wired.
///
/// This function intentionally **does not return a witness-probe dressed up as
/// a valid signature** (the previous behaviour answered "is this pubkey a
/// signer?" regardless of whether they signed this message — a security
/// footgun). It panics at first use with a clear message so callers fail
/// loudly instead of getting false cryptographic confidence (X7).
///
/// # Safety
///
/// This function does not access any caller-provided memory; it always panics.
pub fn sol_verify_signature(signature: &[u8; 64], pubkey: &Pubkey, message: &[u8]) -> bool {
    let _ = (signature, pubkey, message);
    unimplemented!(
        "sol_verify_signature: Ed25519 verification requires a Neo host bridge that is not \
         yet wired. This stub refuses to return a witness-probe as a valid signature (X7)."
    )
}

/// Invoke another program (CPI)
///
/// Maps to: System.Contract.Call
///
/// # Safety
///
/// This function is safe because:
/// - `program_id`, `method`, and `args` are all valid references
///
/// All pointers are derived from valid Rust references guaranteed to outlive the call.
pub fn sol_invoke(program_id: &Pubkey, method: &str, args: &[u8]) -> Result<(), u64> {
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: All pointers come from valid references that outlive this call.
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
///
/// Read from storage (simulates account data read).
///
/// Fills the caller's `buffer` with the stored value and returns
/// `Some(bytes_written)`, or `None` if the key is absent or the value does not
/// fit in `buffer` (X9: previously returned `Some(0)` and never wrote a byte,
/// so Solana programs read all-zero account data).
///
/// # Safety
///
/// This function is safe: `key` and `buffer` are valid references; the wasm
/// import writes only within `buffer`'s bounds via the `out_cap` argument.
pub fn storage_read(key: &[u8], buffer: &mut [u8]) -> Option<usize> {
    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: key.as_ptr()/buffer.as_mut_ptr() come from valid references;
        // the import writes at most buffer.len() bytes (passed as out_cap).
        let written = unsafe {
            neo_storage_get_into(
                key.as_ptr() as i32,
                key.len() as i32,
                buffer.as_mut_ptr() as i32,
                buffer.len() as i32,
            )
        };
        if written < 0 {
            None
        } else {
            Some(written as usize)
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (key, buffer);
        None
    }
}

/// Write to storage (simulates account data write)
///
/// # Safety
///
/// This function is safe because `key` and `data` are valid references.
pub fn storage_write(key: &[u8], data: &[u8]) {
    #[cfg(target_arch = "wasm32")]
    // SAFETY: Both pointers come from valid references.
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
///
/// # Safety
///
/// This function is safe because `key` is a valid reference.
pub fn storage_delete(key: &[u8]) {
    #[cfg(target_arch = "wasm32")]
    // SAFETY: The pointer comes from a valid reference.
    unsafe {
        neo_storage_delete(key.as_ptr() as i32, key.len() as i32);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = key;
    }
}
