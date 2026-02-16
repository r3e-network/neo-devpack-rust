mod generated {
    include!(concat!(env!("OUT_DIR"), "/syscalls.rs"));
}

pub use generated::SyscallInfo;

/// Extended syscall info for native contract methods not in the generated table
///
/// NOTE: These hashes are calculated as the first four bytes (little-endian)
/// of SHA256("SysCallName"), matching Neo N3 syscall hashing.
/// Use the correct Neo N3 syscall hashes from the official specification.
static EXTENDED_SYSCALLS: &[SyscallInfo] = &[
    // Neo.Crypto native contract methods
    // Reference: https://docs.neo.org/docs/en-us/reference/scapi/native.html
    SyscallInfo {
        name: "Neo.Crypto.SHA256",
        hash: 0x1174acd7,
    },
    SyscallInfo {
        name: "Neo.Crypto.RIPEMD160",
        hash: 0xd2d6d126,
    },
    SyscallInfo {
        name: "Neo.Crypto.Murmur32",
        hash: 0x8738a9dc,
    },
    SyscallInfo {
        name: "Neo.Crypto.Keccak256",
        hash: 0xe021b1dc,
    },
    SyscallInfo {
        name: "Neo.Crypto.Hash160",
        hash: 0xac67b840,
    },
    SyscallInfo {
        name: "Neo.Crypto.Hash256",
        hash: 0xd94bd85c,
    },
    SyscallInfo {
        name: "Neo.Crypto.VerifyWithECDsa",
        hash: 0xcf822a6a,
    },
    // Note: CheckSig and CheckMultisig are in the generated table with correct hashes
    // - System.Crypto.CheckSig: 0x27B3E756
    // - System.Crypto.CheckMultisig: 0x3ADCD09E
    // Do NOT add duplicate entries with incorrect hashes here.
];

pub fn all() -> &'static [SyscallInfo] {
    generated::SYSCALLS
}

pub fn extended() -> &'static [SyscallInfo] {
    EXTENDED_SYSCALLS
}

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
