// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

// Neo N3 syscall integration tests aligned with the canonical registry

use neo_devpack::prelude::*;
use neo_syscalls::*;

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
        "Array" => NeoArray::<NeoValue>::new().into(),
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
    let registry = registry();
    let names: Vec<_> = registry.names().collect();
    assert_eq!(names.len(), 37);
    assert!(names.contains(&"System.Runtime.GetTime"));
    assert!(names.contains(&"System.Runtime.GasLeft"));
    assert!(names.contains(&"System.Contract.Call"));
    assert!(names.contains(&"System.Storage.Get"));
}

#[test]
fn hash_lookup_matches_name_lookup() {
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
    let registry = registry();
    for info in registry.iter() {
        let args: Vec<NeoValue> = info.parameters.iter().map(|p| placeholder_arg(p)).collect();
        let result = neovm_syscall(info.hash, &args).expect("syscall invocation failed");
        assert_value_matches_type(&result, info.return_type);
    }
}

#[test]
fn neovm_syscall_handles_unknown_hash() {
    let result = neovm_syscall(0xDEADBEEF, &[]).expect("unexpected error for unknown syscall");
    assert!(result.is_null());
}
