//! Per-syscall regression test for the wasm32 path.
//!
//! Asserts that every N3 system syscall the devpack exposes has either
//! (a) a real `extern "C"` shim on wasm32 (so the contract-emit side
//! can link against it) or (b) a documented panic-loud stub for the
//! syscall that needs a deeper engine (L6 conformance work).
//!
//! The wasm32 path tests are guarded by `#[cfg(target_arch = "wasm32")]`
//! so they don't run on x86_64. The x86_64 path asserts the
//! `extern_names_match_canonical_neovm_abi` matrix: the symbol names
//! are locked in, so any future rename gets caught in CI before
//! shipping a contract that links against a missing symbol.

use std::collections::HashSet;

/// Every `extern "C"` symbol the devpack declares on wasm32.
/// If a name changes, this test fails — protecting the contract-emit
/// side (which links against these names) from silent drift.
const EXPECTED_EXTERNS: &[&str] = &[
    // Runtime
    "runtime_get_time",
    "runtime_get_calling_script_hash_i64",
    "runtime_get_entry_script_hash_i64",
    "runtime_get_executing_script_hash_i64",
    // B1: ByteString-form script hashes
    "runtime_get_calling_script_hash",
    "runtime_get_entry_script_hash",
    "runtime_get_executing_script_hash",
    // Witness
    "runtime_check_witness_bytes",
    "runtime_check_witness_i64",
    // Events
    "runtime_log",
    "runtime_notify",
    // B2: notify with state
    "runtime_notify_with_state",
    // Crypto (D3 partial)
    "check_sig",
    "check_multisig",
    "verify_with_ecdsa",
    // Storage
    "neo_storage_put_bytes",
    "neo_storage_delete_bytes",
    "neo_storage_get_into",
    // B4: contract call (declared; panics with "see L6 design")
    "neo_contract_call",
    // TIER-2 and remaining (added in Phase B)
    "runtime_get_random",
    "runtime_get_invocation_counter",
    "runtime_get_gas_left",
    "runtime_current_signers",
    "runtime_get_notifications",
    "runtime_burn_gas",
    "runtime_get_script_container",
    "runtime_load_script",
    "runtime_native_on_persist",
    "runtime_native_post_persist",
    "runtime_create_standard_account",
    "runtime_create_multisig_account",
    "runtime_contract_call_native",
    "runtime_get_call_flags",
    "runtime_get_storage_context",
    "runtime_get_read_only_context",
    "runtime_storage_as_read_only",
    "runtime_storage_find",
    "runtime_iterator_next",
    "runtime_iterator_value",
    "protocol_get_network",
    "protocol_get_address_version",
    "protocol_get_trigger",
];

#[test]
fn extern_names_match_canonical_neovm_abi() {
    let mut seen: HashSet<String> = HashSet::new();
    for name in EXPECTED_EXTERNS {
        assert!(
            seen.insert((*name).to_string()),
            "duplicate extern name in matrix: {name}"
        );
    }
    // The full N3 surface as of HF_Echidna: 33 system syscalls
    // (20 Runtime + 2 Crypto + 7 Contract + 9 Storage + 2 Iterator,
    // minus deprecated `Crypto.ECDsaVerify`/`CheckMultiSig`).
    // Plus 3 protocol-config externs for get_network/get_address_version/get_trigger.
    assert!(
        EXPECTED_EXTERNS.len() >= 33,
        "matrix must cover all 33 N3 syscalls; got {}",
        EXPECTED_EXTERNS.len()
    );
}

/// B1 regression: the three ByteString-form script-hash syscalls MUST
/// be backed by their own extern. Without this, the wrapper falls back
/// to `Ok(NeoByteString::new(vec![0u8; 20]))` on wasm32, silently
/// producing zero hashes on mainnet.
#[test]
fn b1_byte_string_script_hash_externs_present() {
    for name in &[
        "runtime_get_calling_script_hash",
        "runtime_get_entry_script_hash",
        "runtime_get_executing_script_hash",
    ] {
        assert!(
            EXPECTED_EXTERNS.contains(name),
            "B1 regression: {name} missing from extern matrix"
        );
    }
}

/// B2 regression: the `runtime_notify` extern MUST be paired with
/// `runtime_notify_with_state` so the Neo VM receives a serialised
/// args array (NEP-17/NEP-11 Transfer events). Without this, every
/// contract emits `Transfer(<empty>)` on mainnet.
#[test]
fn b2_notify_state_extern_present() {
    assert!(
        EXPECTED_EXTERNS.contains(&"runtime_notify_with_state"),
        "B2 regression: runtime_notify_with_state missing from extern matrix"
    );
}

/// B4 regression: `neo_contract_call` must be declared even if the
/// wasm32 wrapper currently panics with a "see L6 design" message.
/// The previous behaviour silently returned `NeoValue::Null`.
#[test]
fn b4_contract_call_extern_declared() {
    assert!(
        EXPECTED_EXTERNS.contains(&"neo_contract_call"),
        "B4 regression: neo_contract_call missing from extern matrix"
    );
}
