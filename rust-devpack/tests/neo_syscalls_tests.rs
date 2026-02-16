// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

// Neo N3 syscall integration tests aligned with the canonical registry

use neo_devpack::prelude::*;
use neo_syscalls::*;
use std::sync::{Mutex, MutexGuard, OnceLock};

fn syscall_test_lock() -> MutexGuard<'static, ()> {
    static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    match TEST_LOCK.get_or_init(|| Mutex::new(())).lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn setup_syscall_test() -> MutexGuard<'static, ()> {
    let guard = syscall_test_lock();
    NeoVMSyscall::reset_host_state().expect("host syscall state should reset");
    guard
}

fn registry() -> NeoVMSyscallRegistry {
    NeoVMSyscallRegistry::get_instance()
}

fn placeholder_arg(param: &str) -> NeoValue {
    match param {
        "Boolean" => NeoBoolean::FALSE.into(),
        "Integer" => NeoInteger::new(0).into(),
        "Hash160" => NeoByteString::new(vec![0u8; 20]).into(),
        "Hash256" => NeoByteString::new(vec![0u8; 32]).into(),
        "ByteString" | "Buffer" | "StackItem" | "Any" | "ExecutionContext" => {
            NeoByteString::new(vec![]).into()
        }
        "String" => NeoString::from_str("").into(),
        "Array" | "Iterator" => NeoArray::<NeoValue>::new().into(),
        "Map" => NeoMap::<NeoValue, NeoValue>::new().into(),
        "Struct" => NeoStruct::new().into(),
        _ => NeoValue::Null,
    }
}

fn assert_value_matches_type(value: &NeoValue, ty: &str) {
    match ty {
        "Void" => assert!(value.is_null()),
        "Boolean" => assert!(value.as_boolean().is_some()),
        "Integer" => assert!(value.as_integer().is_some()),
        "Hash160" | "Hash256" | "ByteString" | "Buffer" => {
            assert!(value.as_byte_string().is_some())
        }
        "String" => assert!(value.as_string().is_some()),
        "Array" | "Iterator" => assert!(value.as_array().is_some()),
        "Map" => assert!(value.as_map().is_some()),
        "Struct" => assert!(value.as_struct().is_some()),
        _ => (),
    }
}

#[test]
fn registry_contains_expected_syscalls() {
    let _guard = setup_syscall_test();
    let registry = registry();
    let names: Vec<_> = registry.names().collect();
    assert_eq!(names.len(), 38);
    assert!(names.contains(&"System.Runtime.GetTime"));
    assert!(names.contains(&"System.Runtime.GasLeft"));
    assert!(names.contains(&"System.Contract.Call"));
    assert!(names.contains(&"System.Storage.Get"));
    assert!(names.contains(&"Neo.Crypto.VerifyWithECDsa"));
}

#[test]
fn hash_lookup_matches_name_lookup() {
    let _guard = setup_syscall_test();
    let registry = registry();
    for info in registry.iter() {
        let by_hash = registry
            .get_syscall_by_hash(info.hash)
            .expect("hash lookup failed");
        assert_eq!(info, by_hash);
    }
}

#[test]
fn neovm_syscall_returns_placeholder_for_known_entries() {
    let _guard = setup_syscall_test();
    let registry = registry();
    for info in registry.iter() {
        let args: Vec<NeoValue> = info.parameters.iter().map(|p| placeholder_arg(p)).collect();
        let result = neovm_syscall(info.hash, &args).expect("syscall invocation failed");
        assert_value_matches_type(&result, info.return_type);
    }
}

#[test]
fn neovm_syscall_handles_unknown_hash() {
    let _guard = setup_syscall_test();
    let result = neovm_syscall(0xDEADBEEF, &[]);
    assert!(result.is_err());
}

#[test]
fn neovm_syscall_rejects_argument_count_mismatch() {
    let _guard = setup_syscall_test();
    let registry = registry();
    let info = registry
        .get_syscall("System.Runtime.Log")
        .expect("syscall exists");
    let err = neovm_syscall(info.hash, &[]).unwrap_err();
    assert!(err.message().contains("invalid syscall argument count"));
}

#[test]
fn neovm_syscall_rejects_argument_type_mismatch() {
    let _guard = setup_syscall_test();
    let registry = registry();
    let info = registry
        .get_syscall("System.Runtime.Log")
        .expect("syscall exists");
    let args = [NeoValue::from(NeoInteger::new(7))];
    let err = neovm_syscall(info.hash, &args).unwrap_err();
    assert!(err.message().contains("invalid syscall argument type"));
}

#[test]
fn neovm_syscall_rejects_invalid_hash160_length() {
    let _guard = setup_syscall_test();
    let registry = registry();
    let info = registry
        .get_syscall("System.Contract.Call")
        .expect("syscall exists");

    let args = [
        NeoValue::from(NeoByteString::new(vec![0u8; 19])),
        NeoValue::from(NeoString::from_str("transfer")),
        NeoValue::from(NeoInteger::new(0)),
        NeoValue::from(NeoArray::<NeoValue>::new()),
    ];

    let err = neovm_syscall(info.hash, &args).unwrap_err();
    assert!(err.message().contains("invalid syscall argument type"));
}

#[test]
fn syscall_wrapper_supports_extended_system_surface() {
    let _guard = setup_syscall_test();
    let script_hash = NeoByteString::new(vec![1u8; 20]);
    let method = NeoString::from_str("balanceOf");
    let call_flags = NeoInteger::new(0);
    let args = NeoArray::<NeoValue>::new();

    let call_result = NeoVMSyscall::contract_call(&script_hash, &method, &call_flags, &args)
        .expect("contract call wrapper");
    assert!(call_result.is_null());

    let native_result =
        NeoVMSyscall::contract_call_native(&NeoInteger::new(1)).expect("contract call native");
    assert!(native_result.is_null());

    let flags = NeoVMSyscall::get_call_flags().expect("get call flags");
    assert_eq!(flags, NeoInteger::new(0));

    let standard_account =
        NeoVMSyscall::create_standard_account(&NeoByteString::new(vec![2u8; 33]))
            .expect("create standard account");
    assert_eq!(standard_account.len(), 20);

    let mut pubkeys = NeoArray::<NeoValue>::new();
    pubkeys.push(NeoByteString::new(vec![3u8; 33]).into());
    pubkeys.push(NeoByteString::new(vec![4u8; 33]).into());
    let multisig_account = NeoVMSyscall::create_multisig_account(&NeoInteger::new(2), &pubkeys)
        .expect("create multisig account");
    assert_eq!(multisig_account.len(), 20);

    NeoVMSyscall::native_on_persist().expect("native on persist");
    NeoVMSyscall::native_post_persist().expect("native post persist");

    NeoVMSyscall::set_crypto_verification_results(true, true).expect("set crypto results");
    let check_sig = NeoVMSyscall::check_sig(
        &NeoByteString::new(vec![5u8; 33]),
        &NeoByteString::new(vec![6u8; 64]),
    )
    .expect("check sig");
    assert!(check_sig.as_bool());

    let mut signatures = NeoArray::<NeoValue>::new();
    signatures.push(NeoByteString::new(vec![7u8; 64]).into());
    let check_multisig =
        NeoVMSyscall::check_multisig(&pubkeys, &signatures).expect("check multisig");
    assert!(check_multisig.as_bool());

    let verify_with_ecdsa = NeoVMSyscall::verify_with_ecdsa(
        &NeoByteString::new(vec![8u8; 32]),
        &NeoByteString::new(vec![9u8; 33]),
        &NeoByteString::new(vec![10u8; 64]),
        &NeoInteger::new(1),
    )
    .expect("verify with ecdsa");
    assert!(verify_with_ecdsa.as_bool());

    let iterator_values = NeoArray::<NeoValue>::new();
    let has_next = NeoVMSyscall::iterator_next(&iterator_values).expect("iterator next");
    assert!(has_next.as_bool());
    let iterator_value = NeoVMSyscall::iterator_value(&iterator_values).expect("iterator value");
    assert!(iterator_value.as_array().is_some());

    NeoVMSyscall::burn_gas(&NeoInteger::new(1)).expect("burn gas");
    let signers = NeoVMSyscall::current_signers().expect("current signers");
    assert!(signers.is_empty());
    NeoVMSyscall::load_script(&NeoByteString::new(vec![]), &NeoInteger::new(0), &args)
        .expect("load script");
}

#[test]
fn host_overrides_check_witness_and_script_hash_syscalls() {
    let _guard = setup_syscall_test();

    let registry = registry();
    let check_witness = registry
        .get_syscall("System.Runtime.CheckWitness")
        .expect("check witness syscall");
    let account = NeoByteString::new(vec![9u8; 20]);
    let witness_args = [NeoValue::from(account.clone())];

    let initial = neovm_syscall(check_witness.hash, &witness_args).expect("check witness call");
    assert!(!initial.as_boolean().expect("boolean result").as_bool());

    NeoVMSyscall::set_active_witnesses(std::slice::from_ref(&account)).expect("set active witness");
    let updated = neovm_syscall(check_witness.hash, &witness_args).expect("check witness call");
    assert!(updated.as_boolean().expect("boolean result").as_bool());

    let active_hash = NeoByteString::new(vec![0xAB; 20]);
    NeoVMSyscall::set_active_contract_hash(&active_hash).expect("set active contract hash");

    for name in [
        "System.Runtime.GetCallingScriptHash",
        "System.Runtime.GetEntryScriptHash",
        "System.Runtime.GetExecutingScriptHash",
    ] {
        let syscall = registry.get_syscall(name).expect("script hash syscall");
        let result = neovm_syscall(syscall.hash, &[]).expect("script hash call");
        assert_eq!(
            result.as_byte_string().expect("bytes result"),
            &active_hash,
            "{name} should reflect active host contract hash"
        );
    }

    let check_sig = registry
        .get_syscall("System.Crypto.CheckSig")
        .expect("check sig syscall");
    let check_multisig = registry
        .get_syscall("System.Crypto.CheckMultisig")
        .expect("check multisig syscall");
    let verify_with_ecdsa = registry
        .get_syscall("Neo.Crypto.VerifyWithECDsa")
        .expect("verify with ecdsa syscall");
    let check_sig_args = [
        NeoValue::from(NeoByteString::new(vec![1u8; 33])),
        NeoValue::from(NeoByteString::new(vec![2u8; 64])),
    ];
    let check_multisig_args = [
        NeoValue::from(NeoArray::<NeoValue>::new()),
        NeoValue::from(NeoArray::<NeoValue>::new()),
    ];
    let verify_with_ecdsa_args = [
        NeoValue::from(NeoByteString::new(vec![3u8; 32])),
        NeoValue::from(NeoByteString::new(vec![4u8; 33])),
        NeoValue::from(NeoByteString::new(vec![5u8; 64])),
        NeoValue::from(NeoInteger::new(1)),
    ];

    assert!(!neovm_syscall(check_sig.hash, &check_sig_args)
        .expect("check sig call")
        .as_boolean()
        .expect("boolean result")
        .as_bool());
    assert!(!neovm_syscall(check_multisig.hash, &check_multisig_args)
        .expect("check multisig call")
        .as_boolean()
        .expect("boolean result")
        .as_bool());
    assert!(
        !neovm_syscall(verify_with_ecdsa.hash, &verify_with_ecdsa_args)
            .expect("verify with ecdsa call")
            .as_boolean()
            .expect("boolean result")
            .as_bool()
    );

    NeoVMSyscall::set_crypto_verification_results(true, true).expect("set crypto results");
    assert!(neovm_syscall(check_sig.hash, &check_sig_args)
        .expect("check sig call")
        .as_boolean()
        .expect("boolean result")
        .as_bool());
    assert!(neovm_syscall(check_multisig.hash, &check_multisig_args)
        .expect("check multisig call")
        .as_boolean()
        .expect("boolean result")
        .as_bool());
    assert!(
        neovm_syscall(verify_with_ecdsa.hash, &verify_with_ecdsa_args)
            .expect("verify with ecdsa call")
            .as_boolean()
            .expect("boolean result")
            .as_bool()
    );

    NeoVMSyscall::set_verify_with_ecdsa_result(false).expect("set verify with ecdsa result");
    assert!(
        !neovm_syscall(verify_with_ecdsa.hash, &verify_with_ecdsa_args)
            .expect("verify with ecdsa call")
            .as_boolean()
            .expect("boolean result")
            .as_bool()
    );
}
