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

    let translation = translate_with_config(&wasm, config)
        .expect("translation should succeed");

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
    assert!(method_names.contains(&"get_value"), "should have get_value method");
}

#[test]
fn test_storage_contract_compilation() {
    let wasm = create_storage_wasm();
    let config = TranslationConfig::new("storage-test");

    let translation = translate_with_config(&wasm, config)
        .expect("translation should succeed");

    // Verify storage feature is enabled
    let features = &translation.manifest.value["features"];
    assert_eq!(
        features["storage"].as_bool(),
        Some(true),
        "storage feature should be enabled"
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

    let translation = translate_with_config(&wasm, config)
        .expect("translation should succeed");

    // Verify methods exist
    let methods = translation.manifest.value["abi"]["methods"]
        .as_array()
        .expect("methods should be an array");

    let method_names: Vec<&str> = methods
        .iter()
        .map(|m| m["name"].as_str().unwrap())
        .collect();

    assert!(method_names.contains(&"hash"), "should have hash method");
    assert!(method_names.contains(&"verify"), "should have verify method");
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

    let translation = translate_with_config(&wasm, config)
        .expect("translation should succeed");

    // Check nefMethodTokens in extra
    let extra = &translation.manifest.value["extra"];
    let tokens = extra["nefMethodTokens"].as_array();

    assert!(tokens.is_some(), "should have method tokens");
    let tokens = tokens.unwrap();

    // Should have at least one token for System.Runtime.Log
    let has_log_token = tokens.iter().any(|t| {
        t["method"].as_str() == Some("System.Runtime.Log")
    });

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
    let translation = translate_with_config(&wasm, config)
        .expect("translation should succeed");

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
    let translation = translate_with_config(&wasm, config)
        .expect("translation should succeed");

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
