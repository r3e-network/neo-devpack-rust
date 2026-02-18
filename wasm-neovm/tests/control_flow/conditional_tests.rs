// Conditional control flow tests (if/else, select)

use wasm_neovm::{opcodes, translate_module};

#[test]
fn translate_complex_if_else_chain() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "classify") (param i32) (result i32)
                local.get 0
                i32.const 10
                i32.lt_s
                if (result i32)
                  local.get 0
                  i32.const 5
                  i32.lt_s
                  if (result i32)
                    i32.const 1
                  else
                    i32.const 2
                  end
                else
                  local.get 0
                  i32.const 20
                  i32.lt_s
                  if (result i32)
                    i32.const 3
                  else
                    local.get 0
                    i32.const 30
                    i32.lt_s
                    if (result i32)
                      i32.const 4
                    else
                      i32.const 5
                    end
                  end
                end)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "IfChain").expect("translation succeeds");

    // Multiple nested if/else should generate multiple jumps
    let jmpifnot = opcodes::lookup("JMPIFNOT_L").unwrap().byte;
    let jmp = opcodes::lookup("JMP_L").unwrap().byte;

    assert!(translation.script.contains(&jmpifnot));
    assert!(translation.script.contains(&jmp));
}

#[test]
fn translate_recursive_structure() {
    let wasm = wat::parse_str(
        r#"(module
              (func $factorial (param i32) (result i32)
                local.get 0
                i32.const 1
                i32.le_s
                if (result i32)
                  i32.const 1
                else
                  local.get 0
                  local.get 0
                  i32.const 1
                  i32.sub
                  call $factorial
                  i32.mul
                end)
              (func (export "main") (result i32)
                i32.const 5
                call $factorial)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Recursive").expect("translation succeeds");

    let call = opcodes::lookup("CALL").unwrap().byte;
    let call_l = opcodes::lookup("CALL_L").unwrap().byte;
    // Should have recursive call to self
    let call_count = translation
        .script
        .iter()
        .filter(|&&b| b == call || b == call_l)
        .count();
    assert!(call_count >= 2, "expected recursive calls");
}

#[test]
fn translate_mutual_recursion() {
    let wasm = wat::parse_str(
        r#"(module
              (func $is_even (param i32) (result i32)
                local.get 0
                i32.const 0
                i32.eq
                if (result i32)
                  i32.const 1
                else
                  local.get 0
                  i32.const 1
                  i32.sub
                  call $is_odd
                end)

              (func $is_odd (param i32) (result i32)
                local.get 0
                i32.const 0
                i32.eq
                if (result i32)
                  i32.const 0
                else
                  local.get 0
                  i32.const 1
                  i32.sub
                  call $is_even
                end)

              (func (export "check_even") (param i32) (result i32)
                local.get 0
                call $is_even)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MutualRecursion").expect("translation succeeds");
    assert_eq!(translation.script.last().copied(), Some(0x40));
}
