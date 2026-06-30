// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Feature-coverage sample: the `NeoCrypto` hash surface.
//!
//! `NeoCrypto::{sha256, ripemd160, keccak256, keccak512}` are implemented in
//! pure Rust (the `sha2` / `ripemd` / `tiny_keccak` crates) and compile to
//! plain wasm operating on linear memory — so they produce correct,
//! deterministic digests on every target. `murmur32` is a deterministic
//! (non-standard, documented) 32-bit mixer. All of these are exercised here.
//!
//! ## Not covered: signature-verification syscalls
//!
//! `NeoVMSyscall::{check_sig, check_multisig}` and `verify_with_ecdsa` are
//! `neo::*` imports with a multi-buffer ABI (`pubkey_ptr,len, sig_ptr,len …`).
//! The translator marshals linear-memory buffers into NeoVM stack items only
//! for `check_witness` and the script-hash i64 forms; these crypto imports fall
//! through to a bare `SYSCALL` with their pointer/length arguments left on the
//! stack, which is **not** the `CheckSig(pubkey, signature)` calling
//! convention. They are therefore intentionally NOT exercised here (they would
//! translate but fault on chain). The hashing surface above is the part that is
//! correctly bridged on wasm32.

use neo_devpack::prelude::*;

neo_manifest_overlay!(r#"{ "name": "FeatureCrypto" }"#);

#[neo_contract]
pub struct CryptoContract;

/// Build a deterministic input buffer from a seed so each method has
/// non-trivial, seed-dependent input (and the optimizer can't fold the hash
/// away to a constant). Built on a stack array; only the hashers allocate.
fn input(seed: i64) -> NeoByteString {
    let bytes = seed.to_le_bytes();
    let mut buf = [0u8; 64];
    let mut i = 0;
    while i < 64 {
        buf[i] = bytes[i % 8];
        i += 1;
    }
    NeoByteString::from_slice(&buf)
}

#[neo_contract]
impl CryptoContract {
    pub fn new() -> Self {
        Self
    }

    /// `NeoCrypto::sha256` — returns a 32-byte digest (length 32).
    #[neo_method(safe)]
    pub fn sha(seed: i64) -> i64 {
        NeoCrypto::sha256(&input(seed))
            .map(|d| d.len() as i64)
            .unwrap_or(-1)
    }

    /// `NeoCrypto::ripemd160` — returns a 20-byte digest (length 20).
    #[neo_method(safe)]
    pub fn ripe(seed: i64) -> i64 {
        NeoCrypto::ripemd160(&input(seed))
            .map(|d| d.len() as i64)
            .unwrap_or(-1)
    }

    /// `NeoCrypto::keccak256` — returns a 32-byte digest.
    #[neo_method(safe)]
    pub fn kec256(seed: i64) -> i64 {
        NeoCrypto::keccak256(&input(seed))
            .map(|d| d.len() as i64)
            .unwrap_or(-1)
    }

    /// `NeoCrypto::keccak512` — returns a 64-byte digest.
    #[neo_method(safe)]
    pub fn kec512(seed: i64) -> i64 {
        NeoCrypto::keccak512(&input(seed))
            .map(|d| d.len() as i64)
            .unwrap_or(-1)
    }

    /// First byte of the SHA-256 digest (proves the digest is real data, not a
    /// fixed length — covers reading the hash bytes back out).
    #[neo_method(safe)]
    pub fn sha_first(seed: i64) -> i64 {
        match NeoCrypto::sha256(&input(seed)) {
            Ok(d) if !d.is_empty() => d.as_slice()[0] as i64,
            _ => -1,
        }
    }

    /// `NeoCrypto::murmur32` — deterministic 32-bit mix (documented as
    /// non-standard). Returned as a scalar.
    #[neo_method(safe)]
    pub fn mur(seed: i64) -> i64 {
        NeoCrypto::murmur32(&input(seed), NeoInteger::new(0))
            .map(|n| n.as_i64_saturating())
            .unwrap_or(-1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crypto_digest_lengths() {
        assert_eq!(CryptoContract::sha(7), 32);
        assert_eq!(CryptoContract::ripe(7), 20);
        assert_eq!(CryptoContract::kec256(7), 32);
        assert_eq!(CryptoContract::kec512(7), 64);
        // Deterministic: same seed -> same first byte and same murmur.
        assert_eq!(CryptoContract::sha_first(7), CryptoContract::sha_first(7));
        assert_eq!(CryptoContract::mur(7), CryptoContract::mur(7));
    }
}
