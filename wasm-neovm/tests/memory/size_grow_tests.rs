// Memory size and grow operation tests

use wasm_neovm::translate_module;

#[test]
fn translate_memory_size() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "size") (result i32)
                memory.size))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MemorySize").expect("translation succeeds");

    // memory.size returns current memory size in pages
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_memory_grow() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "grow") (param i32) (result i32)
                local.get 0
                memory.grow))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MemoryGrow").expect("translation succeeds");

    // memory.grow attempts to grow memory by specified pages
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_memory_grow_with_maximum() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1 10)
              (func (export "grow_limited") (param i32) (result i32)
                local.get 0
                memory.grow))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MemoryGrowLimited").expect("translation succeeds");

    // Memory with maximum (10 pages) limits growth
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_memory_grow_consumes_operand_for_control_flow() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "grow_in_block") (param i32) (result i32)
                (local i32)
                (block (result i32)
                  (block
                    local.get 0
                    local.tee 1
                    memory.grow
                    i32.const -1
                    i32.ne
                    br_if 0
                    i32.const 0
                    br 1)
                  i32.const 1)))"#,
    )
    .expect("valid wat");

    // Regression: memory.grow must consume its operand; otherwise branch validation will see
    // an extra value left on the stack and fail on valid modules.
    let translation = translate_module(&wasm, "MemoryGrowBranch").expect("translation succeeds");
    assert!(!translation.script.is_empty());
}
