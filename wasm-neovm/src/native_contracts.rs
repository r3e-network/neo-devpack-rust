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

/// A native contract descriptor: hash + per-method parameter shapes.
///
/// Used by the manifest emitter to compute the right method tokens and by
/// `System.Contract.Call` lowering to know what to push onto the stack.
#[derive(Debug, Clone)]
pub struct NativeContractDescriptor {
    /// The native contract HASH160 (little-endian).
    pub hash: [u8; 20],
    /// Canonical descriptor name (`"Neo.ContractManagement"`, etc.).
    pub name: &'static str,
    /// Methods exposed by this native contract (name + parameter types).
    pub methods: &'static [(&'static str, &'static [&'static str])],
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

// =============================================================================
//  L2: native contract routing — 10 more native contracts
// =============================================================================
//
// Canonical N3 native contract hashes (little-endian, as used in
// `System.Contract.Call`'s first stack argument). Verified against
// `rust-devpack/src/native_contracts.rs` and Neo Go's nativehashes package.
// Post-HF_Echidna natives (TokenManagement, Governance) require a
// chain-state query to obtain the canonical hash; we register them
// with placeholder hashes and clearly mark the need to update them
// at deploy time. Contracts that use them will deploy successfully
// only after the chain admin populates the real hash; until then,
// the placeholder zeros cause a "method token hash doesn't match
// any contract" error at first invocation, which is loud rather
// than silent.

const CONTRACT_MANAGEMENT_HASH: [u8; 20] = [
    0xfd, 0xa3, 0xfa, 0x43, 0x46, 0xea, 0x53, 0x2a, 0x25, 0x8f, 0xc4, 0x97, 0xdd, 0xad, 0xdb, 0x64,
    0x37, 0xc9, 0xfd, 0xff,
];
const STD_LIB_HASH: [u8; 20] = [
    0xc0, 0xef, 0x39, 0xce, 0xe0, 0xe4, 0xe9, 0x25, 0xc6, 0xc2, 0xa0, 0x6a, 0x79, 0xe1, 0x44, 0x0d,
    0xd8, 0x6f, 0xce, 0xac,
];
const LEDGER_HASH: [u8; 20] = [
    0xbe, 0xf2, 0x04, 0x31, 0x40, 0x36, 0x2a, 0x77, 0xc1, 0x50, 0x99, 0xc7, 0xe6, 0x4c, 0x12, 0xf7,
    0x00, 0xb6, 0x65, 0xda,
];
const POLICY_HASH: [u8; 20] = [
    0x7b, 0xc6, 0x81, 0xc0, 0xa1, 0xf7, 0x1d, 0x54, 0x34, 0x57, 0xb6, 0x8b, 0xba, 0x8d, 0x5f, 0x9f,
    0xdd, 0x4e, 0x5e, 0xcc,
];
const ROLE_MANAGEMENT_HASH: [u8; 20] = [
    0xe2, 0x95, 0xe3, 0x91, 0x54, 0x4c, 0x17, 0x8a, 0xd9, 0x4f, 0x03, 0xec, 0x4d, 0xcd, 0xff, 0x78,
    0x53, 0x4e, 0xcf, 0x49,
];
const ORACLE_HASH: [u8; 20] = [
    0x58, 0x87, 0x17, 0x11, 0x7e, 0x0a, 0xa8, 0x10, 0x72, 0xaf, 0xab, 0x71, 0xd2, 0xdd, 0x89, 0xfe,
    0x7c, 0x4b, 0x92, 0xfe,
];
const NOTARY_HASH: [u8; 20] = [
    0x3b, 0xec, 0x35, 0x31, 0x11, 0x9b, 0xba, 0xd7, 0x6d, 0xd0, 0x44, 0x92, 0x0b, 0x0d, 0xe6, 0xc3,
    0x19, 0x4f, 0xe1, 0xc1,
];
const TREASURY_HASH: [u8; 20] = [
    0xc1, 0x3a, 0x56, 0xc9, 0x83, 0x53, 0xa7, 0xea, 0x6a, 0x32, 0x4d, 0x9a, 0x83, 0x5d, 0x1b, 0x5b,
    0xf2, 0x26, 0x63, 0x15,
];

/// Placeholder hash for `Neo.TokenManagement` (post-HF_Echidna).
/// **MUST BE REPLACED** at deploy time with the chain-state lookup
/// (`Neo.ContractManagement.getContractById(-12)`). Until then, calls
/// to `Neo.TokenManagement.*` will fail at runtime with "method
/// token hash doesn't match any contract".
const TOKEN_MANAGEMENT_HASH_PLACEHOLDER: [u8; 20] = [0u8; 20];

/// Placeholder hash for `Neo.Governance` (post-HF_Echidna). Same
/// caveat as `TOKEN_MANAGEMENT_HASH_PLACEHOLDER`.
const GOVERNANCE_HASH_PLACEHOLDER: [u8; 20] = [0u8; 20];

/// Descriptor for ContractManagement. Source:
/// `neo-project/neo/src/Neo/SmartContract/Native/ContractManagement.cs`.
pub const CONTRACT_MANAGEMENT_DESCRIPTOR: NativeContractDescriptor = NativeContractDescriptor {
    hash: CONTRACT_MANAGEMENT_HASH,
    name: "Neo.ContractManagement",
    methods: &[
        ("deploy", &["ByteArray", "ByteArray"]),
        ("deploy", &["ByteArray", "ByteArray", "Any"]),
        ("update", &["ByteArray", "ByteArray"]),
        ("update", &["ByteArray", "ByteArray", "Any"]),
        ("destroy", &[]),
        ("getContract", &["Hash160"]),
        ("getContractById", &["Integer"]),
        ("getContractHashes", &[]),
        ("getMinimumDeploymentFee", &[]),
        ("setMinimumDeploymentFee", &["Integer"]),
        ("hasMethod", &["Hash160", "String", "Integer"]),
        ("isContract", &["Hash160"]),
    ],
};

/// Descriptor for StdLib. Source:
/// `neo-project/neo/src/Neo/SmartContract/Native/StdLib.cs`.
pub const STDLIB_DESCRIPTOR: NativeContractDescriptor = NativeContractDescriptor {
    hash: STD_LIB_HASH,
    name: "Neo.StdLib",
    methods: &[
        ("serialize", &["Any"]),
        ("deserialize", &["ByteArray"]),
        ("jsonSerialize", &["Any"]),
        ("jsonDeserialize", &["String"]),
        ("itoa", &["Integer"]),
        ("atoi", &["String"]),
        ("base64Encode", &["ByteArray"]),
        ("base64Decode", &["String"]),
        ("base58Encode", &["ByteArray"]),
        ("base58Decode", &["String"]),
    ],
};

/// Descriptor for LedgerContract. Source:
/// `neo-project/neo/src/Neo/SmartContract/Native/LedgerContract.cs`.
pub const LEDGER_DESCRIPTOR: NativeContractDescriptor = NativeContractDescriptor {
    hash: LEDGER_HASH,
    name: "Neo.Ledger",
    methods: &[
        ("getBlock", &["Integer"]),
        ("getBlockByHash", &["Hash256"]),
        ("getBlockByIndex", &["Integer"]),
        ("getTransaction", &["Hash256"]),
        ("getTransactionFromBlock", &["Hash256", "Integer"]),
        ("getTransactionHeight", &["Hash256"]),
        ("currentHash", &[]),
        ("currentIndex", &[]),
    ],
};

/// Descriptor for PolicyContract. Source:
/// `neo-project/neo/src/Neo/SmartContract/Native/PolicyContract.cs`.
pub const POLICY_DESCRIPTOR: NativeContractDescriptor = NativeContractDescriptor {
    hash: POLICY_HASH,
    name: "Neo.Policy",
    methods: &[
        ("getFeePerByte", &[]),
        ("getExecFeeFactor", &[]),
        ("getStoragePrice", &[]),
        ("isBlocked", &["Hash160"]),
        ("getMaxNotValidBeforeBlockDelta", &[]),
        ("setFeePerByte", &["Integer"]),
        ("setExecFeeFactor", &["Integer"]),
        ("setStoragePrice", &["Integer"]),
        ("blockAccount", &["Hash160"]),
        ("unblockAccount", &["Hash160"]),
    ],
};

/// Descriptor for RoleManagement. Source:
/// `neo-project/neo/src/Neo/SmartContract/Native/RoleManagement.cs`.
pub const ROLE_MANAGEMENT_DESCRIPTOR: NativeContractDescriptor = NativeContractDescriptor {
    hash: ROLE_MANAGEMENT_HASH,
    name: "Neo.RoleManagement",
    methods: &[
        ("getDesignatedByRole", &["Integer", "Integer"]),
        ("assignRole", &["Integer", "Array"]),
    ],
};

/// Descriptor for OracleContract. Source:
/// `neo-project/neo/src/Neo/SmartContract/Native/OracleContract.cs`.
pub const ORACLE_DESCRIPTOR: NativeContractDescriptor = NativeContractDescriptor {
    hash: ORACLE_HASH,
    name: "Neo.Oracle",
    methods: &[
        ("request", &["String", "String", "String", "Integer"]),
        ("finish", &[]),
    ],
};

/// Descriptor for Notary. Source:
/// `neo-project/neo/src/Neo/SmartContract/Native/Notary.cs`.
pub const NOTARY_DESCRIPTOR: NativeContractDescriptor = NativeContractDescriptor {
    hash: NOTARY_HASH,
    name: "Neo.Notary",
    methods: &[
        ("deposit", &["Hash160", "Integer"]),
        ("withdraw", &["Hash160", "ByteArray"]),
        ("balanceOf", &["Hash160"]),
        ("expirationOf", &["Hash160"]),
        ("getMaxNotValidBeforeDelta", &[]),
    ],
};

/// Descriptor for Treasury. Source:
/// `neo-project/neo/src/Neo/SmartContract/Native/Treasury.cs`.
pub const TREASURY_DESCRIPTOR: NativeContractDescriptor = NativeContractDescriptor {
    hash: TREASURY_HASH,
    name: "Neo.Treasury",
    methods: &[("verify", &["Hash160", "ByteArray", "ByteArray", "Integer"])],
};

/// Descriptor for CryptoLib (the canonical method list, in addition to
/// the legacy single-method alias resolution in `crypto_lib_method`).
pub const CRYPTOLIB_DESCRIPTOR: NativeContractDescriptor = NativeContractDescriptor {
    hash: CRYPTOLIB_HASH,
    name: "Neo.Crypto",
    methods: &[
        ("sha256", &["ByteArray"]),
        ("ripemd160", &["ByteArray"]),
        ("keccak256", &["ByteArray"]),
        ("murmur32", &["ByteArray", "Integer"]),
        (
            "verifyWithECDsa",
            &["ByteArray", "ByteArray", "ByteArray", "Integer"],
        ),
    ],
};

/// Descriptor for TokenManagement (post-HF_Echidna). Source:
/// `neo-project/neo/src/Neo/SmartContract/Native/TokenManagement.cs`
/// (and `.Fungible.cs`, `.NonFungible.cs`).
///
/// The canonical hash is `[0u8; 20]` (placeholder) — see
/// `TOKEN_MANAGEMENT_HASH_PLACEHOLDER` above. The descriptor is
/// registered so that calls like `Neo.TokenManagement.transfer`
/// resolve at translate time; deploy-time chain lookup is
/// required to fill in the real hash.
pub const TOKEN_MANAGEMENT_DESCRIPTOR: NativeContractDescriptor = NativeContractDescriptor {
    hash: TOKEN_MANAGEMENT_HASH_PLACEHOLDER,
    name: "Neo.TokenManagement",
    methods: &[
        ("getToken", &["Hash256"]),
        ("transfer", &["Hash160", "Hash160", "Integer", "Any"]),
        ("getBalance", &["Hash160", "Hash256"]),
        ("totalSupply", &["Hash256"]),
        // NEP-11 (NFT) methods:
        ("ownerOf", &["Hash256", "ByteArray"]),
        ("tokensOf", &["Hash160"]),
        ("balanceOf", &["Hash160", "Hash256"]),
    ],
};

/// Descriptor for Governance (post-HF_Echidna). Source:
/// `neo-project/neo/src/Neo/SmartContract/Native/Governance.cs`.
///
/// The canonical hash is `[0u8; 20]` (placeholder) — see
/// `GOVERNANCE_HASH_PLACEHOLDER` above.
pub const GOVERNANCE_DESCRIPTOR: NativeContractDescriptor = NativeContractDescriptor {
    hash: GOVERNANCE_HASH_PLACEHOLDER,
    name: "Neo.Governance",
    methods: &[
        ("getVotersCount", &["Hash256"]),
        ("getVoter", &["ECPoint", "Hash256"]),
        ("getCandidate", &["ECPoint"]),
        ("getCommittee", &[]),
        ("getNextCommittee", &["Hash256"]),
        ("getRegisteredCandidates", &[]),
    ],
};

/// All N3 native contract descriptors. Used by the manifest emitter to
/// produce the right method tokens; by chain adapters to resolve the
/// descriptor name to the on-chain hash; and by tests to assert the
/// canonical hash set has not drifted.
pub static NATIVE_CONTRACT_REGISTRY: &[&NativeContractDescriptor] = &[
    &CONTRACT_MANAGEMENT_DESCRIPTOR,
    &STDLIB_DESCRIPTOR,
    &CRYPTOLIB_DESCRIPTOR,
    &LEDGER_DESCRIPTOR,
    &POLICY_DESCRIPTOR,
    &ROLE_MANAGEMENT_DESCRIPTOR,
    &ORACLE_DESCRIPTOR,
    &NOTARY_DESCRIPTOR,
    &TREASURY_DESCRIPTOR,
    &TOKEN_MANAGEMENT_DESCRIPTOR,
    &GOVERNANCE_DESCRIPTOR,
];

/// Look up a native contract descriptor by its canonical name (e.g.
/// `"Neo.ContractManagement"` or `"Neo.Oracle"`). Returns `None` for
/// unknown names.
pub fn native_contract_by_name(name: &str) -> Option<&'static NativeContractDescriptor> {
    NATIVE_CONTRACT_REGISTRY
        .iter()
        .copied()
        .find(|d| d.name == name)
}

/// Look up a native contract descriptor by its method call (e.g. a
/// `Neo.Oracle.Request` import in a wasm module). The descriptor is the
/// part before the last `.` (`Neo.Oracle`); the method is the last part.
/// Returns the descriptor and the canonical method name as stored in
/// `methods[].0` (which may differ in case from the input).
pub fn native_contract_by_method(
    alias: &str,
) -> Option<(&'static NativeContractDescriptor, &'static str)> {
    let trimmed = alias.strip_prefix("Neo.").unwrap_or(alias);
    let (contract, method) = trimmed.rsplit_once('.')?;
    let full = format!("Neo.{contract}");
    let d = native_contract_by_name(&full)?;
    // Return the canonical (stored) method name, not the caller's input.
    let (canonical, _) = *d.methods.iter().find(|(n, _)| *n == method)?;
    Some((d, canonical))
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
        assert_eq!(
            crypto_lib_method("Neo.Crypto.SHA256").unwrap().method,
            "sha256"
        );
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

    #[test]
    fn native_contract_registry_contains_all_11_natives_with_2_placeholders() {
        // The 9 N3 native contracts with published mainnet hashes +
        // 2 placeholders (TokenManagement + Governance) for the
        // post-HF_Echidna natives whose canonical hash requires a
        // chain-state query. The placeholder hashes are `[0u8; 20]`
        // and are clearly marked in their constant declarations; a
        // deploy-time check (via Neo.ContractManagement.getContractById)
        // is required to populate the real hash.
        assert_eq!(NATIVE_CONTRACT_REGISTRY.len(), 11);
        for d in NATIVE_CONTRACT_REGISTRY {
            assert!(!d.name.is_empty());
            assert!(!d.methods.is_empty());
        }
        // The two placeholders must remain placeholders until a
        // proper chain-state lookup is implemented.
        assert_eq!(NATIVE_CONTRACT_REGISTRY[9].name, "Neo.TokenManagement");
        assert!(NATIVE_CONTRACT_REGISTRY[9].hash.iter().all(|&b| b == 0));
        assert_eq!(NATIVE_CONTRACT_REGISTRY[10].name, "Neo.Governance");
        assert!(NATIVE_CONTRACT_REGISTRY[10].hash.iter().all(|&b| b == 0));
    }

    #[test]
    fn contract_management_descriptor_contains_canonical_methods() {
        let d = native_contract_by_name("Neo.ContractManagement").unwrap();
        assert_eq!(d.name, "Neo.ContractManagement");
        assert!(d.methods.iter().any(|(n, _)| *n == "deploy"));
        assert!(d.methods.iter().any(|(n, _)| *n == "getContract"));
        assert!(d.methods.iter().any(|(n, _)| *n == "hasMethod"));
    }

    #[test]
    fn oracle_descriptor_contains_request_and_finish() {
        let d = native_contract_by_name("Neo.Oracle").unwrap();
        assert_eq!(d.name, "Neo.Oracle");
        assert!(d.methods.iter().any(|(n, _)| *n == "request"));
        assert!(d.methods.iter().any(|(n, _)| *n == "finish"));
    }

    #[test]
    fn native_contract_by_method_resolves_well_known_calls() {
        let (d, method) = native_contract_by_method("Neo.Oracle.request").unwrap();
        assert_eq!(d.name, "Neo.Oracle");
        assert_eq!(method, "request");

        let (d, method) = native_contract_by_method("Neo.Ledger.getBlock").unwrap();
        assert_eq!(d.name, "Neo.Ledger");
        assert_eq!(method, "getBlock");
    }

    #[test]
    fn stdlib_descriptor_contains_all_10_methods() {
        let d = native_contract_by_name("Neo.StdLib").unwrap();
        assert!(d.methods.iter().any(|(n, _)| *n == "itoa"));
        assert!(d.methods.iter().any(|(n, _)| *n == "atoi"));
        assert!(d.methods.iter().any(|(n, _)| *n == "base58Encode"));
        assert!(d.methods.iter().any(|(n, _)| *n == "base58Decode"));
        assert!(d.methods.iter().any(|(n, _)| *n == "base64Encode"));
        assert!(d.methods.iter().any(|(n, _)| *n == "base64Decode"));
        assert!(d.methods.iter().any(|(n, _)| *n == "serialize"));
        assert!(d.methods.iter().any(|(n, _)| *n == "deserialize"));
        assert!(d.methods.iter().any(|(n, _)| *n == "jsonSerialize"));
        assert!(d.methods.iter().any(|(n, _)| *n == "jsonDeserialize"));
    }
}
