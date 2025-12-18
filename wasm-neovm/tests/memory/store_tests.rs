// Memory store operation tests

use wasm_neovm::translate_module;

#[test]
fn translate_i32_store() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "store") (param i32 i32)
                local.get 0
                local.get 1
                i32.store))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "I32Store").expect("translation succeeds");

    // i32.store requires helper function
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_i32_store8() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "store8") (param i32 i32)
                local.get 0
                local.get 1
                i32.store8))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "I32Store8").expect("translation succeeds");

    // store8 stores lowest byte only
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_i32_store16() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "store16") (param i32 i32)
                local.get 0
                local.get 1
                i32.store16))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "I32Store16").expect("translation succeeds");

    // store16 stores lowest 2 bytes
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_i64_store() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "store64") (param i32 i64)
                local.get 0
                local.get 1
                i64.store))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "I64Store").expect("translation succeeds");

    // i64.store stores 8 bytes
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_store_with_offset_and_alignment() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "store_offset_align") (param i32 i32)
                local.get 0
                local.get 1
                i32.store offset=8 align=4))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "StoreOffsetAlign").expect("translation succeeds");

    // Combined offset and alignment
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_store_at_zero() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "store_zero") (param i32)
                i32.const 0
                local.get 0
                i32.store))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "StoreZero").expect("translation succeeds");

    // Storing at address 0
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_memory_byte_swap_pattern() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "swap_bytes") (param i32)
                local.get 0
                local.get 0
                i32.load8_u offset=0
                local.get 0
                i32.load8_u offset=1
                local.get 0
                i32.store8 offset=0
                local.get 0
                i32.store8 offset=1))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ByteSwap").expect("translation succeeds");

    // Complex pattern: swap two bytes in memory
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_memory_pointer_arithmetic() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "pointer_arith") (param i32) (result i32)
                local.get 0
                i32.const 4
                i32.add
                i32.load))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "PointerArith").expect("translation succeeds");

    // Pointer arithmetic: base + offset
    assert!(!translation.script.is_empty());
}
