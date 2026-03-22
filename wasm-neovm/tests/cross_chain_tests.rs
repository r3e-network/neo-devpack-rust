// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Cross-chain compilation integration tests
//!
//! Tests end-to-end compilation of contracts from different source chains.

use wasm_neovm::{translate_with_config, SourceChain, TranslationConfig};

/// Helper to create a minimal Solana-style WASM module that uses Neo syscalls
fn create_solana_hello_wasm() -> Vec<u8> {
    // Minimal WASM module with Neo import for runtime_log
    wat::parse_str(
        r#"
        (module
            (import "neo" "runtime_log" (func $log (param i32 i32)))
            (memory (export "memory") 1)
            (data (i32.const 0) "Hello from Solana!")

            (func (export "hello")
                i32.const 0      ;; message ptr
                i32.const 18     ;; message len
                call $log
            )

            (func (export "get_value") (result i32)
                i32.const 42
            )
        )
        "#,
    )
    .expect("failed to parse WAT")
}

/// Helper to create a WASM module that uses storage
fn create_storage_wasm() -> Vec<u8> {
    wat::parse_str(
        r#"
        (module
            (import "neo" "storage_get" (func $storage_get (param i32 i32) (result i64)))
            (import "neo" "storage_put" (func $storage_put (param i32 i32 i32 i32)))
            (memory (export "memory") 1)
            (data (i32.const 0) "key")
            (data (i32.const 16) "value")

            (func (export "store")
                i32.const 0      ;; key ptr
                i32.const 3      ;; key len
                i32.const 16     ;; value ptr
                i32.const 5      ;; value len
                call $storage_put
            )

            (func (export "load") (result i64)
                i32.const 0      ;; key ptr
                i32.const 3      ;; key len
                call $storage_get
            )
        )
        "#,
    )
    .expect("failed to parse WAT")
}

/// Helper to create a WASM module that uses crypto functions
fn create_crypto_wasm() -> Vec<u8> {
    wat::parse_str(
        r#"
        (module
            (import "neo" "crypto_sha256" (func $sha256 (param i32 i32 i32)))
            (import "neo" "runtime_check_witness" (func $check_witness (param i32) (result i32)))
            (memory (export "memory") 1)
            (data (i32.const 0) "test data")

            (func (export "hash")
                i32.const 0      ;; data ptr
                i32.const 9      ;; data len
                i32.const 64     ;; output ptr
                call $sha256
            )

            (func (export "verify") (result i32)
                i32.const 0      ;; hash ptr (20 bytes)
                call $check_witness
            )
        )
        "#,
    )
    .expect("failed to parse WAT")
}

#[test]
fn test_solana_hello_compilation() {
    let wasm = create_solana_hello_wasm();
    let config = TranslationConfig::new("solana-hello");

    let translation = translate_with_config(&wasm, config).expect("translation should succeed");

    // Verify NEF was generated
    assert!(!translation.script.is_empty(), "script should not be empty");

    // Verify manifest has the expected methods
    let methods = translation.manifest.value["abi"]["methods"]
        .as_array()
        .expect("methods should be an array");

    let method_names: Vec<&str> = methods
        .iter()
        .map(|m| m["name"].as_str().unwrap())
        .collect();

    assert!(method_names.contains(&"hello"), "should have hello method");
    assert!(
        method_names.contains(&"get_value"),
        "should have get_value method"
    );
}

#[test]
fn test_storage_contract_compilation() {
    let wasm = create_storage_wasm();
    let config = TranslationConfig::new("storage-test");

    let translation = translate_with_config(&wasm, config).expect("translation should succeed");

    // Neo Express requires manifest.features to be an empty object.
    let features = &translation.manifest.value["features"];
    assert!(
        features
            .as_object()
            .map(|value| value.is_empty())
            .unwrap_or(false),
        "features should be an empty object for Neo Express compatibility"
    );

    // Verify methods
    let methods = translation.manifest.value["abi"]["methods"]
        .as_array()
        .expect("methods should be an array");

    assert_eq!(methods.len(), 2, "should have 2 methods");
}

#[test]
fn test_crypto_contract_compilation() {
    let wasm = create_crypto_wasm();
    let config = TranslationConfig::new("crypto-test");

    let translation = translate_with_config(&wasm, config).expect("translation should succeed");

    // Verify methods exist
    let methods = translation.manifest.value["abi"]["methods"]
        .as_array()
        .expect("methods should be an array");

    let method_names: Vec<&str> = methods
        .iter()
        .map(|m| m["name"].as_str().unwrap())
        .collect();

    assert!(method_names.contains(&"hash"), "should have hash method");
    assert!(
        method_names.contains(&"verify"),
        "should have verify method"
    );
}

#[test]
fn test_source_chain_parsing() {
    assert_eq!(SourceChain::from_str("neo"), Some(SourceChain::Neo));
    assert_eq!(SourceChain::from_str("native"), Some(SourceChain::Neo));
    assert_eq!(SourceChain::from_str("solana"), Some(SourceChain::Solana));
    assert_eq!(SourceChain::from_str("sol"), Some(SourceChain::Solana));
    assert_eq!(SourceChain::from_str("move"), Some(SourceChain::Move));
    assert_eq!(SourceChain::from_str("aptos"), Some(SourceChain::Move));
    assert_eq!(SourceChain::from_str("sui"), Some(SourceChain::Move));
    assert_eq!(SourceChain::from_str("unknown"), None);
}

#[test]
fn test_manifest_method_tokens_generated() {
    let wasm = create_solana_hello_wasm();
    let config = TranslationConfig::new("token-test");

    let translation = translate_with_config(&wasm, config).expect("translation should succeed");

    // Check nefMethodTokens in extra
    let extra = &translation.manifest.value["extra"];
    let tokens = extra["nefMethodTokens"].as_array();

    assert!(tokens.is_some(), "should have method tokens");
    let tokens = tokens.unwrap();

    // Should have at least one token for System.Runtime.Log
    let has_log_token = tokens
        .iter()
        .any(|t| t["method"].as_str() == Some("System.Runtime.Log"));

    assert!(has_log_token, "should have System.Runtime.Log token");
}

#[test]
fn test_multiple_exports_preserved() {
    let wasm = wat::parse_str(
        r#"
        (module
            (func $impl (result i32)
                i32.const 100
            )
            (export "method_a" (func $impl))
            (export "method_b" (func $impl))
            (export "method_c" (func $impl))
        )
        "#,
    )
    .expect("failed to parse WAT");

    let config = TranslationConfig::new("multi-export");
    let translation = translate_with_config(&wasm, config).expect("translation should succeed");

    let methods = translation.manifest.value["abi"]["methods"]
        .as_array()
        .expect("methods should be an array");

    assert_eq!(methods.len(), 3, "should have 3 exported methods");
}

#[test]
fn test_return_type_detection() {
    let wasm = wat::parse_str(
        r#"
        (module
            (func (export "void_method"))
            (func (export "int_method") (result i32) i32.const 0)
            (func (export "long_method") (result i64) i64.const 0)
        )
        "#,
    )
    .expect("failed to parse WAT");

    let config = TranslationConfig::new("return-types");
    let translation = translate_with_config(&wasm, config).expect("translation should succeed");

    let methods = translation.manifest.value["abi"]["methods"]
        .as_array()
        .expect("methods should be an array");

    for method in methods {
        let name = method["name"].as_str().unwrap();
        let return_type = method["returntype"].as_str().unwrap();

        match name {
            "void_method" => assert_eq!(return_type, "Void"),
            "int_method" | "long_method" => assert_eq!(return_type, "Integer"),
            _ => panic!("unexpected method: {}", name),
        }
    }
}

#[test]
fn test_solana_adapter_maps_syscalls() {
    let wasm = wat::parse_str(
        r#"
        (module
            (import "solana" "sol_log" (func $log (param i32 i32)))
            (memory (export "memory") 1)
            (data (i32.const 0) "hi")
            (func (export "entry")
                i32.const 0
                i32.const 2
                call $log
            )
        )
        "#,
    )
    .expect("failed to parse WAT");

    let config = TranslationConfig::new("sol-log").with_source_chain(SourceChain::Solana);
    let translation = translate_with_config(&wasm, config)
        .expect("translation should succeed via Solana adapter");

    let tokens = translation.manifest.value["extra"]["nefMethodTokens"]
        .as_array()
        .expect("nefMethodTokens should be present");

    let has_log = tokens
        .iter()
        .any(|t| t["method"].as_str() == Some("System.Runtime.Log"));
    assert!(
        has_log,
        "System.Runtime.Log token should be emitted for sol_log import"
    );
}

#[test]
fn test_move_source_chain_supported_for_basic_wasm() {
    let wasm = wat::parse_str(
        r#"
        (module
            (import "move_stdlib" "hash_sha256" (func $hash (param i32 i32 i32)))
            (memory (export "memory") 1)
            (data (i32.const 0) "data")
            (func (export "noop")
                i32.const 0
                i32.const 4
                i32.const 32
                call $hash
            )
        )
        "#,
    )
    .expect("failed to parse WAT");

    let config = TranslationConfig::new("move-basic").with_source_chain(SourceChain::Move);
    let translation =
        translate_with_config(&wasm, config).expect("Move source chain should be translated");

    assert!(!translation.script.is_empty(), "script should not be empty");
}

#[test]
fn test_move_resource_import_enables_storage_feature() {
    let wasm = wat::parse_str(
        r#"
        (module
            (import "move_resource" "move_to" (func $move_to (param i32 i32)))
            (memory (export "memory") 1)
            (data (i32.const 0) "\01\02\03\04\05\06\07\08\09\0a\0b\0c\0d\0e\0f\10\11\12\13\14\15\16\17\18\19\1a\1b\1c\1d\1e\1f\20")
            (func (export "publish")
                i32.const 0   ;; key ptr
                i32.const 32  ;; key len
                call $move_to
            )
        )
        "#,
    )
    .expect("failed to parse WAT");

    let config = TranslationConfig::new("move-resource").with_source_chain(SourceChain::Move);
    let translation =
        translate_with_config(&wasm, config).expect("Move resource imports should translate");

    assert!(
        translation.manifest.value["features"]
            .as_object()
            .map(|value| value.is_empty())
            .unwrap_or(false),
        "features should be an empty object for Neo Express compatibility"
    );
}

/// Move bytecode (.mv) inputs should be auto-translated via move-neovm
#[test]
fn test_move_bytecode_input_translates() {
    // magic + version + table_count=0 + code: LdU8 7, Ret
    let mv: Vec<u8> = vec![
        0xa1, 0x1c, 0xeb, 0x0b, // magic
        0x06, 0x00, 0x00, 0x00, // version
        0x00, // table count
        0x06, 0x07, // LdU8 7
        0x02, // Ret
    ];

    let wasm = move_neovm::translate_move_to_wasm(&mv, "mv-bytecode")
        .expect("move-neovm should lower bytecode")
        .wasm;

    let config = TranslationConfig::new("mv-bytecode").with_source_chain(SourceChain::Move);
    let translation =
        translate_with_config(&wasm, config).expect("Move bytecode input should translate");
    assert!(
        !translation.script.is_empty(),
        "translation script should be produced"
    );
}

#[test]
fn test_move_stdlib_hash_maps_to_neo_crypto() {
    let wasm = wat::parse_str(
        r#"
        (module
            (import "move_stdlib" "hash_sha256" (func $hash (param i32 i32 i32)))
            (memory (export "memory") 1)
            (data (i32.const 0) "data")
            (func (export "hash")
                i32.const 0      ;; data ptr
                i32.const 4      ;; data len
                i32.const 64     ;; out ptr
                call $hash
            )
        )
        "#,
    )
    .expect("failed to parse WAT");

    let config = TranslationConfig::new("move-hash").with_source_chain(SourceChain::Move);
    let translation =
        translate_with_config(&wasm, config).expect("Move stdlib hash should translate");

    // Ensure manifest has the exported method
    let methods = translation.manifest.value["abi"]["methods"]
        .as_array()
        .expect("methods array");
    let names: Vec<&str> = methods
        .iter()
        .map(|m| m["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"hash"), "hash method should be exported");
}
