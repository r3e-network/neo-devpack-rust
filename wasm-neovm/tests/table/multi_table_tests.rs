// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

// Multiple table support tests

use wasm_neovm::{opcodes, translate_module};

#[test]
fn translate_table_multiple_tables_supported() {
    let wasm = wat::parse_str(
        r#"(module
              (table $t1 5 10 funcref)
              (table $t2 3 8 funcref)

              (func $f1 (result i32) i32.const 1)
              (func $f2 (result i32) i32.const 2)
              (func $f3 (result i32) i32.const 3)

              (elem $t1 (i32.const 0) $f1 $f2)
              (elem $t2 (i32.const 0) $f2 $f3)

              (func (export "sum_sizes") (result i32)
                table.size $t1
                table.size $t2
                i32.add)
            )"#,
    )
    .expect("valid wat");

    let translation =
        translate_module(&wasm, "MultipleTables").expect("translator should accept tables");

    let methods = translation
        .manifest
        .value
        .get("abi")
        .and_then(|abi| abi.get("methods"))
        .and_then(|m| m.as_array())
        .expect("manifest abi methods array");
    assert_eq!(
        methods.len(),
        1,
        "expected single exported method in manifest"
    );

    let call_l = opcodes::lookup("CALL_L").unwrap().byte;
    assert!(
        translation.script.contains(&call_l),
        "expected helper calls for table operations in script"
    );
}

#[test]
fn translate_table_funcref_exports_rejected() {
    let wasm = wat::parse_str(
        r#"(module
              (table 5 funcref)

              (func (export "get_table") (param i32) (result funcref)
                local.get 0
                table.get 0)
            )"#,
    )
    .expect("valid wat");

    let err = translate_module(&wasm, "FuncrefExport")
        .expect_err("translator should reject funcref ABI returns");
    let has_ref_error = err
        .chain()
        .any(|cause| cause.to_string().contains("reference type"));
    assert!(has_ref_error, "unexpected error: {err}");
}

#[test]
fn translate_table_ref_func() {
    let wasm = wat::parse_str(
        r#"(module
              (table 10 funcref)
              (func $target (result i32) i32.const 123)

              (func (export "store_func_ref")
                i32.const 0
                ref.func $target
                table.set 0)

              (func (export "load_func_ref") (result funcref)
                i32.const 0
                table.get 0)
            )"#,
    )
    .expect("valid wat");

    let err = translate_module(&wasm, "RefFunc")
        .expect_err("translator should reject funcref table references");
    let has_ref_error = err
        .chain()
        .any(|cause| cause.to_string().contains("reference type"));
    assert!(has_ref_error, "unexpected error: {err}");
}

#[test]
fn translate_table_ref_eq() {
    let wasm = wat::parse_str(
        r#"(module
              (table 10 funcref)
              (func $f1 (result i32) i32.const 1)
              (func $f2 (result i32) i32.const 2)

              (func (export "compare_refs") (result i32)
                i32.const 0
                table.get 0
                i32.const 1
                table.get 0
                ref.eq
                if (result i32)
                  i32.const 1
                else
                  i32.const 0
                end)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "RefEq").expect("translation succeeds");

    let equal = opcodes::lookup("EQUAL").unwrap().byte;
    assert!(
        translation.script.contains(&equal),
        "ref.eq should emit EQUAL"
    );
}
