// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

mod generated {
    include!(concat!(env!("OUT_DIR"), "/syscalls.rs"));
}

pub use generated::SyscallInfo;

/// Extended syscall info for descriptors not in the generated table.
///
/// NOTE: `Neo.Crypto.*` names are **not** registered interop services on Neo
/// N3 — they are methods on the `CryptoLib` native contract and must be
/// invoked via `System.Contract.Call`. They are intentionally absent from
/// this table; the translator re-routes them in `emit_neo_syscall` using
/// [`crate::native_contracts::crypto_lib_method`]. `System.Crypto.CheckSig`
/// and `System.Crypto.CheckMultisig` are real registered syscalls and live in
/// the generated table (hashes `0x27B3E756` / `0x3ADCD09E`).
static EXTENDED_SYSCALLS: &[SyscallInfo] = &[];

/// Return all generated Neo N3 syscalls.
pub fn all() -> &'static [SyscallInfo] {
    generated::SYSCALLS
}

/// Return the extended syscall table (native contract methods).
pub fn extended() -> &'static [SyscallInfo] {
    EXTENDED_SYSCALLS
}

/// Look up a syscall by its canonical name (case-insensitive).
pub fn lookup(name: &str) -> Option<&'static SyscallInfo> {
    generated::SYSCALLS
        .iter()
        .find(|info| info.name.eq_ignore_ascii_case(name))
}

/// Extended lookup that includes native contract methods
pub fn lookup_extended(name: &str) -> Option<&'static SyscallInfo> {
    // First try the generated syscalls
    if let Some(info) = lookup(name) {
        return Some(info);
    }
    // Then try extended syscalls (native contract methods)
    EXTENDED_SYSCALLS
        .iter()
        .find(|info| info.name.eq_ignore_ascii_case(name))
}

/// Look up a syscall by its 4-byte hash.
pub fn lookup_by_hash(hash: u32) -> Option<&'static SyscallInfo> {
    generated::SYSCALLS
        .iter()
        .find(|info| info.hash == hash)
        .or_else(|| EXTENDED_SYSCALLS.iter().find(|info| info.hash == hash))
}

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};

    use super::*;

    fn syscall_hash(name: &str) -> u32 {
        let mut hasher = Sha256::new();
        hasher.update(name.as_bytes());
        let digest = hasher.finalize();
        u32::from_le_bytes([digest[0], digest[1], digest[2], digest[3]])
    }

    #[test]
    fn extended_syscalls_have_correct_hashes() {
        for info in EXTENDED_SYSCALLS {
            assert_eq!(
                info.hash,
                syscall_hash(info.name),
                "extended syscall hash mismatch for {}",
                info.name
            );
        }
    }

    #[test]
    fn lookup_by_hash_includes_extended_entries() {
        for info in EXTENDED_SYSCALLS {
            let resolved = lookup_by_hash(info.hash)
                .unwrap_or_else(|| panic!("expected lookup_by_hash for {}", info.name));
            assert_eq!(resolved.name, info.name);
        }
    }

    #[test]
    fn extended_hashes_do_not_conflict_with_generated_syscalls() {
        for extra in EXTENDED_SYSCALLS {
            if let Some(generated) = generated::SYSCALLS.iter().find(|s| s.hash == extra.hash) {
                assert_eq!(
                    generated.name, extra.name,
                    "hash collision between generated '{}' and extended '{}'",
                    generated.name, extra.name
                );
            }
        }
    }
}
