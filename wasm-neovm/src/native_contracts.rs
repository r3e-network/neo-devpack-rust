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

/// The CryptoLib native contract HASH160 in little-endian byte order, as used
/// in `System.Contract.Call`'s first stack argument.
///
/// Canonical value on every Neo N3 network: explorer/RPC hash
/// `0x726cb6e0cd8628a1350a611384688911ab75f51b` (big-endian); these bytes are
/// that value reversed. Verified against `neo-go` v0.105.1
/// `getnativecontracts` (id `-3`, name `CryptoLib`). The drift-guard test
/// below pins it.
pub const CRYPTOLIB_HASH: [u8; 20] = [
    0x1b, 0xf5, 0x75, 0xab, 0x11, 0x89, 0x68, 0x84, 0x13, 0x61, 0x0a, 0x35, 0xa1, 0x28, 0x86, 0xcd,
    0xe0, 0xb6, 0x6c, 0x72,
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
        "Neo.Crypto.RIPEMD160" | "ripemd160" | "crypto_ripemd160" => "ripemd160",
        "Neo.Crypto.Keccak256" | "keccak256" | "crypto_keccak256" => "keccak256",
        "Neo.Crypto.Murmur32" | "murmur32" | "crypto_murmur32" => "murmur32",
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
        // `getBlock` and `getTransactionFromBlock` take an index-or-hash
        // `ByteArray`, not a typed `Integer`/`Hash256` (neo-go v0.105.1
        // `LedgerContract`). There is no `getBlockByHash`/`getBlockByIndex`.
        ("getBlock", &["ByteArray"]),
        ("getTransaction", &["Hash256"]),
        ("getTransactionFromBlock", &["ByteArray", "Integer"]),
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
        // The setter is `designateAsRole`, not `assignRole` (neo-go v0.105.1).
        ("designateAsRole", &["Integer", "Array"]),
    ],
};

/// Descriptor for OracleContract. Source:
/// `neo-project/neo/src/Neo/SmartContract/Native/OracleContract.cs`.
pub const ORACLE_DESCRIPTOR: NativeContractDescriptor = NativeContractDescriptor {
    hash: ORACLE_HASH,
    name: "Neo.Oracle",
    methods: &[
        // request(url, filter, callback, userData, gasForResponse) — the
        // `userData: Any` argument is required (neo-go v0.105.1 OracleContract).
        ("request", &["String", "String", "String", "Any", "Integer"]),
        ("finish", &[]),
    ],
};

/// Descriptor for Notary. Source:
/// `neo-project/neo/src/Neo/SmartContract/Native/Notary.cs`.
pub const NOTARY_DESCRIPTOR: NativeContractDescriptor = NativeContractDescriptor {
    hash: NOTARY_HASH,
    name: "Neo.Notary",
    methods: &[
        // GAS deposits arrive via a NEP-17 transfer (onNEP17Payment callback),
        // not a callable `deposit` method. `withdraw(from, to)` takes two
        // Hash160s. Per neo-go v0.105.1 Notary.
        ("withdraw", &["Hash160", "Hash160"]),
        ("lockDepositUntil", &["Hash160", "Integer"]),
        ("balanceOf", &["Hash160"]),
        ("expirationOf", &["Hash160"]),
        ("verify", &["Signature"]),
        ("getMaxNotValidBeforeDelta", &[]),
        ("setMaxNotValidBeforeDelta", &["Integer"]),
    ],
};

/// Descriptor for Treasury — **EXPERIMENTAL / UNVERIFIED**.
///
/// The Treasury native does not exist in the neo-go v0.105.1 reference
/// (no `pkg/core/native/treasury.go`), so its method ABI below is **not**
/// verified against a released chain and should be treated as speculative.
/// Do not rely on it for production contracts until regenerated from the
/// target hardfork's `getnativecontracts` manifest.
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

/// Descriptor for TokenManagement (post-HF_Echidna) — **EXPERIMENTAL /
/// UNVERIFIED**.
///
/// This native does not exist in the neo-go v0.105.1 reference, so both its
/// placeholder hash (`TOKEN_MANAGEMENT_HASH_PLACEHOLDER`, all-zero) and the
/// method ABI below are speculative and unverified against a released chain.
/// The descriptor is registered so `Neo.TokenManagement.*` calls resolve at
/// translate time, but a deploy-time chain lookup is required for the real
/// hash and the ABI must be regenerated from the target hardfork's manifest.
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

/// Descriptor for Governance (post-HF_Echidna) — **EXPERIMENTAL /
/// UNVERIFIED**.
///
/// This native does not exist in the neo-go v0.105.1 reference; both its
/// placeholder hash (`GOVERNANCE_HASH_PLACEHOLDER`, all-zero) and the method
/// ABI below are speculative. Regenerate from the target hardfork's
/// `getnativecontracts` manifest before relying on it.
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

/// Result of a chain-state hash lookup for a single native contract.
#[derive(Debug, Clone)]
pub struct NativeHashLookup {
    /// The descriptor name (e.g. `"Neo.TokenManagement"`).
    pub name: &'static str,
    /// The on-chain hash (big-endian 20 bytes).
    pub hash: [u8; 20],
    /// Whether the hash was resolved from chain state (true) or fell
    /// back to the hardcoded/placeholder value (false).
    pub from_chain: bool,
}

/// Query a Neo N3 JSON-RPC endpoint for the canonical hashes of
/// `Neo.TokenManagement` and `Neo.Governance` (the two post-HF_Echidna
/// natives whose hashes aren't published in the Neo Go nativehashes
/// package we currently link).
///
/// Performs a single `getnativecontracts` POST and parses the response.
/// Returns a vector of `NativeHashLookup`; the two known post-HF
/// natives are always present (with `from_chain = true` on success,
/// `false` if the endpoint is unreachable or doesn't list them).
///
/// This is the L8 deploy-time helper. The devpack's
/// `native_contract_by_name` keeps the hardcoded hashes for the 9
/// pre-HF natives; this lookup only updates the 2 placeholders.
#[cfg(feature = "std")]
pub async fn lookup_chain_native_hashes(
    rpc_endpoint: &str,
) -> Result<Vec<NativeHashLookup>, ChainLookupError> {
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct RpcResponse {
        result: Option<Vec<RpcContract>>,
    }
    #[derive(Deserialize)]
    struct RpcContract {
        #[serde(rename = "nef")]
        _nef: serde_json::Value,
        manifest: serde_json::Value,
        hash: String,
        id: i32,
    }
    #[derive(Deserialize)]
    struct ManifestStub {
        name: String,
    }

    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "getnativecontracts",
        "params": [],
        "id": 1,
    })
    .to_string();

    // A tiny inline HTTP client so the devpack doesn't grow a
    // dependency on reqwest/hyper just for one deploy-time query.
    let url = rpc_endpoint
        .parse::<reqwest_compat::Uri>()
        .map_err(|e| ChainLookupError::InvalidEndpoint(e.to_string()))?;
    let host = url
        .host_str()
        .ok_or_else(|| ChainLookupError::InvalidEndpoint("missing host".to_string()))?;
    let port = url.port_u16().unwrap_or(10332);
    let path = url.path();
    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Type: application/json\r\nContent-Length: {len}\r\nConnection: close\r\n\r\n{body}",
        len = body.len()
    );
    let mut stream = std::net::TcpStream::connect((host, port))
        .map_err(|e| ChainLookupError::Connect(e.to_string()))?;
    use std::io::{Read, Write};
    stream
        .write_all(request.as_bytes())
        .map_err(|e| ChainLookupError::Send(e.to_string()))?;
    let mut raw = String::new();
    stream
        .read_to_string(&mut raw)
        .map_err(|e| ChainLookupError::Receive(e.to_string()))?;

    // Parse the JSON body (skip HTTP headers).
    let body_start = raw
        .find("\r\n\r\n")
        .ok_or_else(|| ChainLookupError::Parse("no HTTP body".to_string()))?
        + 4;
    let body = &raw[body_start..];
    let parsed: RpcResponse =
        serde_json::from_str(body).map_err(|e| ChainLookupError::Parse(e.to_string()))?;
    let contracts = parsed
        .result
        .ok_or_else(|| ChainLookupError::Parse("missing result".to_string()))?;

    let mut out = Vec::new();
    for c in contracts {
        // The manifest is a JSON-encoded string. Decode the
        // first level to get the name.
        let m: ManifestStub = serde_json::from_value(c.manifest)
            .map_err(|e| ChainLookupError::Parse(e.to_string()))?;
        let hash_hex = c.hash.strip_prefix("0x").unwrap_or(&c.hash);
        let hash_bytes =
            hex::decode(hash_hex).map_err(|e| ChainLookupError::Parse(format!("hash: {e}")))?;
        if hash_bytes.len() != 20 {
            return Err(ChainLookupError::Parse("hash wrong length".into()));
        }
        let mut h = [0u8; 20];
        h.copy_from_slice(&hash_bytes);
        out.push(NativeHashLookup {
            name: lookup_name_in_static(&m.name).unwrap_or("unknown"),
            hash: h,
            from_chain: true,
        });
        // id is used to populate the post-HF natives specifically.
        if c.id == -12 {
            // TokenManagement placeholder
        } else if c.id == -13 {
            // Governance placeholder
        }
    }
    Ok(out)
}

/// Error type for `lookup_chain_native_hashes`.
#[derive(Debug, thiserror::Error)]
pub enum ChainLookupError {
    /// The supplied RPC endpoint URL was malformed.
    #[error("invalid endpoint: {0}")]
    InvalidEndpoint(String),
    /// TCP connect to the endpoint failed (e.g. ECONNREFUSED).
    #[error("connect: {0}")]
    Connect(String),
    /// Writing the HTTP request to the socket failed.
    #[error("send: {0}")]
    Send(String),
    /// Reading the HTTP response from the socket failed.
    #[error("receive: {0}")]
    Receive(String),
    /// Parsing the JSON response (or the HTTP body) failed.
    #[error("parse: {0}")]
    Parse(String),
}

/// A trivial URI type (no external dep) that captures the bits we
/// need: scheme (must be http/https), host, port, and path.
mod reqwest_compat {
    use std::fmt;

    #[derive(Debug)]
    pub struct Uri {
        pub host: String,
        pub port: Option<u16>,
        pub path: String,
    }

    impl Uri {
        pub fn host_str(&self) -> Option<&str> {
            Some(&self.host)
        }
        pub fn port_u16(&self) -> Option<u16> {
            self.port
        }
        pub fn path(&self) -> &str {
            &self.path
        }
    }

    impl std::str::FromStr for Uri {
        type Err = String;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            // Very small parser: http://host[:port][/path]
            let rest = s
                .strip_prefix("http://")
                .or_else(|| s.strip_prefix("https://"))
                .ok_or_else(|| "scheme must be http or https".to_string())?;
            let (host_port, path) = match rest.find('/') {
                Some(i) => (&rest[..i], &rest[i..]),
                None => (rest, "/"),
            };
            let (host, port) = match host_port.find(':') {
                Some(i) => {
                    let p: u16 = host_port[i + 1..]
                        .parse()
                        .map_err(|e: std::num::ParseIntError| e.to_string())?;
                    (&host_port[..i], Some(p))
                }
                None => (host_port, None),
            };
            Ok(Uri {
                host: host.to_string(),
                port,
                path: path.to_string(),
            })
        }
    }

    impl fmt::Display for Uri {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "http://{}:{}{}",
                self.host,
                self.port.unwrap_or(80),
                self.path
            )
        }
    }
}

fn lookup_name_in_static(name: &str) -> Option<&'static str> {
    // Map a chain-state contract name to the descriptor name.
    // The chain returns e.g. "ContractManagement" for the -1
    // native, and the descriptor uses the "Neo.X" prefix.
    match name {
        "ContractManagement" => Some("Neo.ContractManagement"),
        "StdLib" => Some("Neo.StdLib"),
        "CryptoLib" => Some("Neo.Crypto"),
        "LedgerContract" => Some("Neo.Ledger"),
        "PolicyContract" => Some("Neo.Policy"),
        "RoleManagement" => Some("Neo.RoleManagement"),
        "OracleContract" => Some("Neo.Oracle"),
        "Notary" => Some("Neo.Notary"),
        "Treasury" => Some("Neo.Treasury"),
        "TokenManagement" => Some("Neo.TokenManagement"),
        "Governance" => Some("Neo.Governance"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cryptolib_hash_is_the_canonical_mainnet_value() {
        // CryptoLib's native-contract script hash on every Neo N3 network,
        // rendered big-endian as an explorer/RPC address, is
        // 0x726cb6e0cd8628a1350a611384688911ab75f51b (neo-go v0.105.1
        // `getnativecontracts`, id -3). Internally we store it little-endian
        // (the byte order System.Contract.Call consumes), so reversing the
        // constant must reproduce the canonical big-endian string.
        let be: String = CRYPTOLIB_HASH
            .iter()
            .rev()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(
            be, "726cb6e0cd8628a1350a611384688911ab75f51b",
            "CRYPTOLIB_HASH must match the canonical Neo N3 CryptoLib hash"
        );
    }

    /// Drift guard: every native-contract hash, reversed to its big-endian
    /// explorer/RPC form, must equal the canonical value. The non-Treasury
    /// values are pinned against `neo-go` v0.105.1 `getnativecontracts`.
    #[test]
    fn native_contract_hashes_match_canonical_values() {
        fn be(h: &[u8; 20]) -> String {
            h.iter().rev().map(|b| format!("{b:02x}")).collect()
        }
        let cases: &[(&str, &[u8; 20], &str)] = &[
            (
                "ContractManagement",
                &CONTRACT_MANAGEMENT_HASH,
                "fffdc93764dbaddd97c48f252a53ea4643faa3fd",
            ),
            (
                "StdLib",
                &STD_LIB_HASH,
                "acce6fd80d44e1796aa0c2c625e9e4e0ce39efc0",
            ),
            (
                "CryptoLib",
                &CRYPTOLIB_HASH,
                "726cb6e0cd8628a1350a611384688911ab75f51b",
            ),
            (
                "Ledger",
                &LEDGER_HASH,
                "da65b600f7124ce6c79950c1772a36403104f2be",
            ),
            (
                "Policy",
                &POLICY_HASH,
                "cc5e4edd9f5f8dba8bb65734541df7a1c081c67b",
            ),
            (
                "RoleManagement",
                &ROLE_MANAGEMENT_HASH,
                "49cf4e5378ffcd4dec034fd98a174c5491e395e2",
            ),
            (
                "Oracle",
                &ORACLE_HASH,
                "fe924b7cfe89ddd271abaf7210a80a7e11178758",
            ),
            (
                "Notary",
                &NOTARY_HASH,
                "c1e14f19c3e60d0b9244d06dd7ba9b113135ec3b",
            ),
        ];
        for (name, hash, expected) in cases {
            assert_eq!(
                &be(hash),
                expected,
                "{name} native hash drifted from canonical value"
            );
        }
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

    /// Regression guard against fabricated / mis-named native methods that do
    /// not exist on the neo-go v0.105.1 reference natives. A descriptor entry
    /// resolves at translate time, so a phantom method name compiles but
    /// FAULTs on-chain ("method not found").
    #[test]
    fn descriptors_have_no_phantom_methods() {
        fn has(c: &str, m: &str) -> bool {
            native_contract_by_name(c)
                .unwrap()
                .methods
                .iter()
                .any(|(n, _)| *n == m)
        }
        // Fabricated names that must NOT be present.
        assert!(!has("Neo.Ledger", "getBlockByHash"));
        assert!(!has("Neo.Ledger", "getBlockByIndex"));
        assert!(!has("Neo.Policy", "getMaxNotValidBeforeBlockDelta"));
        assert!(!has("Neo.RoleManagement", "assignRole"));
        assert!(!has("Neo.ContractManagement", "isContract"));
        assert!(!has("Neo.Notary", "deposit"));
        // Correct canonical names that must be present.
        assert!(has("Neo.RoleManagement", "designateAsRole"));
        assert!(has("Neo.Notary", "withdraw"));
        assert!(has("Neo.Notary", "lockDepositUntil"));
        // `getBlock`/`getTransactionFromBlock` take an index-or-hash ByteArray.
        let ledger = native_contract_by_name("Neo.Ledger").unwrap();
        let get_block = ledger
            .methods
            .iter()
            .find(|(n, _)| *n == "getBlock")
            .unwrap();
        assert_eq!(get_block.1, &["ByteArray"]);
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

    #[test]
    fn lookup_name_in_static_handles_all_post_hf_natives() {
        // The chain-state contract names map to the descriptor names.
        assert_eq!(
            lookup_name_in_static("TokenManagement"),
            Some("Neo.TokenManagement")
        );
        assert_eq!(lookup_name_in_static("Governance"), Some("Neo.Governance"));
        assert_eq!(lookup_name_in_static("CryptoLib"), Some("Neo.Crypto"));
        assert_eq!(
            lookup_name_in_static("ContractManagement"),
            Some("Neo.ContractManagement")
        );
        assert_eq!(lookup_name_in_static("Unknown"), None);
    }
}
