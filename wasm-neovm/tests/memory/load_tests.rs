// Memory load operation tests

use wasm_neovm::{opcodes, translate_module};

#[test]
fn translate_i32_load() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "load") (param i32) (result i32)
                local.get 0
                i32.load))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "I32Load").expect("translation succeeds");

    // i32.load requires helper function for memory access
    assert!(
        !translation.script.is_empty(),
        "should generate i32.load bytecode"
    );
}

#[test]
fn translate_i32_load8_s() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "load8_s") (param i32) (result i32)
                local.get 0
                i32.load8_s))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "I32Load8S").expect("translation succeeds");

    // load8_s loads 1 byte with sign extension
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_i32_load8_u() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "load8_u") (param i32) (result i32)
                local.get 0
                i32.load8_u))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "I32Load8U").expect("translation succeeds");

    // load8_u loads 1 byte with zero extension
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_i32_load16_s() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "load16_s") (param i32) (result i32)
                local.get 0
                i32.load16_s))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "I32Load16S").expect("translation succeeds");

    // load16_s loads 2 bytes with sign extension
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_i32_load8_u_zero_extend_without_shifts() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "load8_u") (param i32) (result i32)
                local.get 0
                i32.load8_u))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "I32Load8UZeroExtend").expect("translation succeeds");

    let and = opcodes::lookup("AND").unwrap().byte;
    let shr = opcodes::lookup("SHR").unwrap().byte;

    assert!(
        translation.script.contains(&and),
        "zero extension should mask high bits with AND"
    );
    assert!(
        !translation.script.contains(&shr),
        "unsigned load should not perform arithmetic right shifts"
    );
}

#[test]
fn translate_i64_load() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "load64") (param i32) (result i64)
                local.get 0
                i64.load))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "I64Load").expect("translation succeeds");

    // i64.load loads 8 bytes
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_i64_load32_u() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "load32_u") (param i32) (result i64)
                local.get 0
                i64.load32_u))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "I64Load32U").expect("translation succeeds");

    // load32_u loads 4 bytes with zero extension to i64
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_load_with_offset() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "load_offset") (param i32) (result i32)
                local.get 0
                i32.load offset=4))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "LoadOffset").expect("translation succeeds");

    // offset=4 adds 4 to the address
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_load_with_alignment() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "load_aligned") (param i32) (result i32)
                local.get 0
                i32.load align=4))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "LoadAligned").expect("translation succeeds");

    // align=4 specifies 4-byte alignment (hint for optimization)
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_load_at_boundary() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "load_boundary") (result i32)
                i32.const 65532
                i32.load))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "LoadBoundary").expect("translation succeeds");

    // Loading at page boundary (65536 - 4 bytes)
    assert!(!translation.script.is_empty());
}
