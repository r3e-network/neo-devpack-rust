use wasm_neovm::{opcodes, translate_module, write_nef_with_metadata, MethodToken};

#[test]
fn translate_complete_nep17_token_pattern() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (global $total_supply (mut i64) (i64.const 1000000))
              (table 10 funcref)

              ;; Storage helpers
              (func $get_balance (param i32) (result i64)
                local.get 0
                i64.load)

              (func $set_balance (param i32 i64)
                local.get 0
                local.get 1
                i64.store)

              ;; NEP-17 totalSupply
              (func (export "totalSupply") (result i64)
                global.get $total_supply)

              ;; NEP-17 balanceOf
              (func (export "balanceOf") (param i32) (result i64)
                local.get 0
                call $get_balance)

              ;; NEP-17 transfer
              (func (export "transfer") (param i32 i32 i64) (result i32)
                (local i64 i64)

                ;; Get from balance
                local.get 0
                call $get_balance
                local.set 3

                ;; Check sufficient balance
                local.get 3
                local.get 2
                i64.lt_u
                if
                  i32.const 0
                  return
                end

                ;; Get to balance
                local.get 1
                call $get_balance
                local.set 4

                ;; Update balances
                local.get 0
                local.get 3
                local.get 2
                i64.sub
                call $set_balance

                local.get 1
                local.get 4
                local.get 2
                i64.add
                call $set_balance

                i32.const 1)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "NEP17Token").expect("translation succeeds");

    let call_l = opcodes::lookup("CALL_L").unwrap().byte;
    let ret = opcodes::lookup("RET").unwrap().byte;

    assert!(
        translation.script.contains(&call_l),
        "should have function calls"
    );
    assert!(
        translation.script.contains(&ret),
        "should have return statements"
    );
    let last = translation.script.last().copied();
    assert!(
        matches!(last, Some(0x40) | Some(0x38)),
        "expected NEP-17 script to end in RET or ABORT sentinel, found {:?}",
        last
    );
}

#[test]
fn translate_oracle_consumer_pattern() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (global $request_id (mut i32) (i32.const 0))

              (func $store_data (param i32 i32)
                local.get 0
                local.get 1
                i32.store)

              (func $load_data (param i32) (result i32)
                local.get 0
                i32.load)

              (func (export "requestData") (param i32) (result i32)
                ;; Increment request ID
                global.get $request_id
                i32.const 1
                i32.add
                global.set $request_id

                ;; Store URL pointer
                i32.const 0
                local.get 0
                call $store_data

                ;; Return request ID
                global.get $request_id)

              (func (export "callback") (param i32 i32)
                ;; Store oracle response
                local.get 0
                local.get 1
                call $store_data)

              (func (export "getData") (param i32) (result i32)
                local.get 0
                call $load_data)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "OracleConsumer").expect("translation succeeds");
    let last = translation.script.last().copied();
    assert!(
        matches!(last, Some(0x40) | Some(0x38)),
        "expected oracle consumer script to end in RET or ABORT sentinel, found {:?}",
        last
    );
}

#[test]
fn translate_multi_signature_pattern() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (global $required_sigs (mut i32) (i32.const 2))
              (global $total_sigs (mut i32) (i32.const 3))

              (func $count_signatures (param i32) (result i32)
                (local i32 i32)
                i32.const 0
                local.set 1

                loop $count
                  local.get 1
                  local.get 0
                  i32.ge_u
                  br_if 1

                  local.get 1
                  i32.load8_u
                  i32.const 1
                  i32.eq
                  if
                    local.get 2
                    i32.const 1
                    i32.add
                    local.set 2
                  end

                  local.get 1
                  i32.const 1
                  i32.add
                  local.set 1
                  br $count
                end

                local.get 2)

              (func (export "verify") (param i32) (result i32)
                local.get 0
                call $count_signatures
                global.get $required_sigs
                i32.ge_u)

              (func (export "addSignature") (param i32 i32)
                local.get 0
                i32.const 1
                i32.store8)
            )"#,
    )
    .expect("valid wat");

    let err = translate_module(&wasm, "MultiSig").expect_err("invalid branch should fail");
    let branch_issue = err
        .chain()
        .any(|cause| cause.to_string().contains("branch requires"));
    assert!(
        branch_issue,
        "unexpected multisig branch error: {}",
        err
    );
}

#[test]
fn translate_dao_voting_pattern() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (global $proposal_count (mut i32) (i32.const 0))

              (func $get_votes (param i32) (result i32)
                local.get 0
                i32.const 8
                i32.add
                i32.load)

              (func $get_threshold (param i32) (result i32)
                local.get 0
                i32.const 12
                i32.add
                i32.load)

              (func (export "createProposal") (param i32) (result i32)
                (local i32)

                global.get $proposal_count
                local.set 1

                ;; Store threshold
                local.get 1
                i32.const 12
                i32.add
                local.get 0
                i32.store

                ;; Increment proposal count
                local.get 1
                i32.const 1
                i32.add
                global.set $proposal_count

                local.get 1)

              (func (export "vote") (param i32 i32)
                ;; Add vote
                local.get 0
                i32.const 8
                i32.add
                local.get 0
                call $get_votes
                local.get 1
                i32.add
                i32.store)

              (func (export "checkPassed") (param i32) (result i32)
                local.get 0
                call $get_votes
                local.get 0
                call $get_threshold
                i32.ge_u)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "DAOVoting").expect("translation succeeds");
    let last = translation.script.last().copied();
    assert!(
        matches!(last, Some(0x40) | Some(0x38)),
        "expected DAO voting script to end in RET or ABORT sentinel, found {:?}",
        last
    );
}

#[test]
fn translate_nft_contract_pattern() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (global $next_token_id (mut i32) (i32.const 1))

              (func $get_owner (param i32) (result i32)
                local.get 0
                i32.const 4
                i32.mul
                i32.load)

              (func $set_owner (param i32 i32)
                local.get 0
                i32.const 4
                i32.mul
                local.get 1
                i32.store)

              (func (export "mint") (param i32) (result i32)
                (local i32)

                ;; Get current token ID
                global.get $next_token_id
                local.set 1

                ;; Set owner
                local.get 1
                local.get 0
                call $set_owner

                ;; Increment token ID
                local.get 1
                i32.const 1
                i32.add
                global.set $next_token_id

                local.get 1)

              (func (export "ownerOf") (param i32) (result i32)
                local.get 0
                call $get_owner)

              (func (export "transfer") (param i32 i32 i32) (result i32)
                ;; Check current owner
                local.get 0
                call $get_owner
                local.get 1
                i32.ne
                if
                  i32.const 0
                  return
                end

                ;; Transfer ownership
                local.get 0
                local.get 2
                call $set_owner

                i32.const 1)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "NFTContract").expect("translation succeeds");

    let call_l = opcodes::lookup("CALL_L").unwrap().byte;
    assert!(
        translation.script.contains(&call_l),
        "should have function calls"
    );
}

#[test]
fn translate_escrow_contract_pattern() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (global $state (mut i32) (i32.const 0))

              (func (export "create") (param i32 i32 i64)
                ;; Store buyer
                i32.const 0
                local.get 0
                i32.store

                ;; Store seller
                i32.const 4
                local.get 1
                i32.store

                ;; Store amount
                i32.const 8
                local.get 2
                i64.store

                ;; Set state to pending
                i32.const 1
                global.set $state)

              (func (export "complete") (result i32)
                global.get $state
                i32.const 1
                i32.ne
                if
                  i32.const 0
                  return
                end

                ;; Set state to completed
                i32.const 2
                global.set $state

                i32.const 1)

              (func (export "cancel") (result i32)
                global.get $state
                i32.const 1
                i32.ne
                if
                  i32.const 0
                  return
                end

                ;; Set state to cancelled
                i32.const 3
                global.set $state

                i32.const 1)

              (func (export "getState") (result i32)
                global.get $state)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Escrow").expect("translation succeeds");
    let last = translation.script.last().copied();
    assert!(
        matches!(last, Some(0x40) | Some(0x38)),
        "expected escrow script to end in RET or ABORT sentinel, found {:?}",
        last
    );
}

#[test]
fn translate_full_nef_generation_with_metadata() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "add") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add)

              (func (export "multiply") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.mul)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Calculator").expect("translation succeeds");

    // Create temporary file for NEF output
    let temp_dir = std::env::temp_dir();
    let nef_path = temp_dir.join("test_calculator.nef");
    let token = MethodToken {
        contract_hash: [0x11; 20],
        method: "add".to_string(),
        parameters_count: 2,
        has_return_value: true,
        call_flags: 0x01,
    };
    write_nef_with_metadata(
        &translation.script,
        Some("ipfs://calculator"),
        &[token.clone()],
        &nef_path,
    )
    .expect("NEF generation succeeds");

    // Verify NEF file was created
    assert!(nef_path.exists(), "NEF file should be created");
    let nef_output = std::fs::read(&nef_path).expect("Read NEF file");
    assert!(!nef_output.is_empty());
    assert!(nef_output.len() > 100, "NEF should have reasonable size");

    // Clean up
    let _ = std::fs::remove_file(nef_path);
}

#[test]
fn translate_complex_state_machine() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (global $state (mut i32) (i32.const 0))

              (func (export "transition") (param i32) (result i32)
                global.get $state
                i32.const 0
                i32.eq
                if
                  local.get 0
                  i32.const 1
                  i32.eq
                  if
                    i32.const 1
                    global.set $state
                    i32.const 1
                    return
                  end
                end

                global.get $state
                i32.const 1
                i32.eq
                if
                  local.get 0
                  i32.const 2
                  i32.eq
                  if
                    i32.const 2
                    global.set $state
                    i32.const 1
                    return
                  end
                end

                global.get $state
                i32.const 2
                i32.eq
                if
                  local.get 0
                  i32.const 0
                  i32.eq
                  if
                    i32.const 0
                    global.set $state
                    i32.const 1
                    return
                  end
                end

                i32.const 0)

              (func (export "getState") (result i32)
                global.get $state)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "StateMachine").expect("translation succeeds");

    let jmpifnot = opcodes::lookup("JMPIFNOT_L").unwrap().byte;
    assert!(
        translation.script.contains(&jmpifnot),
        "should have conditional branches"
    );
}
