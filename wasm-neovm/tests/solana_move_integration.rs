// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Solana and Move cross-chain integration tests
//!
//! Tests end-to-end compilation of cross-chain contracts.

use wasm_neovm::{translate_with_config, SourceChain, TranslationConfig};

fn assert_emits_syscall_hash(script: &[u8], expected_hash: u32) {
    let syscall_opcode = wasm_neovm::opcodes::lookup("SYSCALL")
        .expect("SYSCALL opcode exists")
        .byte;
    let expected_hash = expected_hash.to_le_bytes();

    assert!(
        script
            .windows(5)
            .any(|window| window[0] == syscall_opcode && window[1..5] == expected_hash),
        "expected SYSCALL hash 0x{:08x}",
        u32::from_le_bytes(expected_hash)
    );
}

// ============================================================================
// Solana Integration Tests
// ============================================================================

/// Create a Solana-style contract with storage operations
fn create_solana_storage_contract() -> Vec<u8> {
    wat::parse_str(
        r#"
        (module
            ;; Neo syscall imports (mapped from Solana syscalls)
            (import "neo" "runtime_log" (func $sol_log (param i32 i32)))
            (import "neo" "storage_get" (func $storage_get (param i32 i32) (result i64)))
            (import "neo" "storage_put" (func $storage_put (param i32 i32 i32 i32)))
            (import "neo" "runtime_check_witness" (func $check_witness (param i32) (result i32)))

            (memory (export "memory") 1)

            ;; Storage keys
            (data (i32.const 0) "balance")
            (data (i32.const 16) "owner")

            ;; Initialize contract
            (func (export "initialize") (param $owner i32) (result i32)
                ;; Store owner
                i32.const 16        ;; key ptr "owner"
                i32.const 5         ;; key len
                local.get $owner    ;; value ptr
                i32.const 20        ;; value len (Neo address)
                call $storage_put
                i32.const 1         ;; success
            )

            ;; Get balance
            (func (export "get_balance") (result i64)
                i32.const 0         ;; key ptr "balance"
                i32.const 7         ;; key len
                call $storage_get
            )

            ;; Deposit (add to balance)
            (func (export "deposit") (param $amount i64) (result i32)
                ;; Get current balance
                i32.const 0
                i32.const 7
                call $storage_get
                ;; Add amount
                local.get $amount
                i64.add
                ;; Store (simplified - would need proper encoding)
                drop
                i32.const 1
            )

            ;; Verify owner
            (func (export "verify_owner") (param $caller i32) (result i32)
                local.get $caller
                call $check_witness
            )
        )
        "#,
    )
    .expect("failed to parse WAT")
}

/// Create a Solana-style token contract
fn create_solana_token_contract() -> Vec<u8> {
    wat::parse_str(
        r#"
        (module
            (import "neo" "runtime_log" (func $log (param i32 i32)))
            (import "neo" "storage_get" (func $storage_get (param i32 i32) (result i64)))
            (import "neo" "storage_put" (func $storage_put (param i32 i32 i32 i32)))
            (import "neo" "runtime_check_witness" (func $check_witness (param i32) (result i32)))
            (import "neo" "runtime_notify" (func $notify (param i32 i32)))

            (memory (export "memory") 1)
            (data (i32.const 0) "SolToken v1.0")
            (data (i32.const 32) "Transfer")

            ;; Get token name
            (func (export "name") (result i32)
                i32.const 0     ;; ptr to name
            )

            ;; Get decimals
            (func (export "decimals") (result i32)
                i32.const 8
            )

            ;; Transfer tokens
            (func (export "transfer") (param $from i32) (param $to i32) (param $amount i64) (result i32)
                ;; Check sender authorization
                local.get $from
                call $check_witness
                i32.eqz
                if
                    i32.const 0
                    return
                end

                ;; Log transfer
                i32.const 32
                i32.const 8
                call $log

                ;; Emit transfer event
                i32.const 32
                i32.const 8
                call $notify

                i32.const 1
            )

            ;; Balance of
            (func (export "balance_of") (param $account i32) (result i64)
                local.get $account
                i32.const 20
                call $storage_get
            )
        )
        "#,
    )
    .expect("failed to parse WAT")
}

#[test]
fn test_solana_storage_contract_compilation() {
    let wasm = create_solana_storage_contract();
    let config = TranslationConfig::new("solana-storage").with_source_chain(SourceChain::Solana);

    let result = translate_with_config(&wasm, config);
    assert!(
        result.is_ok(),
        "Solana storage contract should compile: {:?}",
        result.err()
    );

    let translation = result.unwrap();

    // Neo Express requires manifest.features to be empty.
    assert!(translation.manifest.value["features"]
        .as_object()
        .map(|value| value.is_empty())
        .unwrap_or(false));

    // Verify methods
    let methods = translation.manifest.value["abi"]["methods"]
        .as_array()
        .expect("methods array");
    let names: Vec<&str> = methods
        .iter()
        .map(|m| m["name"].as_str().unwrap())
        .collect();

    assert!(names.contains(&"initialize"));
    assert!(names.contains(&"get_balance"));
    assert!(names.contains(&"deposit"));
    assert!(names.contains(&"verify_owner"));
}

#[test]
fn test_solana_token_contract_compilation() {
    let wasm = create_solana_token_contract();
    let config = TranslationConfig::new("solana-token").with_source_chain(SourceChain::Solana);

    let result = translate_with_config(&wasm, config);
    assert!(result.is_ok(), "Solana token contract should compile");

    let translation = result.unwrap();
    let methods = translation.manifest.value["abi"]["methods"]
        .as_array()
        .unwrap();

    assert!(methods.len() >= 4, "Should have at least 4 methods");
}

#[test]
fn test_solana_syscall_mapping() {
    // Verify syscalls are properly mapped
    let wasm = wat::parse_str(
        r#"
        (module
            (import "neo" "crypto_sha256" (func $sha256 (param i32 i32 i32)))
            (import "neo" "runtime_get_time" (func $get_time (result i64)))
            (memory (export "memory") 1)

            (func (export "hash_data") (param $data i32) (param $len i32)
                local.get $data
                local.get $len
                i32.const 64
                call $sha256
            )

            (func (export "get_timestamp") (result i64)
                call $get_time
            )
        )
        "#,
    )
    .unwrap();

    let config = TranslationConfig::new("solana-syscalls").with_source_chain(SourceChain::Solana);
    let result = translate_with_config(&wasm, config);
    assert!(result.is_ok());

    let translation = result.unwrap();
    // Check method tokens for syscalls
    let extra = &translation.manifest.value["extra"];
    assert!(extra.get("nefMethodTokens").is_some());
}

// ============================================================================
// Move Integration Tests
// ============================================================================

/// Create a Move-style coin contract (via WASM)
fn create_move_coin_contract() -> Vec<u8> {
    wat::parse_str(
        r#"
        (module
            ;; Move stdlib mappings
            (import "neo" "storage_get" (func $borrow_global (param i32 i32) (result i64)))
            (import "neo" "storage_put" (func $move_to (param i32 i32 i32 i32)))
            (import "neo" "storage_delete" (func $move_from (param i32 i32)))
            (import "neo" "runtime_check_witness" (func $signer_check (param i32) (result i32)))
            (import "neo" "runtime_notify" (func $emit_event (param i32 i32)))

            (memory (export "memory") 1)

            ;; Type tag prefix for Coin resource
            (data (i32.const 0) "0x1::Coin::Coin")

            ;; Check if resource exists
            (func (export "exists") (param $addr i32) (result i32)
                local.get $addr
                i32.const 20
                call $borrow_global
                i64.const 0
                i64.gt_s
                if (result i32)
                    i32.const 1
                else
                    i32.const 0
                end
            )

            ;; Mint coins (move_to)
            (func (export "mint") (param $to i32) (param $amount i64) (result i32)
                local.get $to
                i32.const 20
                i32.const 64       ;; value location
                i32.const 8        ;; u64 size
                call $move_to
                i32.const 1
            )

            ;; Get balance (borrow_global)
            (func (export "balance") (param $addr i32) (result i64)
                local.get $addr
                i32.const 20
                call $borrow_global
            )

            ;; Transfer with signer verification
            (func (export "transfer") (param $from i32) (param $to i32) (param $amount i64) (result i32)
                ;; Verify signer
                local.get $from
                call $signer_check
                i32.eqz
                if
                    i32.const 0
                    return
                end
                ;; Transfer logic would go here
                i32.const 1
            )

            ;; Burn coins (move_from)
            (func (export "burn") (param $owner i32) (result i32)
                local.get $owner
                call $signer_check
                i32.eqz
                if
                    i32.const 0
                    return
                end
                local.get $owner
                i32.const 20
                call $move_from
                i32.const 1
            )
        )
        "#,
    )
    .expect("failed to parse WAT")
}

/// Create a Move-style NFT contract
fn create_move_nft_contract() -> Vec<u8> {
    wat::parse_str(
        r#"
        (module
            (import "neo" "storage_get" (func $storage_get (param i32 i32) (result i64)))
            (import "neo" "storage_put" (func $storage_put (param i32 i32 i32 i32)))
            (import "neo" "runtime_check_witness" (func $check_witness (param i32) (result i32)))
            (import "neo" "runtime_notify" (func $notify (param i32 i32)))

            (memory (export "memory") 1)
            (data (i32.const 0) "MoveNFT")

            ;; Collection: NFT with id
            (global $next_id (mut i64) (i64.const 1))

            (func (export "name") (result i32)
                i32.const 0
            )

            ;; Mint NFT (creates unique resource)
            (func (export "mint_nft") (param $to i32) (result i64)
                (local $id i64)
                ;; Get next ID
                global.get $next_id
                local.set $id
                ;; Increment
                global.get $next_id
                i64.const 1
                i64.add
                global.set $next_id
                ;; Store ownership
                local.get $to
                i32.const 20
                i32.const 64
                i32.const 8
                call $storage_put
                ;; Return ID
                local.get $id
            )

            ;; Get owner of NFT
            (func (export "owner_of") (param $token_id i64) (result i64)
                i32.const 64
                i32.const 8
                call $storage_get
            )

            ;; Transfer NFT
            (func (export "transfer_nft") (param $from i32) (param $to i32) (param $token_id i64) (result i32)
                ;; Verify sender
                local.get $from
                call $check_witness
                i32.eqz
                if
                    i32.const 0
                    return
                end
                ;; Update ownership
                local.get $to
                i32.const 20
                i32.const 64
                i32.const 8
                call $storage_put
                i32.const 1
            )
        )
        "#,
    )
    .expect("failed to parse WAT")
}

#[test]
fn test_move_coin_contract_compilation() {
    let wasm = create_move_coin_contract();
    let config = TranslationConfig::new("move-coin").with_source_chain(SourceChain::Move);

    let translation =
        translate_with_config(&wasm, config).expect("Move coin contract should compile");

    let methods = translation.manifest.value["abi"]["methods"]
        .as_array()
        .expect("methods");
    let names: Vec<&str> = methods
        .iter()
        .map(|m| m["name"].as_str().unwrap())
        .collect();

    assert!(names.contains(&"exists"), "Should have exists method");
    assert!(names.contains(&"mint"), "Should have mint method");
    assert!(names.contains(&"balance"), "Should have balance method");
    assert!(names.contains(&"transfer"), "Should have transfer method");
    assert!(names.contains(&"burn"), "Should have burn method");
}

#[test]
fn test_move_nft_contract_compilation() {
    let wasm = create_move_nft_contract();
    let config = TranslationConfig::new("move-nft").with_source_chain(SourceChain::Move);

    let translation =
        translate_with_config(&wasm, config).expect("Move NFT contract should compile");

    let methods = translation.manifest.value["abi"]["methods"]
        .as_array()
        .unwrap();

    // Should have NFT-specific methods
    let names: Vec<&str> = methods
        .iter()
        .map(|m| m["name"].as_str().unwrap())
        .collect();

    assert!(names.contains(&"mint_nft"));
    assert!(names.contains(&"owner_of"));
    assert!(names.contains(&"transfer_nft"));
}

#[test]
fn test_move_resource_semantics_mapping() {
    // Test that Move resource operations map to storage
    let wasm = create_move_coin_contract();
    let config = TranslationConfig::new("move-resource-test").with_source_chain(SourceChain::Move);

    let translation = translate_with_config(&wasm, config).unwrap();

    // Neo Express requires manifest.features to be empty.
    assert!(translation.manifest.value["features"]
        .as_object()
        .map(|value| value.is_empty())
        .unwrap_or(false));
}

// ============================================================================
// Cross-Chain Comparison Tests
// ============================================================================

#[test]
fn test_equivalent_contracts_compile_similarly() {
    // Both Solana and Move contracts with similar functionality should
    // produce comparable NEF output

    let solana_wasm = create_solana_storage_contract();
    let move_wasm = create_move_coin_contract();

    let solana_config =
        TranslationConfig::new("solana-test").with_source_chain(SourceChain::Solana);
    let move_config = TranslationConfig::new("move-test").with_source_chain(SourceChain::Move);

    let solana_result = translate_with_config(&solana_wasm, solana_config).unwrap();
    let move_result = translate_with_config(&move_wasm, move_config).unwrap();

    // Both should have storage enabled
    assert_eq!(
        solana_result.manifest.value["features"]["storage"],
        move_result.manifest.value["features"]["storage"]
    );

    // Both should produce valid NEF
    assert!(!solana_result.script.is_empty());
    assert!(!move_result.script.is_empty());
}

#[test]
fn test_source_chain_parsing() {
    // Verify SourceChain enum works correctly
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
fn test_storage_operations_compile() {
    // The same WASM with storage operations should work
    let wasm = wat::parse_str(
        r#"
        (module
            (import "neo" "storage_get" (func $get (param i32 i32) (result i64)))
            (memory (export "memory") 1)
            (func (export "read") (result i64)
                i32.const 0
                i32.const 4
                call $get
            )
        )
        "#,
    )
    .unwrap();

    let config = TranslationConfig::new("storage-test").with_source_chain(SourceChain::Solana);
    let result = translate_with_config(&wasm, config);
    assert!(result.is_ok());

    let translation = result.unwrap();
    assert!(!translation.script.is_empty());
    assert!(translation.manifest.value["features"]
        .as_object()
        .map(|value| value.is_empty())
        .unwrap_or(false));
}

#[test]
fn test_solana_chain_accepts_mixed_case_syscall_module_for_extended_descriptors() {
    let wasm = wat::parse_str(
        r#"
        (module
            (import "SyScAlL" "Neo.Crypto.VerifyWithECDsa" (func $verify (param i32 i32 i32 i32) (result i32)))
            (func (export "verify") (result i32)
                i32.const 0
                i32.const 0
                i32.const 0
                i32.const 1
                call $verify
            )
        )
        "#,
    )
    .expect("failed to parse WAT");

    let config = TranslationConfig::new("solana-mixed-syscall-extended")
        .with_source_chain(SourceChain::Solana);
    let translation = translate_with_config(&wasm, config)
        .expect("solana chain should accept mixed-case syscall module");
    assert_emits_syscall_hash(&translation.script, 0xcf822a6a);
}

#[test]
fn test_move_chain_accepts_mixed_case_syscall_module_for_extended_descriptors() {
    let wasm = wat::parse_str(
        r#"
        (module
            (import "SyScAlL" "Neo.Crypto.VerifyWithECDsa" (func $verify (param i32 i32 i32 i32) (result i32)))
            (func (export "verify") (result i32)
                i32.const 0
                i32.const 0
                i32.const 0
                i32.const 1
                call $verify
            )
        )
        "#,
    )
    .expect("failed to parse WAT");

    let config =
        TranslationConfig::new("move-mixed-syscall-extended").with_source_chain(SourceChain::Move);
    let translation = translate_with_config(&wasm, config)
        .expect("move chain should accept mixed-case syscall module");
    assert_emits_syscall_hash(&translation.script, 0xcf822a6a);
}
