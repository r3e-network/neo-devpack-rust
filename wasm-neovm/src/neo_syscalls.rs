// Neo N3 syscall mappings for WASM imports
// Maps friendly import names from (import "neo" "function_name") to actual Neo syscall names

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Mapping from WASM import names to Neo syscall names
/// Used for (import "neo" "storage_get") style imports
pub static NEO_SYSCALL_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut map = HashMap::new();

    // Storage syscalls
    map.insert("storage_get", "System.Storage.Get");
    map.insert("storage_put", "System.Storage.Put");
    map.insert("storage_delete", "System.Storage.Delete");
    map.insert("storage_find", "System.Storage.Find");
    map.insert("storage_get_context", "System.Storage.GetContext");
    map.insert(
        "storage_get_readonly_context",
        "System.Storage.GetReadOnlyContext",
    );
    map.insert("storage_as_readonly", "System.Storage.AsReadOnly");

    // Runtime syscalls
    map.insert("check_witness", "System.Runtime.CheckWitness");
    map.insert("runtime_check_witness", "System.Runtime.CheckWitness");
    map.insert("log", "System.Runtime.Log");
    map.insert("runtime_log", "System.Runtime.Log");
    map.insert("notify", "System.Runtime.Notify");
    map.insert("runtime_notify", "System.Runtime.Notify");
    map.insert("get_time", "System.Runtime.GetTime");
    map.insert("runtime_get_time", "System.Runtime.GetTime");
    map.insert("get_trigger", "System.Runtime.GetTrigger");
    map.insert("runtime_get_trigger", "System.Runtime.GetTrigger");
    map.insert("get_platform", "System.Runtime.Platform");
    map.insert("get_script_container", "System.Runtime.GetScriptContainer");
    map.insert(
        "get_executing_script_hash",
        "System.Runtime.GetExecutingScriptHash",
    );
    map.insert(
        "get_calling_script_hash",
        "System.Runtime.GetCallingScriptHash",
    );
    map.insert("get_entry_script_hash", "System.Runtime.GetEntryScriptHash");
    map.insert("get_network", "System.Runtime.GetNetwork");
    map.insert("get_random", "System.Runtime.GetRandom");
    map.insert(
        "get_invocation_counter",
        "System.Runtime.GetInvocationCounter",
    );
    map.insert("get_notifications", "System.Runtime.GetNotifications");
    map.insert("get_address_version", "System.Runtime.GetAddressVersion");
    map.insert("current_signers", "System.Runtime.CurrentSigners");
    map.insert("burn_gas", "System.Runtime.BurnGas");
    map.insert("gas_left", "System.Runtime.GasLeft");
    map.insert("load_script", "System.Runtime.LoadScript");

    // Crypto syscalls
    map.insert("verify_signature", "System.Crypto.CheckSig");
    map.insert("check_sig", "System.Crypto.CheckSig");
    map.insert("check_multisig", "System.Crypto.CheckMultisig");
    map.insert("crypto_sha256", "Neo.Crypto.SHA256");
    map.insert("crypto_hash160", "Neo.Crypto.Hash160");
    map.insert("crypto_hash256", "Neo.Crypto.Hash256");
    // Hashing helpers are not exposed as Neo syscalls. Callers should lower
    // to opcodes (e.g. `opcode::HASH160`) or native contract calls instead.

    // Contract management syscalls
    map.insert("call_contract", "System.Contract.Call");
    map.insert("contract_call", "System.Contract.Call");
    map.insert("contract_create", "System.Contract.Call"); // Creation done via Call to ContractManagement
    map.insert("contract_destroy", "System.Contract.Call"); // Destruction done via Call to ContractManagement
    map.insert("call_native", "System.Contract.CallNative");
    map.insert("get_call_flags", "System.Contract.GetCallFlags");
    map.insert(
        "create_standard_account",
        "System.Contract.CreateStandardAccount",
    );
    map.insert(
        "create_multisig_account",
        "System.Contract.CreateMultisigAccount",
    );

    // Iterator syscalls
    map.insert("iterator_next", "System.Iterator.Next");
    map.insert("iterator_value", "System.Iterator.Value");

    map
});

/// Lookup a Neo syscall name from a WASM import name
pub fn lookup_neo_syscall(import_name: &str) -> Option<&'static str> {
    NEO_SYSCALL_MAP.get(import_name).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
