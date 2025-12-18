use wasm_neovm::translate_module;

#[test]
fn translate_block_with_branches() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "branch") (param i32)
                block
                  local.get 0
                  br_if 0
                  br 0
                end)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Branch").expect("translation succeeds");
    assert_eq!(translation.script.last().copied(), Some(0x40));
}

#[test]
fn translate_loop_with_back_edge() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "loop")
                loop
                  br 0
                end)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Loop").expect("translation succeeds");
    assert_eq!(translation.script.last().copied(), Some(0x40));
}

#[test]
fn translate_if_else_structure() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "cond") (param i32)
                block
                  local.get 0
                  if
                    i32.const 1
                    drop
                  else
                    i32.const 2
                    drop
                  end
                end)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Cond").expect("translation succeeds");
    assert_eq!(translation.script.last().copied(), Some(0x40));
}

#[test]
fn translate_br_table_dynamic() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "dispatch") (param i32)
                block
                  block
                    local.get 0
                    br_table 1 0
                  end
                end)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Dispatch").expect("translation succeeds");

    let dup = wasm_neovm::opcodes::lookup("DUP").unwrap().byte;
    let jmp_if = wasm_neovm::opcodes::lookup("JMPIF_L").unwrap().byte;
    let drop = wasm_neovm::opcodes::lookup("DROP").unwrap().byte;

    assert!(translation.script.contains(&dup));
    assert!(translation.script.contains(&jmp_if));

    // Ensure there is at least one DROP to clear the index before branching.
    assert!(translation.script.iter().filter(|&&b| b == drop).count() >= 2);
}

#[test]
fn translate_br_table_constant_index() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "dispatch_const")
                block
                  block
                    i32.const 2
                    br_table 1 0
                  end
                end)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "DispatchConst").expect("translation succeeds");

    let dup = wasm_neovm::opcodes::lookup("DUP").unwrap().byte;
    assert!(
        !translation.script.contains(&dup),
        "constant br_table should not emit DUP comparisons"
    );
}
