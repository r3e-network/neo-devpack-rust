//! End-to-end Move to NEF translation tests

use move_neovm::{
    bytecode::{BytecodeVersion, FunctionDef, MoveModule, MoveOpcode, StructDef, TypeTag},
    translate_to_wasm,
};

/// Test translating a simple Move module to WASM
#[test]
fn test_move_to_wasm_simple_add() {
    let module = MoveModule {
        version: BytecodeVersion(6),
        name: "SimpleAdd".to_string(),
        structs: vec![],
        functions: vec![FunctionDef {
            name: "add".to_string(),
            is_public: true,
            is_entry: false,
            parameters: vec![TypeTag::U64, TypeTag::U64],
            returns: vec![TypeTag::U64],
            code: vec![
                MoveOpcode::CopyLoc(0),
                MoveOpcode::CopyLoc(1),
                MoveOpcode::Add,
                MoveOpcode::Ret,
            ],
        }],
    };

    let wasm = translate_to_wasm(&module).expect("Translation should succeed");

    // Verify WASM magic
    assert_eq!(&wasm[0..4], b"\0asm", "Should have WASM magic");
    // Verify version
    assert_eq!(&wasm[4..8], &[0x01, 0x00, 0x00, 0x00], "Should be WASM version 1");
    // Should have content beyond header
    assert!(wasm.len() > 20, "Should have sections beyond header");
}

/// Test translating a module with multiple functions
#[test]
fn test_move_to_wasm_multiple_functions() {
    let module = MoveModule {
        version: BytecodeVersion(6),
        name: "Math".to_string(),
        structs: vec![],
        functions: vec![
            FunctionDef {
                name: "add".to_string(),
                is_public: true,
                is_entry: false,
                parameters: vec![TypeTag::U64, TypeTag::U64],
                returns: vec![TypeTag::U64],
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
                code: vec![
                    MoveOpcode::CopyLoc(0),
                    MoveOpcode::CopyLoc(1),
                    MoveOpcode::Sub,
                    MoveOpcode::Ret,
                ],
            },
            FunctionDef {
                name: "mul".to_string(),
                is_public: true,
                is_entry: false,
                parameters: vec![TypeTag::U64, TypeTag::U64],
                returns: vec![TypeTag::U64],
                code: vec![
                    MoveOpcode::CopyLoc(0),
                    MoveOpcode::CopyLoc(1),
                    MoveOpcode::Mul,
                    MoveOpcode::Ret,
                ],
            },
        ],
    };

    let wasm = translate_to_wasm(&module).expect("Translation should succeed");

    // Verify basic structure
    assert_eq!(&wasm[0..4], b"\0asm");
    assert!(wasm.len() > 50, "Should have substantial content for 3 functions");
}

/// Test translating a module with entry function
#[test]
fn test_move_to_wasm_entry_function() {
    let module = MoveModule {
        version: BytecodeVersion(6),
        name: "EntryModule".to_string(),
        structs: vec![],
        functions: vec![FunctionDef {
            name: "main".to_string(),
            is_public: false,
            is_entry: true,
            parameters: vec![],
            returns: vec![],
            code: vec![
                MoveOpcode::LdU64(42),
                MoveOpcode::Pop,
                MoveOpcode::Ret,
            ],
        }],
    };

    let wasm = translate_to_wasm(&module).expect("Translation should succeed");
    assert_eq!(&wasm[0..4], b"\0asm");
}

/// Test translating comparison operations
#[test]
fn test_move_to_wasm_comparisons() {
    let module = MoveModule {
        version: BytecodeVersion(6),
        name: "Compare".to_string(),
        structs: vec![],
        functions: vec![FunctionDef {
            name: "less_than".to_string(),
            is_public: true,
            is_entry: false,
            parameters: vec![TypeTag::U64, TypeTag::U64],
            returns: vec![TypeTag::Bool],
            code: vec![
                MoveOpcode::CopyLoc(0),
                MoveOpcode::CopyLoc(1),
                MoveOpcode::Lt,
                MoveOpcode::Ret,
            ],
        }],
    };

    let wasm = translate_to_wasm(&module).expect("Translation should succeed");
    assert_eq!(&wasm[0..4], b"\0asm");
}

/// Test translating control flow
#[test]
fn test_move_to_wasm_control_flow() {
    let module = MoveModule {
        version: BytecodeVersion(6),
        name: "Control".to_string(),
        structs: vec![],
        functions: vec![FunctionDef {
            name: "abs".to_string(),
            is_public: true,
            is_entry: false,
            parameters: vec![TypeTag::U64],
            returns: vec![TypeTag::U64],
            code: vec![
                MoveOpcode::CopyLoc(0),
                MoveOpcode::LdU64(0),
                MoveOpcode::Lt,
                MoveOpcode::BrFalse(2),
                MoveOpcode::CopyLoc(0),
                MoveOpcode::Ret,
            ],
        }],
    };

    let wasm = translate_to_wasm(&module).expect("Translation should succeed");
    assert_eq!(&wasm[0..4], b"\0asm");
}

/// Test translating with resource struct
#[test]
fn test_move_to_wasm_with_resource() {
    let module = MoveModule {
        version: BytecodeVersion(6),
        name: "Coin".to_string(),
        structs: vec![StructDef {
            name: "Coin".to_string(),
            is_resource: true,
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
            code: vec![MoveOpcode::LdU64(100), MoveOpcode::Ret],
        }],
    };

    let wasm = translate_to_wasm(&module).expect("Translation should succeed");
    assert_eq!(&wasm[0..4], b"\0asm");
}

/// Test empty module translation
#[test]
fn test_move_to_wasm_empty_module() {
    let module = MoveModule {
        version: BytecodeVersion(6),
        name: "Empty".to_string(),
        structs: vec![],
        functions: vec![],
    };

    let wasm = translate_to_wasm(&module).expect("Empty module should translate");
    assert_eq!(&wasm[0..4], b"\0asm");
}

/// Test Move bytecode validation
#[test]
fn test_move_bytecode_validation() {
    use move_neovm::validate_move_bytecode;

    // Valid Move magic
    assert!(validate_move_bytecode(&[0xa1, 0x1c, 0xeb, 0x0b, 0x06, 0x00, 0x00, 0x00]));

    // Invalid - WASM magic
    assert!(!validate_move_bytecode(&[0x00, 0x61, 0x73, 0x6d]));

    // Invalid - too short
    assert!(!validate_move_bytecode(&[0xa1, 0x1c]));

    // Invalid - wrong magic
    assert!(!validate_move_bytecode(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]));
}
