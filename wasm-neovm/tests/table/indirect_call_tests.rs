// Table indirect call tests

use wasm_neovm::{opcodes, translate_module};

#[test]
fn translate_table_indirect_calls() {
    let wasm = wat::parse_str(
        r#"(module
              (type $sig (func (param i32) (result i32)))
              (table 10 funcref)
              (elem (i32.const 0) $add_one $add_two $add_three)

              (func $add_one (param i32) (result i32)
                local.get 0
                i32.const 1
                i32.add)

              (func $add_two (param i32) (result i32)
                local.get 0
                i32.const 2
                i32.add)

              (func $add_three (param i32) (result i32)
                local.get 0
                i32.const 3
                i32.add)

              (func (export "dispatch") (param i32 i32) (result i32)
                local.get 1
                local.get 0
                call_indirect (type $sig))
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "IndirectCalls").expect("translation succeeds");

    let call_indirect = opcodes::lookup("CALL_L").unwrap().byte;
    assert!(
        translation.script.contains(&call_indirect),
        "expected indirect call instruction"
    );
}

#[test]
fn translate_table_complex_dispatch() {
    let wasm = wat::parse_str(
        r#"(module
              (type $bin_op (func (param i32 i32) (result i32)))
              (table 8 funcref)

              (func $add (param i32 i32) (result i32)
                local.get 0 local.get 1 i32.add)
              (func $sub (param i32 i32) (result i32)
                local.get 0 local.get 1 i32.sub)
              (func $mul (param i32 i32) (result i32)
                local.get 0 local.get 1 i32.mul)
              (func $div (param i32 i32) (result i32)
                local.get 0 local.get 1 i32.div_s)
              (func $mod (param i32 i32) (result i32)
                local.get 0 local.get 1 i32.rem_s)

              (elem (i32.const 0) $add $sub $mul $div $mod)

              (func (export "calc") (param i32 i32 i32) (result i32)
                local.get 1
                local.get 2
                local.get 0
                call_indirect (type $bin_op))
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ComplexDispatch").expect("translation succeeds");
    let last = translation.script.last().copied();
    assert!(
        matches!(last, Some(0x40) | Some(0x38)),
        "expected complex dispatch script to end with RET or ABORT sentinel, found {:?}",
        last
    );
}
