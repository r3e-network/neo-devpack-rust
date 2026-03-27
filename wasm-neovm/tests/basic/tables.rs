// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use wasm_neovm::translate_module;

#[test]
fn translate_table_helpers_cover_operations() {
    use std::borrow::Cow;
    use wasm_encoder::{
        CodeSection, ElementSection, Elements, ExportKind, ExportSection, Function,
        FunctionSection, HeapType, Module, RefType, TableSection, TableType, TypeSection, ValType,
    };

    let mut module = Module::new();

    let mut types = TypeSection::new();
    types.ty().function(vec![], vec![]); // type 0: [] -> []
    types.ty().function(vec![], vec![ValType::I32]); // type 1
    types.ty().function(vec![], vec![ValType::I32]); // type 2
    module.section(&types);

    let mut functions = FunctionSection::new();
    functions.function(0); // $target
    functions.function(1); // ops
    functions.function(2); // size
    functions.function(0); // drop (reuse type 0)
    module.section(&functions);

    let mut tables = TableSection::new();
    tables.table(TableType {
        element_type: RefType::FUNCREF,
        minimum: 4,
        maximum: None,
        shared: false,
        table64: false,
    });
    module.section(&tables);

    let mut exports = ExportSection::new();
    exports.export("ops", ExportKind::Func, 1);
    exports.export("size", ExportKind::Func, 2);
    exports.export("drop", ExportKind::Func, 3);
    module.section(&exports);

    let mut elements = ElementSection::new();
    elements.passive(Elements::Functions(Cow::Owned(vec![0])));
    module.section(&elements);

    let mut codes = CodeSection::new();

    // $target body (empty)
    let mut target_body = Function::new(vec![]);
    target_body.instructions().end();
    codes.function(&target_body);

    // ops body exercising table helpers
    let mut ops_body = Function::new(vec![]);
    ops_body
        .instructions()
        .i32_const(0)
        .ref_null(HeapType::FUNC)
        .i32_const(1)
        .table_fill(0)
        .i32_const(0)
        .i32_const(0)
        .i32_const(1)
        .table_init(0, 0)
        .i32_const(0)
        .table_get(0)
        .drop()
        .i32_const(2)
        .ref_func(0)
        .table_set(0)
        .i32_const(0)
        .i32_const(0)
        .i32_const(1)
        .table_copy(0, 0)
        .ref_func(0)
        .i32_const(1)
        .table_grow(0)
        .end();
    codes.function(&ops_body);

    // size body returns the current table size
    let mut size_body = Function::new(vec![]);
    size_body.instructions().table_size(0).end();
    codes.function(&size_body);

    // drop body releases the passive element segment
    let mut drop_body = Function::new(vec![]);
    drop_body.instructions().elem_drop(0).end();
    codes.function(&drop_body);

    module.section(&codes);

    let wasm = module.finish();

    let translation = translate_module(&wasm, "TableOps").expect("translation succeeds");

    let call = wasm_neovm::opcodes::lookup("CALL").unwrap().byte;
    let call_l = wasm_neovm::opcodes::lookup("CALL_L").unwrap().byte;
    let call_count = translation
        .script
        .iter()
        .filter(|&&opcode| opcode == call || opcode == call_l)
        .count();
    assert!(
        call_count >= 6,
        "expected helper calls for table operations"
    );

    let abort = wasm_neovm::opcodes::lookup("ABORT").unwrap().byte;
    assert!(
        translation.script.contains(&abort),
        "table dispatch should include abort traps"
    );

    let manifest = translation
        .manifest
        .to_json_string()
        .expect("manifest serialises");
    assert!(manifest.contains("\"name\": \"TableOps\""));
    assert!(manifest.contains("\"name\": \"ops\""));
}

#[test]
fn translate_table_inline_initializer() {
    let wasm = wat::parse_str(
        r#"(module
              (func $f0)
              (func $f1)
              (table funcref (elem $f0 $f1))
              (func (export "touch")
                i32.const 0
                table.get 0
                drop
                i32.const 1
                table.get 0
                drop))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "InlineTable").expect("translation succeeds");

    let sts_fld4 = wasm_neovm::opcodes::lookup("STSFLD4").unwrap().byte;
    assert!(
        translation.script.contains(&sts_fld4),
        "runtime init should store the table into static slot 4"
    );

    let lds_fld4 = wasm_neovm::opcodes::lookup("LDSFLD4").unwrap().byte;
    assert!(
        translation.script.contains(&lds_fld4),
        "table.get should load from the table static slot"
    );
}

#[test]
fn translate_multi_table_operations() {
    let wasm = wat::parse_str(
        r#"(module
              (type $t0 (func))
              (func $f0)
              (func $f1)
              (table 3 funcref)
              (table 2 funcref)
              (elem (i32.const 0) func $f0 $f1)
              (elem (table 1) (i32.const 1) func $f1)
              (func (export "manipulate") (result i32)
                i32.const 0
                table.get 0
                drop
                i32.const 1
                table.get 1
                drop
                i32.const 2
                ref.func $f0
                table.set 0
                i32.const 0
                ref.func $f1
                table.set 1
                i32.const 0
                i32.const 0
                i32.const 1
                table.copy 0 1
                i32.const 1
                ref.null func
                i32.const 1
                table.fill 1
                table.size 1))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MultiTable").expect("translation succeeds");

    let lds_fld4 = wasm_neovm::opcodes::lookup("LDSFLD4").unwrap().byte;
    let lds_fld5 = wasm_neovm::opcodes::lookup("LDSFLD5").unwrap().byte;
    assert!(
        translation.script.contains(&lds_fld4),
        "table operations should load from table 0 slot"
    );
    assert!(
        translation.script.contains(&lds_fld5),
        "table operations should load from table 1 slot"
    );

    let sts_fld4 = wasm_neovm::opcodes::lookup("STSFLD4").unwrap().byte;
    let sts_fld5 = wasm_neovm::opcodes::lookup("STSFLD5").unwrap().byte;
    assert!(
        translation.script.contains(&sts_fld4),
        "runtime init should store table 0"
    );
    assert!(
        translation.script.contains(&sts_fld5),
        "runtime init should store table 1"
    );

    let call = wasm_neovm::opcodes::lookup("CALL").unwrap().byte;
    let call_l = wasm_neovm::opcodes::lookup("CALL_L").unwrap().byte;
    let helper_calls = translation
        .script
        .iter()
        .filter(|&&opcode| opcode == call || opcode == call_l)
        .count();
    assert!(
        helper_calls >= 4,
        "expected helper invocations for table ops"
    );
}

#[test]
fn translate_table_init_and_drop_guards() {
    use std::borrow::Cow;
    use wasm_encoder::{
        CodeSection, ElementSection, Elements, ExportKind, ExportSection, Function,
        FunctionSection, Module, TableSection, TableType, TypeSection,
    };

    let mut module = Module::new();

    let mut types = TypeSection::new();
    types.ty().function(vec![], vec![]);
    module.section(&types);

    let mut functions = FunctionSection::new();
    functions.function(0); // $f0
    functions.function(0); // init
    functions.function(0); // drop
    functions.function(0); // reuse
    module.section(&functions);

    let mut tables = TableSection::new();
    tables.table(TableType {
        element_type: wasm_encoder::RefType::FUNCREF,
        minimum: 2,
        maximum: None,
        shared: false,
        table64: false,
    });
    module.section(&tables);

    let mut exports = ExportSection::new();
    exports.export("init", ExportKind::Func, 1);
    exports.export("drop", ExportKind::Func, 2);
    exports.export("reuse", ExportKind::Func, 3);
    module.section(&exports);

    let mut elements = ElementSection::new();
    elements.passive(Elements::Functions(Cow::Owned(vec![0])));
    module.section(&elements);

    let mut codes = CodeSection::new();

    let mut f0_body = Function::new(vec![]);
    f0_body.instructions().end();
    codes.function(&f0_body);

    let mut init_body = Function::new(vec![]);
    init_body
        .instructions()
        .i32_const(0)
        .i32_const(0)
        .i32_const(1)
        .table_init(0, 0)
        .end();
    codes.function(&init_body);

    let mut drop_body = Function::new(vec![]);
    drop_body.instructions().elem_drop(0).end();
    codes.function(&drop_body);

    let mut reuse_body = Function::new(vec![]);
    reuse_body
        .instructions()
        .i32_const(0)
        .i32_const(0)
        .i32_const(1)
        .table_init(0, 0)
        .end();
    codes.function(&reuse_body);

    module.section(&codes);

    let wasm = module.finish();

    let translation = translate_module(&wasm, "TableReuse").expect("translation succeeds");

    let lds_drop = wasm_neovm::opcodes::lookup("LDSFLD6").unwrap().byte;
    assert!(
        translation.script.contains(&lds_drop),
        "table.init helper should load the passive element drop slot"
    );

    let sts_drop = wasm_neovm::opcodes::lookup("STSFLD6").unwrap().byte;
    assert!(
        translation.script.contains(&sts_drop),
        "elem.drop should mark the segment as dropped"
    );

    let equal = wasm_neovm::opcodes::lookup("EQUAL").unwrap().byte;
    assert!(
        translation.script.contains(&equal),
        "table.init guard should compare the drop flag against zero"
    );
}

#[test]
fn translate_table_passive_expression_segment() {
    let wasm = wat::parse_str(
        r#"(module
              (func $f0)
              (table 2 funcref)
              (elem (i32.const 0) func $f0)
              (elem funcref (ref.func $f0) (ref.null func))
              (func (export "init")
                i32.const 0
                i32.const 0
                i32.const 2
                table.init 0 1
                elem.drop 1
                i32.const 0
                i32.const 0
                i32.const 2
                table.init 0 1))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "TableExpr").expect("translation succeeds");

    let pushm1 = wasm_neovm::opcodes::lookup("PUSHM1").unwrap().byte;
    assert!(
        translation.script.contains(&pushm1),
        "passive expression segments should encode ref.null as -1"
    );

    let equal = wasm_neovm::opcodes::lookup("EQUAL").unwrap().byte;
    assert!(
        translation.script.contains(&equal),
        "table.init helper should compare drop flag against zero"
    );
}

#[test]
fn translate_table_grow_with_maximum() {
    let wasm = wat::parse_str(
        r#"(module
              (table 1 2 funcref)
              (func (export "grow_ok") (result i32)
                ref.null func
                i32.const 1
                table.grow 0)
              (func (export "grow_fail") (result i32)
                ref.null func
                i32.const 2
                table.grow 0))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "TableGrow").expect("translation succeeds");

    let call = wasm_neovm::opcodes::lookup("CALL").unwrap().byte;
    let call_l = wasm_neovm::opcodes::lookup("CALL_L").unwrap().byte;
    let helper_calls = translation
        .script
        .iter()
        .filter(|&&opcode| opcode == call || opcode == call_l)
        .count();
    assert!(
        helper_calls >= 2,
        "expected grow helpers to be invoked for both functions"
    );

    let pushm1 = wasm_neovm::opcodes::lookup("PUSHM1").unwrap().byte;
    assert!(
        translation.script.contains(&pushm1),
        "table.grow helper should return -1 when exceeding the maximum"
    );
}
