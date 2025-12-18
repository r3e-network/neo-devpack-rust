mod generated {
    include!(concat!(env!("OUT_DIR"), "/syscalls.rs"));
}

pub use generated::SyscallInfo;

/// Extended syscall info for native contract methods not in the generated table
static EXTENDED_SYSCALLS: &[SyscallInfo] = &[
    // Neo.Crypto native contract methods
    SyscallInfo {
        name: "Neo.Crypto.SHA256",
        hash: 0x906e6e88,
    },
    SyscallInfo {
        name: "Neo.Crypto.RIPEMD160",
        hash: 0x8a85a0a4,
    },
    SyscallInfo {
        name: "Neo.Crypto.Murmur32",
        hash: 0x76259782,
    },
    SyscallInfo {
        name: "Neo.Crypto.Keccak256",
        hash: 0x5c8ecd46,
    },
    SyscallInfo {
        name: "Neo.Crypto.Hash160",
        hash: 0x57c6b646,
    },
    SyscallInfo {
        name: "Neo.Crypto.Hash256",
        hash: 0x3ced0552,
    },
    SyscallInfo {
        name: "Neo.Crypto.VerifyWithECDsa",
        hash: 0x40746983,
    },
    SyscallInfo {
        name: "Neo.Crypto.CheckSig",
        hash: 0xbb359497,
    },
    SyscallInfo {
        name: "Neo.Crypto.CheckMultisig",
        hash: 0xb8645d5f,
    },
];

pub fn all() -> &'static [SyscallInfo] {
    generated::SYSCALLS
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
    generated::SYSCALLS.iter().find(|info| info.hash == hash)
}
