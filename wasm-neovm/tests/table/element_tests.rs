// Table element segment tests (elem, table.init, elem.drop)

use wasm_neovm::{opcodes, translate_module};

#[test]
fn translate_table_declared_segment_ignored() {
    let wasm = wat::parse_str(
        r#"(module
              (table 5 funcref)
              (elem declare funcref)
              (func (export "size") (result i32)
                table.size 0)
            )"#,
    )
    .expect("valid wat");

    let err = translate_module(&wasm, "TableDeclared")
        .expect_err("declared element segments are not supported");
    let has_declared_error = err
        .chain()
        .any(|cause| cause.to_string().contains("declared element segment"));
    assert!(has_declared_error, "unexpected error: {err}");
}

#[test]
fn translate_table_element_negative_offset_fails_with_bounds_error() {
    let wasm = wat::parse_str(
        r#"(module
              (table 5 funcref)
              (func $f0 (result i32) i32.const 0)
              (elem (i32.const -1) $f0)
              (func (export "size") (result i32)
                table.size 0)
            )"#,
    )
    .expect("valid wat");

    let err = translate_module(&wasm, "TableElemNegOffset")
        .expect_err("element segment offset should be interpreted as u32 and trap on table bounds");
    let reports_bounds = err
        .chain()
        .any(|cause| cause.to_string().contains("writes past table bounds"));
    assert!(reports_bounds, "unexpected error: {err}");

    let reports_negative = err
        .chain()
        .any(|cause| cause.to_string().contains("must be non-negative"));
    assert!(
        !reports_negative,
        "unexpected negative-offset rejection: {err}"
    );
}

#[test]
fn translate_table_init_and_drop() {
    let wasm = wat::parse_str(
        r#"(module
              (table 10 funcref)

              (func $f1 (result i32) i32.const 1)
              (func $f2 (result i32) i32.const 2)
              (func $f3 (result i32) i32.const 3)

              (elem $segment funcref (ref.func $f1) (ref.func $f2) (ref.func $f3))

              (func (export "init_and_drop")
                i32.const 0
                i32.const 0
                i32.const 3
                table.init 0 $segment

                elem.drop $segment)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "TableInitDrop").expect("translation succeeds");

    let call_l = opcodes::lookup("CALL_L").unwrap().byte;
    assert!(
        translation.script.contains(&call_l),
        "table.init should use helper"
    );
}

#[test]
fn translate_table_init_allows_len_reaching_end_of_table() {
    let wasm = wat::parse_str(
        r#"(module
              (table 3 funcref)

              (func $f1 (result i32) i32.const 1)
              (func $f2 (result i32) i32.const 2)
              (func $f3 (result i32) i32.const 3)

              (elem $segment funcref (ref.func $f1) (ref.func $f2) (ref.func $f3))

              (func (export "init_all")
                i32.const 0
                i32.const 0
                i32.const 3
                table.init 0 $segment)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "TableInitEnd").expect("translation succeeds");

    let size = opcodes::lookup("SIZE").unwrap().byte;
    let gt = opcodes::lookup("GT").unwrap().byte;
    let jmpif_l = opcodes::lookup("JMPIF_L").unwrap().byte;
    assert!(
        translation
            .script
            .windows(3)
            .any(|window| window == [size, gt, jmpif_l]),
        "expected table.init bounds check to trap only when end > size"
    );
}
