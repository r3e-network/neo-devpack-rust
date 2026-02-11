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
fn translate_table_grow() {
    let wasm = wat::parse_str(
        r#"(module
              (table 5 10 funcref)

              (func (export "grow_test") (result i32)
                table.size 0
                ref.null func
                i32.const 3
                table.grow 0
                drop
                table.size 0)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "TableGrow").expect("translation succeeds");

    let call_l = opcodes::lookup("CALL_L").unwrap().byte;
    assert!(
        translation.script.contains(&call_l),
        "table.grow should use helper"
    );
}

#[test]
fn translate_table_grow_failure_path() {
    let wasm = wat::parse_str(
        r#"(module
              (table 1 1 funcref)
              (func (export "grow_fail") (result i32)
                ref.null func
                i32.const 2
                table.grow 0)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "TableGrowFail").expect("translation succeeds");

    let pushm1 = opcodes::lookup("PUSHM1").unwrap().byte;
    assert!(
        translation.script.contains(&pushm1),
        "table.grow helper should provide -1 failure return"
    );

    let append = opcodes::lookup("APPEND").unwrap().byte;
    assert!(
        translation.script.contains(&append),
        "table.grow helper should append new entries when growth succeeds"
    );
}

#[test]
fn translate_table_fill() {
    let wasm = wat::parse_str(
        r#"(module
              (table 10 funcref)
              (func $dummy (result i32) i32.const 42)

              (func (export "fill_table")
                i32.const 0
                ref.func $dummy
                i32.const 5
                table.fill 0)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "TableFill").expect("translation succeeds");

    let call_l = opcodes::lookup("CALL_L").unwrap().byte;
    assert!(
        translation.script.contains(&call_l),
        "table.fill should use helper"
    );
}

#[test]
fn translate_table_fill_allows_len_reaching_end_of_table() {
    let wasm = wat::parse_str(
        r#"(module
              (table 5 funcref)
              (func $dummy (result i32) i32.const 42)

              (func (export "fill_all")
                i32.const 0
                ref.func $dummy
                i32.const 5
                table.fill 0)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "TableFillEnd").expect("translation succeeds");

    let size = opcodes::lookup("SIZE").unwrap().byte;
    let gt = opcodes::lookup("GT").unwrap().byte;
    let jmpif_l = opcodes::lookup("JMPIF_L").unwrap().byte;
    assert!(
        translation
            .script
            .windows(3)
            .any(|window| window == [size, gt, jmpif_l]),
        "expected table.fill bounds check to trap only when dest+len > size"
    );
}

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
        .expect_err("element segment negative offset should be rejected");

    let reports_negative = err
        .chain()
        .any(|cause| cause.to_string().contains("must be non-negative"));
    assert!(reports_negative, "unexpected error: {err}");
}

#[test]
fn translate_table_copy_overlap_safe() {
    let wasm = wat::parse_str(
        r#"(module
              (table 5 funcref)
              (func $f0 (result i32) i32.const 0)
              (func $f1 (result i32) i32.const 1)
              (elem (i32.const 0) $f0 $f1)

              (func (export "copy_overlap")
                i32.const 1
                i32.const 0
                i32.const 2
                table.copy 0 0)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "TableCopyOverlap").expect("translation succeeds");

    let newarray0 = opcodes::lookup("NEWARRAY0").unwrap().byte;
    let append = opcodes::lookup("APPEND").unwrap().byte;
    assert!(
        translation.script.contains(&newarray0),
        "table.copy helper should allocate a temporary buffer"
    );
    assert!(
        translation.script.iter().filter(|&&b| b == append).count() >= 1,
        "table.copy helper should stage entries via APPEND"
    );
}

#[test]
fn translate_table_copy() {
    let wasm = wat::parse_str(
        r#"(module
              (table $src 10 funcref)
              (table $dst 10 funcref)

              (func $f1 (result i32) i32.const 1)
              (func $f2 (result i32) i32.const 2)

              (elem $src (i32.const 0) $f1 $f2)

              (func (export "copy_tables")
                i32.const 0
                i32.const 0
                i32.const 2
                table.copy $dst $src)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "TableCopy").expect("translation succeeds");

    let call_l = opcodes::lookup("CALL_L").unwrap().byte;
    assert!(
        translation.script.contains(&call_l),
        "table.copy should use helper"
    );
}

#[test]
fn translate_table_copy_allows_len_reaching_end_of_table() {
    let wasm = wat::parse_str(
        r#"(module
              (table $src 2 funcref)
              (table $dst 2 funcref)

              (func $f1 (result i32) i32.const 1)
              (func $f2 (result i32) i32.const 2)

              (elem $src (i32.const 0) $f1 $f2)

              (func (export "copy_all")
                i32.const 0
                i32.const 0
                i32.const 2
                table.copy $dst $src)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "TableCopyEnd").expect("translation succeeds");

    let size = opcodes::lookup("SIZE").unwrap().byte;
    let gt = opcodes::lookup("GT").unwrap().byte;
    let jmpif_l = opcodes::lookup("JMPIF_L").unwrap().byte;
    assert!(
        translation
            .script
            .windows(3)
            .any(|window| window == [size, gt, jmpif_l]),
        "expected table.copy bounds check to trap only when end > size"
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

#[test]
fn translate_table_bounds_checking() {
    let wasm = wat::parse_str(
        r#"(module
              (table 5 funcref)
              (func $safe (result i32) i32.const 42)

              (elem (i32.const 0) $safe)

              (func (export "test_bounds") (param i32) (result funcref)
                local.get 0
                table.get 0)
            )"#,
    )
    .expect("valid wat");

    let err = translate_module(&wasm, "TableBounds")
        .expect_err("translator should reject funcref table bounds operations");
    let has_ref_error = err
        .chain()
        .any(|cause| cause.to_string().contains("reference type"));
    assert!(has_ref_error, "unexpected error: {err}");
}

#[test]
fn translate_table_null_handling() {
    let wasm = wat::parse_str(
        r#"(module
              (table 10 funcref)

              (func (export "set_null")
                i32.const 0
                ref.null func
                table.set 0)

              (func (export "check_null") (param i32) (result i32)
                local.get 0
                table.get 0
                ref.is_null
                if (result i32)
                  i32.const 1
                else
                  i32.const 0
                end)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "TableNull").expect("translation succeeds");
    let last = translation.script.last().copied();
    assert!(
        matches!(last, Some(0x40) | Some(0x38)),
        "expected table null handling to end with RET or ABORT sentinel, found {:?}",
        last
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

#[test]
fn translate_table_dynamic_resize() {
    let wasm = wat::parse_str(
        r#"(module
              (table 1 100 funcref)
              (func $dummy (result i32) i32.const 42)

              (func (export "dynamic_grow") (param i32) (result i32)
                (local i32)
                table.size 0
                local.set 1

                ref.null func
                local.get 0
                table.grow 0
                drop

                table.size 0
                local.get 1
                i32.sub)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "DynamicResize").expect("translation succeeds");
    assert_eq!(translation.script.last().copied(), Some(0x40));
}
