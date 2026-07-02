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
//! ## On-chain CryptoLib hashing (the `Neo.Crypto.*` bridge)
//!
//! `sha_onchain` / `ripe_onchain` / `sha_onchain_seeded` exercise the OTHER
//! crypto surface: the `neo::crypto_sha256` / `neo::crypto_ripemd160` imports,
//! which the translator lowers to a real
//! `System.Contract.Call(CryptoLib, method, CallFlags.ReadOnly, [data])` —
//! the `(ptr, len)` buffer is marshalled out of wasm linear memory, the args
//! are PACKed into the required Array, and the scoped CryptoLib manifest
//! permission is auto-inserted. The i64 the externs "return" is the digest
//! `ByteString` left on the NeoVM stack; returning it from a method hands the
//! digest to the caller. (On non-wasm32 hosts these methods fall back to the
//! pure-Rust hashers and return the digest length instead.)
//!
//! ## Not covered: check_sig / check_multisig
//!
//! `NeoVMSyscall::{check_sig, check_multisig}` are `neo::*` imports with a
//! multi-buffer ABI that still falls through to a bare `SYSCALL` with their
//! pointer/length arguments left on the stack — **not** the
//! `CheckSig(pubkey, signature)` calling convention. They are intentionally
//! NOT exercised here (they would translate but fault on chain).
//! `verify_with_ecdsa` IS correctly bridged (three marshalled buffers + curve
//! via CryptoLib `verifyWithECDsa`), but needs a real key/signature fixture,
//! so it is exercised by the translator's unit tests instead.

use neo_devpack::prelude::*;

neo_manifest_overlay!(r#"{ "name": "FeatureCrypto" }"#);

// On-chain CryptoLib hash imports (wasm32 only). The translator lowers each
// to `System.Contract.Call(CryptoLib, <method>, CallFlags.ReadOnly, [data])`
// with the `(ptr, len)` buffer marshalled out of linear memory; the i64
// result is the digest `ByteString` left on the NeoVM stack.
#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "neo")]
extern "C" {
    #[link_name = "crypto_sha256"]
    fn neo_crypto_sha256(ptr: i32, len: i32) -> i64;
    #[link_name = "crypto_ripemd160"]
    fn neo_crypto_ripemd160(ptr: i32, len: i32) -> i64;
}

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

    /// On-chain CryptoLib `sha256` of the static input `b"abc"` — the method
    /// returns the 32-byte digest ByteString itself (known vector:
    /// `ba7816bf…0015ad`). On non-wasm32 hosts: the digest length (32).
    #[neo_method(safe)]
    pub fn sha_onchain() -> i64 {
        let data = *b"abc";
        #[cfg(target_arch = "wasm32")]
        {
            // SAFETY: pointer/length cover a live local byte array.
            unsafe { neo_crypto_sha256(data.as_ptr() as i32, data.len() as i32) }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            NeoCrypto::sha256(&NeoByteString::from_slice(&data))
                .map(|d| d.len() as i64)
                .unwrap_or(-1)
        }
    }

    /// On-chain CryptoLib `ripemd160` of `b"abc"` — returns the 20-byte
    /// digest ByteString (known vector: `8eb208f7…5a0bfc`). On non-wasm32
    /// hosts: the digest length (20).
    #[neo_method(safe)]
    pub fn ripe_onchain() -> i64 {
        let data = *b"abc";
        #[cfg(target_arch = "wasm32")]
        {
            // SAFETY: pointer/length cover a live local byte array.
            unsafe { neo_crypto_ripemd160(data.as_ptr() as i32, data.len() as i32) }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            NeoCrypto::ripemd160(&NeoByteString::from_slice(&data))
                .map(|d| d.len() as i64)
                .unwrap_or(-1)
        }
    }

    /// On-chain CryptoLib `sha256` of the heap-built 64-byte seeded input —
    /// exercises marshalling a runtime-computed (non-static) buffer out of
    /// linear memory. Returns the digest ByteString. On non-wasm32 hosts: the
    /// digest length (32).
    #[neo_method(safe)]
    pub fn sha_onchain_seeded(seed: i64) -> i64 {
        let data = input(seed);
        #[cfg(target_arch = "wasm32")]
        {
            // SAFETY: pointer/length cover the NeoByteString's live buffer.
            unsafe { neo_crypto_sha256(data.as_slice().as_ptr() as i32, data.len() as i32) }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            NeoCrypto::sha256(&data).map(|d| d.len() as i64).unwrap_or(-1)
        }
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
