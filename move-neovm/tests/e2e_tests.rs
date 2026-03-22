// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! End-to-end Move to WASM translation tests (experimental)

use move_neovm::{
    bytecode::{
        AbilitySet, BytecodeVersion, FunctionDef, MoveModule, MoveOpcode, StructDef, TypeTag,
    },
    translate_to_wasm,
};
use wasmparser::Validator;

fn assert_valid_wasm(bytes: &[u8]) {
    assert!(
        Validator::new().validate_all(bytes).is_ok(),
        "generated wasm did not validate"
    );
}

/// Test translating a simple Move module to WASM
#[test]
fn test_move_to_wasm_simple_add() {
    let module = MoveModule {
        version: BytecodeVersion(6),
        name: "SimpleAdd".to_string(),
        identifiers_offset: 0,
        identifiers_count: 0,
        struct_defs_offset: 0,
        struct_defs_count: 0,
        _function_handles_offset: 0,
        _function_handles_count: 0,
        function_defs_offset: 0,
        function_defs_count: 0,
        structs: vec![],
        functions: vec![FunctionDef {
            name: "add".to_string(),
            is_public: true,
            is_entry: false,
            parameters: vec![TypeTag::U64, TypeTag::U64],
            returns: vec![TypeTag::U64],
            locals: vec![],
            code: vec![
                MoveOpcode::CopyLoc(0),
                MoveOpcode::CopyLoc(1),
                MoveOpcode::Add,
                MoveOpcode::Ret,
            ],
        }],
    };

    let wasm = translate_to_wasm(&module).expect("Translation should succeed");
    assert_valid_wasm(&wasm);
}

/// Test translating a module with multiple functions
#[test]
fn test_move_to_wasm_multiple_functions() {
    let module = MoveModule {
        version: BytecodeVersion(6),
        name: "Math".to_string(),
        identifiers_offset: 0,
        identifiers_count: 0,
        struct_defs_offset: 0,
        struct_defs_count: 0,
        _function_handles_offset: 0,
        _function_handles_count: 0,
        function_defs_offset: 0,
        function_defs_count: 0,
        structs: vec![],
        functions: vec![
            FunctionDef {
                name: "add".to_string(),
                is_public: true,
                is_entry: false,
                parameters: vec![TypeTag::U64, TypeTag::U64],
                returns: vec![TypeTag::U64],
                locals: vec![],
                code: vec![
                    MoveOpcode::CopyLoc(0),
                    MoveOpcode::CopyLoc(1),
                    MoveOpcode::Add,
                    MoveOpcode::Ret,
                ],
            },
            FunctionDef {
                name: "sub".to_string(),
                is_public: true,
                is_entry: false,
                parameters: vec![TypeTag::U64, TypeTag::U64],
                returns: vec![TypeTag::U64],
                locals: vec![],
                code: vec![
                    MoveOpcode::CopyLoc(0),
                    MoveOpcode::CopyLoc(1),
                    MoveOpcode::Sub,
                    MoveOpcode::Ret,
                ],
            },
        ],
    };

    let wasm = translate_to_wasm(&module).expect("Translation should succeed");
    assert_valid_wasm(&wasm);
}

/// Test translating a module with entry function
#[test]
fn test_move_to_wasm_entry_function() {
    let module = MoveModule {
        version: BytecodeVersion(6),
        name: "EntryModule".to_string(),
        identifiers_offset: 0,
        identifiers_count: 0,
        struct_defs_offset: 0,
        struct_defs_count: 0,
        _function_handles_offset: 0,
        _function_handles_count: 0,
        function_defs_offset: 0,
        function_defs_count: 0,
        structs: vec![],
        functions: vec![FunctionDef {
            name: "main".to_string(),
            is_public: false,
            is_entry: true,
            parameters: vec![],
            returns: vec![],
            locals: vec![],
            code: vec![MoveOpcode::LdU64(42), MoveOpcode::Pop, MoveOpcode::Ret],
        }],
    };

    let wasm = translate_to_wasm(&module).expect("Translation should succeed");
    assert_valid_wasm(&wasm);
}

/// Test translating comparison operations
#[test]
fn test_move_to_wasm_comparisons() {
    let module = MoveModule {
        version: BytecodeVersion(6),
        name: "Compare".to_string(),
        identifiers_offset: 0,
        identifiers_count: 0,
        struct_defs_offset: 0,
        struct_defs_count: 0,
        _function_handles_offset: 0,
        _function_handles_count: 0,
        function_defs_offset: 0,
        function_defs_count: 0,
        structs: vec![],
        functions: vec![FunctionDef {
            name: "less_than".to_string(),
            is_public: true,
            is_entry: false,
            parameters: vec![TypeTag::U64, TypeTag::U64],
            returns: vec![TypeTag::Bool],
            locals: vec![],
            code: vec![
                MoveOpcode::CopyLoc(0),
                MoveOpcode::CopyLoc(1),
                MoveOpcode::Lt,
                MoveOpcode::Ret,
            ],
        }],
    };

    let wasm = translate_to_wasm(&module).expect("Translation should succeed");
    assert_valid_wasm(&wasm);
}

/// Branch offsets should be honoured via the dispatch loop
#[test]
fn test_move_to_wasm_control_flow() {
    let module = MoveModule {
        version: BytecodeVersion(6),
        name: "Control".to_string(),
        identifiers_offset: 0,
        identifiers_count: 0,
        struct_defs_offset: 0,
        struct_defs_count: 0,
        _function_handles_offset: 0,
        _function_handles_count: 0,
        function_defs_offset: 0,
        function_defs_count: 0,
        structs: vec![],
        functions: vec![FunctionDef {
            name: "abs".to_string(),
            is_public: true,
            is_entry: false,
            parameters: vec![TypeTag::U64],
            returns: vec![TypeTag::U64],
            locals: vec![TypeTag::U64],
            code: vec![
                MoveOpcode::CopyLoc(0),
                MoveOpcode::LdU64(0),
                MoveOpcode::Lt,
                MoveOpcode::BrFalse(6),
                MoveOpcode::CopyLoc(0),
                MoveOpcode::Ret,
                MoveOpcode::CopyLoc(0),
                MoveOpcode::LdU64(0),
                MoveOpcode::Sub,
                MoveOpcode::Ret,
            ],
        }],
    };

    let wasm = translate_to_wasm(&module).expect("branches should lower");
    assert_valid_wasm(&wasm);
}

/// Test translating with resource struct (no resource ops)
#[test]
fn test_move_to_wasm_with_resource() {
    let module = MoveModule {
        version: BytecodeVersion(6),
        name: "Coin".to_string(),
        identifiers_offset: 0,
        identifiers_count: 0,
        struct_defs_offset: 0,
        struct_defs_count: 0,
        _function_handles_offset: 0,
        _function_handles_count: 0,
        function_defs_offset: 0,
        function_defs_count: 0,
        structs: vec![StructDef {
            name: "Coin".to_string(),
            abilities: AbilitySet {
                key: true,
                store: true,
                copy: false,
                drop: false,
            },
            fields: vec![],
        }],
        functions: vec![FunctionDef {
            name: "value".to_string(),
            is_public: true,
            is_entry: false,
            parameters: vec![TypeTag::Reference(Box::new(TypeTag::Struct(
                "Coin".to_string(),
            )))],
            returns: vec![TypeTag::U64],
            locals: vec![],
            code: vec![MoveOpcode::LdU64(100), MoveOpcode::Ret],
        }],
    };

    let wasm = translate_to_wasm(&module).expect("Translation should succeed");
    assert_valid_wasm(&wasm);
}

/// Resource operations should be lowered to storage syscalls
#[test]
fn test_move_resource_ops_lowered() {
    let module = MoveModule {
        version: BytecodeVersion(6),
        name: "ResourceOps".to_string(),
        identifiers_offset: 0,
        identifiers_count: 0,
        struct_defs_offset: 0,
        struct_defs_count: 0,
        _function_handles_offset: 0,
        _function_handles_count: 0,
        function_defs_offset: 0,
        function_defs_count: 0,
        structs: vec![StructDef {
            name: "Coin".to_string(),
            abilities: AbilitySet {
                key: true,
                store: true,
                copy: true,
                drop: true,
            },
            fields: vec![],
        }],
        functions: vec![FunctionDef {
            name: "publish".to_string(),
            is_public: true,
            is_entry: true,
            parameters: vec![TypeTag::Address, TypeTag::U64],
            returns: vec![],
            locals: vec![TypeTag::Address, TypeTag::U64],
            code: vec![
                MoveOpcode::CopyLoc(0),
                MoveOpcode::CopyLoc(1),
                MoveOpcode::MoveTo(0),
                MoveOpcode::Ret,
            ],
        }],
    };

    let wasm = translate_to_wasm(&module).expect("resource ops should lower");
    assert_valid_wasm(&wasm);
}

/// Test empty module translation
#[test]
fn test_move_to_wasm_empty_module() {
    let module = MoveModule {
        version: BytecodeVersion(6),
        name: "Empty".to_string(),
        identifiers_offset: 0,
        identifiers_count: 0,
        struct_defs_offset: 0,
        struct_defs_count: 0,
        _function_handles_offset: 0,
        _function_handles_count: 0,
        function_defs_offset: 0,
        function_defs_count: 0,
        structs: vec![],
        functions: vec![],
    };

    let wasm = translate_to_wasm(&module).expect("Translation should succeed");
    assert_valid_wasm(&wasm);
}

/// Copying a non-copyable resource should error
#[test]
fn test_copy_of_resource_errors() {
    let module = MoveModule {
        version: BytecodeVersion(6),
        name: "Copy".to_string(),
        identifiers_offset: 0,
        identifiers_count: 0,
        struct_defs_offset: 0,
        struct_defs_count: 0,
        _function_handles_offset: 0,
        _function_handles_count: 0,
        function_defs_offset: 0,
        function_defs_count: 0,
        structs: vec![StructDef {
            name: "Coin".to_string(),
            abilities: AbilitySet {
                key: true,
                store: true,
                copy: false,
                drop: false,
            },
            fields: vec![],
        }],
        functions: vec![FunctionDef {
            name: "bad".to_string(),
            is_public: true,
            is_entry: false,
            parameters: vec![TypeTag::Struct("Coin".to_string())],
            returns: vec![],
            locals: vec![],
            code: vec![MoveOpcode::CopyLoc(0), MoveOpcode::Pop, MoveOpcode::Ret],
        }],
    };

    let err = translate_to_wasm(&module).expect_err("copy should be rejected");
    let msg = format!("{err:#}");
    assert!(msg.contains("copy of resource"));
}

/// Test Move bytecode validation
#[test]
fn test_move_bytecode_validation() {
    use move_neovm::validate_move_bytecode;

    // Valid Move magic
    assert!(validate_move_bytecode(&[
        0xa1, 0x1c, 0xeb, 0x0b, 0x06, 0x00, 0x00, 0x00
    ]));

    // Invalid - WASM magic
    assert!(!validate_move_bytecode(&[0x00, 0x61, 0x73, 0x6d]));

    // Invalid - too short
    assert!(!validate_move_bytecode(&[0xa1, 0x1c]));

    // Invalid - wrong magic
    assert!(!validate_move_bytecode(&[
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
    ]));
}
