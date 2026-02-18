// Neo N3 syscall mappings for WASM imports
// Maps friendly import names from (import "neo" "function_name") to actual Neo syscall names

use once_cell::sync::Lazy;
use std::collections::HashMap;

use crate::syscalls;

fn descriptor_fingerprint(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            output.push(ch.to_ascii_lowercase());
        }
    }
    output
}

fn camel_to_snake(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut prev_was_lower_or_digit = false;

    for ch in input.chars() {
        if ch.is_ascii_uppercase() {
            if prev_was_lower_or_digit {
                output.push('_');
            }
            output.push(ch.to_ascii_lowercase());
            prev_was_lower_or_digit = false;
        } else {
            output.push(ch.to_ascii_lowercase());
            prev_was_lower_or_digit = ch.is_ascii_lowercase() || ch.is_ascii_digit();
        }
    }

    output
}

fn canonical_aliases(descriptor: &str) -> Vec<String> {
    let mut parts = descriptor.split('.');
    let root = match parts.next() {
        Some(value) => value,
        None => return Vec::new(),
    };

    let category = match parts.next() {
        Some(value) => value,
        None => return Vec::new(),
    };
    let method = match parts.next() {
        Some(value) => value,
        None => return Vec::new(),
    };
    if parts.next().is_some() {
        return Vec::new();
    }

    let category = category.to_ascii_lowercase();
    let method = camel_to_snake(method);

    match root {
        "System" | "Neo" => vec![format!("{category}_{method}")],
        _ => Vec::new(),
    }
}

fn register_fingerprint(
    map: &mut HashMap<String, &'static str>,
    key: &str,
    descriptor: &'static str,
) {
    let fingerprint = descriptor_fingerprint(key);
    if fingerprint.is_empty() {
        return;
    }

    if let Some(existing) = map.get(&fingerprint) {
        if *existing != descriptor {
            panic!(
                "ambiguous syscall fingerprint '{}' for '{}' and '{}' (from key '{}')",
                fingerprint, existing, descriptor, key
            );
        }
        return;
    }

    map.insert(fingerprint, descriptor);
}

/// Mapping from WASM import names to Neo syscall names
/// Used for (import "neo" "storage_get") style imports
pub static NEO_SYSCALL_MAP: Lazy<HashMap<String, &'static str>> = Lazy::new(|| {
    let mut map: HashMap<String, &'static str> = HashMap::new();
    let mut alias = |import_name: &str, descriptor: &'static str| {
        map.insert(import_name.to_string(), descriptor);
    };

    // Storage syscalls
    alias("storage_get", "System.Storage.Get");
    alias("storage_put", "System.Storage.Put");
    alias("storage_delete", "System.Storage.Delete");
    alias("storage_find", "System.Storage.Find");
    alias("storage_get_context", "System.Storage.GetContext");
    alias(
        "storage_get_readonly_context",
        "System.Storage.GetReadOnlyContext",
    );
    alias(
        "storage_get_read_only_context",
        "System.Storage.GetReadOnlyContext",
    );
    alias("storage_as_readonly", "System.Storage.AsReadOnly");
    alias("storage_as_read_only", "System.Storage.AsReadOnly");

    // Runtime syscalls
    alias("check_witness", "System.Runtime.CheckWitness");
    alias("runtime_check_witness", "System.Runtime.CheckWitness");
    alias("runtime_check_witness_hash", "System.Runtime.CheckWitness");
    alias("log", "System.Runtime.Log");
    alias("runtime_log", "System.Runtime.Log");
    alias("notify", "System.Runtime.Notify");
    alias("runtime_notify", "System.Runtime.Notify");
    alias("get_time", "System.Runtime.GetTime");
    alias("runtime_get_time", "System.Runtime.GetTime");
    alias("get_trigger", "System.Runtime.GetTrigger");
    alias("runtime_get_trigger", "System.Runtime.GetTrigger");
    alias("get_platform", "System.Runtime.Platform");
    alias("runtime_platform", "System.Runtime.Platform");
    alias("get_script_container", "System.Runtime.GetScriptContainer");
    alias(
        "get_executing_script_hash",
        "System.Runtime.GetExecutingScriptHash",
    );
    alias(
        "get_calling_script_hash",
        "System.Runtime.GetCallingScriptHash",
    );
    alias("get_entry_script_hash", "System.Runtime.GetEntryScriptHash");
    alias("get_network", "System.Runtime.GetNetwork");
    alias("runtime_get_network", "System.Runtime.GetNetwork");
    alias("get_random", "System.Runtime.GetRandom");
    alias(
        "get_invocation_counter",
        "System.Runtime.GetInvocationCounter",
    );
    alias("get_notifications", "System.Runtime.GetNotifications");
    alias("get_address_version", "System.Runtime.GetAddressVersion");
    alias("current_signers", "System.Runtime.CurrentSigners");
    alias("burn_gas", "System.Runtime.BurnGas");
    alias("gas_left", "System.Runtime.GasLeft");
    alias("load_script", "System.Runtime.LoadScript");

    // Crypto syscalls
    alias("verify_signature", "System.Crypto.CheckSig");
    alias("check_sig", "System.Crypto.CheckSig");
    alias("check_multisig", "System.Crypto.CheckMultisig");
    alias("verify_with_ecdsa", "Neo.Crypto.VerifyWithECDsa");
    alias("crypto_verify_with_ecdsa", "Neo.Crypto.VerifyWithECDsa");
    alias("crypto_sha256", "Neo.Crypto.SHA256");
    alias("crypto_hash160", "Neo.Crypto.Hash160");
    alias("crypto_hash256", "Neo.Crypto.Hash256");
    // Hashing helpers are not exposed as Neo syscalls. Callers should lower
    // to opcodes (e.g. `opcode::HASH160`) or native contract calls instead.

    // Contract management syscalls
    alias("call_contract", "System.Contract.Call");
    alias("contract_call", "System.Contract.Call");
    alias("contract_create", "System.Contract.Call"); // Creation done via Call to ContractManagement
    alias("contract_destroy", "System.Contract.Call"); // Destruction done via Call to ContractManagement
    alias("call_native", "System.Contract.CallNative");
    alias("contract_call_native", "System.Contract.CallNative");
    alias("get_call_flags", "System.Contract.GetCallFlags");
    alias(
        "create_standard_account",
        "System.Contract.CreateStandardAccount",
    );
    alias(
        "create_multisig_account",
        "System.Contract.CreateMultisigAccount",
    );

    // Iterator syscalls
    alias("iterator_next", "System.Iterator.Next");
    alias("iterator_value", "System.Iterator.Value");

    // Extended descriptor names (resolved through lookup_extended)
    alias("Neo.Crypto.SHA256", "Neo.Crypto.SHA256");
    alias("Neo.Crypto.RIPEMD160", "Neo.Crypto.RIPEMD160");
    alias("Neo.Crypto.Murmur32", "Neo.Crypto.Murmur32");
    alias("Neo.Crypto.Keccak256", "Neo.Crypto.Keccak256");
    alias("Neo.Crypto.Hash160", "Neo.Crypto.Hash160");
    alias("Neo.Crypto.Hash256", "Neo.Crypto.Hash256");
    alias("Neo.Crypto.VerifyWithECDsa", "Neo.Crypto.VerifyWithECDsa");

    // Canonical coverage for all Neo N3 engine syscalls and extended descriptors.
    for syscall in syscalls::all().iter().chain(syscalls::extended().iter()) {
        let descriptor = syscall.name;
        map.entry(descriptor.to_string()).or_insert(descriptor);
        for generated_alias in canonical_aliases(descriptor) {
            map.entry(generated_alias).or_insert(descriptor);
        }
    }

    map
});

pub static NEO_SYSCALL_FINGERPRINT_MAP: Lazy<HashMap<String, &'static str>> = Lazy::new(|| {
    let mut map: HashMap<String, &'static str> = HashMap::new();

    for (alias, descriptor) in NEO_SYSCALL_MAP.iter() {
        register_fingerprint(&mut map, alias, descriptor);
    }

    for syscall in syscalls::all().iter().chain(syscalls::extended().iter()) {
        register_fingerprint(&mut map, syscall.name, syscall.name);
        for alias in canonical_aliases(syscall.name) {
            register_fingerprint(&mut map, &alias, syscall.name);
        }
    }

    map
});

/// Lookup a Neo syscall name from a WASM import name
pub fn lookup_neo_syscall(import_name: &str) -> Option<&'static str> {
    if let Some(mapped) = NEO_SYSCALL_MAP.get(import_name) {
        return Some(*mapped);
    }

    let fingerprint = descriptor_fingerprint(import_name);
    if fingerprint.is_empty() {
        return None;
    }

    NEO_SYSCALL_FINGERPRINT_MAP.get(&fingerprint).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_alias_maps_to_known_syscall(alias: &str) {
        let descriptor = lookup_neo_syscall(alias)
            .unwrap_or_else(|| panic!("missing alias mapping for {alias}"));
        let resolved = syscalls::lookup_extended(descriptor)
            .unwrap_or_else(|| panic!("mapped descriptor '{descriptor}' is unknown"));
        assert_eq!(resolved.name, descriptor);
    }

    #[test]
    fn test_storage_syscalls() {
        assert_eq!(
            lookup_neo_syscall("storage_get"),
            Some("System.Storage.Get")
        );
        assert_eq!(
            lookup_neo_syscall("storage_put"),
            Some("System.Storage.Put")
        );
        assert_eq!(
            lookup_neo_syscall("storage_delete"),
            Some("System.Storage.Delete")
        );
    }

    #[test]
    fn test_runtime_syscalls() {
        assert_eq!(
            lookup_neo_syscall("check_witness"),
            Some("System.Runtime.CheckWitness")
        );
        assert_eq!(lookup_neo_syscall("log"), Some("System.Runtime.Log"));
        assert_eq!(lookup_neo_syscall("notify"), Some("System.Runtime.Notify"));
        assert_eq!(
            lookup_neo_syscall("get_time"),
            Some("System.Runtime.GetTime")
        );
    }

    #[test]
    fn test_crypto_syscalls() {
        assert_eq!(
            lookup_neo_syscall("verify_signature"),
            Some("System.Crypto.CheckSig")
        );
        assert_eq!(
            lookup_neo_syscall("check_sig"),
            Some("System.Crypto.CheckSig")
        );
        assert_eq!(
            lookup_neo_syscall("verify_with_ecdsa"),
            Some("Neo.Crypto.VerifyWithECDsa")
        );
        assert_eq!(
            lookup_neo_syscall("crypto_verify_with_ecdsa"),
            Some("Neo.Crypto.VerifyWithECDsa")
        );
    }

    #[test]
    fn hashes_are_not_syscalls() {
        assert_eq!(lookup_neo_syscall("hash160"), None);
        assert_eq!(lookup_neo_syscall("hash256"), None);
    }

    #[test]
    fn test_contract_syscalls() {
        assert_eq!(
            lookup_neo_syscall("call_contract"),
            Some("System.Contract.Call")
        );
        assert_eq!(
            lookup_neo_syscall("contract_call"),
            Some("System.Contract.Call")
        );
    }

    #[test]
    fn test_unknown_syscall() {
        assert_eq!(lookup_neo_syscall("unknown_function"), None);
    }

    #[test]
    fn canonical_system_syscalls_have_aliases() {
        for syscall in syscalls::all() {
            assert_eq!(lookup_neo_syscall(syscall.name), Some(syscall.name));
            for alias in canonical_aliases(syscall.name) {
                assert_eq!(lookup_neo_syscall(&alias), Some(syscall.name));
            }
        }
    }

    #[test]
    fn canonical_extended_syscalls_have_aliases() {
        for syscall in syscalls::extended() {
            assert_eq!(lookup_neo_syscall(syscall.name), Some(syscall.name));
            for alias in canonical_aliases(syscall.name) {
                assert_eq!(lookup_neo_syscall(&alias), Some(syscall.name));
            }
        }
    }

    #[test]
    fn generated_aliases_cover_edge_cases() {
        assert_eq!(
            lookup_neo_syscall("runtime_get_calling_script_hash"),
            Some("System.Runtime.GetCallingScriptHash")
        );
        assert_eq!(
            lookup_neo_syscall("runtime_get_executing_script_hash"),
            Some("System.Runtime.GetExecutingScriptHash")
        );
        assert_eq!(
            lookup_neo_syscall("runtime_get_invocation_counter"),
            Some("System.Runtime.GetInvocationCounter")
        );
        assert_eq!(
            lookup_neo_syscall("storage_get_read_only_context"),
            Some("System.Storage.GetReadOnlyContext")
        );
    }

    #[test]
    fn all_aliases_resolve_to_known_syscalls() {
        for alias in NEO_SYSCALL_MAP.keys().map(String::as_str) {
            assert_alias_maps_to_known_syscall(alias);
        }
    }

    #[test]
    fn lookup_accepts_case_separator_and_whitespace_variants() {
        assert_eq!(
            lookup_neo_syscall("system.runtime.gettime"),
            Some("System.Runtime.GetTime")
        );
        assert_eq!(
            lookup_neo_syscall(" System/Runtime/GetExecutingScriptHash "),
            Some("System.Runtime.GetExecutingScriptHash")
        );
        assert_eq!(
            lookup_neo_syscall("runtime-get-time"),
            Some("System.Runtime.GetTime")
        );
        assert_eq!(
            lookup_neo_syscall("neo.crypto.verifywithecdsa"),
            Some("Neo.Crypto.VerifyWithECDsa")
        );
    }

    #[test]
    fn lookup_rejects_near_collision_variants_with_missing_characters() {
        assert_eq!(lookup_neo_syscall("runtime_get_tim"), None);
        assert_eq!(lookup_neo_syscall("system.runtime.gettim"), None);
        assert_eq!(lookup_neo_syscall("runtime_get_invocation_counte"), None);
        assert_eq!(lookup_neo_syscall("system/storage/getreadonlycontex"), None);
    }

    #[test]
    fn canonical_descriptor_fingerprints_resolve_without_ambiguity() {
        for syscall in syscalls::all().iter().chain(syscalls::extended().iter()) {
            let fingerprint = descriptor_fingerprint(syscall.name);
            let resolved = NEO_SYSCALL_FINGERPRINT_MAP
                .get(&fingerprint)
                .copied()
                .unwrap_or_else(|| panic!("missing fingerprint mapping for {}", syscall.name));
            assert_eq!(
                resolved, syscall.name,
                "fingerprint of canonical descriptor '{}' should resolve to itself",
                syscall.name
            );
        }
    }
}
