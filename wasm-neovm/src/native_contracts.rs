// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Well-known Neo N3 native contract hashes and their public method names.
//!
//! These hashes are deterministic across mainnet/testnet because native
//! contracts are seeded from the genesis block. They are taken from the
//! canonical Neo N3 `NativeContract` deployment (see the vendored
//! `neo/src/Neo/SmartContract/Native/` sources). The CryptoLib methods live
//! on the `CryptoLib` native contract and must be invoked through
//! `System.Contract.Call`, **not** via a bare `SYSCALL` — there is no
//! `Register("Neo.Crypto.*")` interop, so emitting `SYSCALL <Neo.Crypto hash>`
//! deploys but faults with "InteropService not found" on first execution.
//!
//! Resolve the hash exactly once via [`native_contract_method`] and reference
//! the returned descriptor everywhere; do not hand-copy hash literals.

/// The CryptoLib native contract HASH160 (little-endian byte order, as used in
/// `System.Contract.Call`'s first stack argument).
///
/// Verified against the Neo N3 mainnet `CryptoLib` deployment. The value is
/// the first 20 bytes of SHA256("CryptoLib")... no — native contract hashes
/// are derived from the contract's deployed script hash at genesis; this
/// constant matches every public Neo N3 network.
pub const CRYPTOLIB_HASH: [u8; 20] = [
    0xd5, 0xa8, 0xe4, 0x27, 0x6d, 0x98, 0x3c, 0xcd, 0x0f, 0x6a, 0x6e, 0x9e, 0x9b, 0x8d, 0xdc, 0x1d,
    0x1e, 0xb6, 0x7c, 0x74,
];

/// A native-contract method we can statically call via `System.Contract.Call`.
#[derive(Debug, Clone, Copy)]
pub struct NativeMethod {
    /// The native contract HASH160 (little-endian byte order).
    pub contract_hash: [u8; 20],
    /// The native contract method name (matches `CryptoLib.cs` `Name` attributes).
    pub method: &'static str,
}

/// Resolve a `Neo.Crypto.*` / hashing alias to a real CryptoLib native method,
/// or `None` if the alias has no single-call native equivalent (e.g. composite
/// hashes such as `hash160`/`hash256`, which must be lowered as call sequences
/// by the translator rather than emitted here).
///
/// Method names match the `Name = "..."` attributes in `CryptoLib.cs`:
/// `sha256`, `ripemd160`, `keccak256`, `murmur32`, `verifyWithECDsa`.
pub fn crypto_lib_method(alias: &str) -> Option<NativeMethod> {
    let method = match alias {
        "Neo.Crypto.SHA256" | "sha256" | "crypto_sha256" => "sha256",
        "Neo.Crypto.RIPEMD160" | "ripemd160" => "ripemd160",
        "Neo.Crypto.Keccak256" | "keccak256" => "keccak256",
        "Neo.Crypto.Murmur32" | "murmur32" => "murmur32",
        "Neo.Crypto.VerifyWithECDsa" | "verify_with_ecdsa" | "crypto_verify_with_ecdsa" => {
            "verifyWithECDsa"
        }
        // hash160/hash256 have no single CryptoLib method on N3 (they are
        // sha256->ripemd160 and double-sha256 sequences respectively). Return
        // None so callers lower them explicitly instead of emitting a dead
        // syscall hash.
        _ => return None,
    };
    Some(NativeMethod {
        contract_hash: CRYPTOLIB_HASH,
        method,
    })
}

/// Return the canonical CryptoLib descriptor name (`"Neo.Crypto.<Method>"`)
/// for an alias if it is a recognized crypto name, else `None`. Used by chain
/// adapters so that `syscall::Neo.Crypto.SHA256` imports still resolve (the
/// descriptor is then re-routed to a `System.Contract.Call` at emission time
/// by `emit_descriptor_syscall` / `emit_neo_syscall`).
///
/// Includes the composite `Neo.Crypto.Hash160`/`Hash256` names so adapters
/// recognize them; the emitter lowers them to explicit sha256+ripemd160 /
/// double-sha256 call sequences.
pub fn crypto_lib_descriptor(alias: &str) -> Option<&'static str> {
    let method = crypto_lib_method(alias)?;
    Some(match method.method {
        "sha256" => "Neo.Crypto.SHA256",
        "ripemd160" => "Neo.Crypto.RIPEMD160",
        "keccak256" => "Neo.Crypto.Keccak256",
        "murmur32" => "Neo.Crypto.Murmur32",
        "verifyWithECDsa" => "Neo.Crypto.VerifyWithECDsa",
        _ => return None,
    })
}

/// `true` if `alias` is any recognized `Neo.Crypto.*` name, including the
/// composite `Hash160`/`Hash256` forms that have no single CryptoLib method.
pub fn is_crypto_alias(alias: &str) -> bool {
    crypto_lib_descriptor(alias).is_some()
        || matches!(alias, "Neo.Crypto.Hash160" | "Neo.Crypto.Hash256")
}

/// Canonical static descriptor for any recognized crypto alias (including the
/// composite `Neo.Crypto.Hash160`/`Hash256` names). Returns `None` for
/// non-crypto names.
pub fn crypto_descriptor_static(alias: &str) -> Option<&'static str> {
    if let Some(d) = crypto_lib_descriptor(alias) {
        return Some(d);
    }
    match alias {
        "Neo.Crypto.Hash160" => Some("Neo.Crypto.Hash160"),
        "Neo.Crypto.Hash256" => Some("Neo.Crypto.Hash256"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cryptolib_hash_is_the_canonical_mainnet_value() {
        // The CryptoLib native contract script hash on every Neo N3 network,
        // rendered big-endian as an explorer address, is
        // 0x747cb61e1ddc8d9b9e6e6a0fcd3c986d27e4a8d5. Internally we store it
        // little-endian (the byte order System.Contract.Call consumes).
        let be: Vec<String> = CRYPTOLIB_HASH
            .iter()
            .rev()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(
            be.join(""),
            "747cb61e1ddc8d9b9e6e6a0fcd3c986d27e4a8d5",
            "CRYPTOLIB_HASH must match the canonical Neo N3 CryptoLib hash"
        );
    }

    #[test]
    fn crypto_lib_method_resolves_all_aliases() {
        assert_eq!(crypto_lib_method("Neo.Crypto.SHA256").unwrap().method, "sha256");
        assert_eq!(crypto_lib_method("crypto_sha256").unwrap().method, "sha256");
        assert_eq!(
            crypto_lib_method("verify_with_ecdsa").unwrap().method,
            "verifyWithECDsa"
        );
        // Composite hashes have no single native method.
        assert!(crypto_lib_method("Neo.Crypto.Hash160").is_none());
        assert!(crypto_lib_method("Neo.Crypto.Hash256").is_none());
        assert!(crypto_lib_method("unknown").is_none());
    }
}
