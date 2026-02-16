// Comprehensive memory operation tests for WASM-NeoVM translator
// Phase 2: High-priority coverage additions

use wasm_neovm::{opcodes, translate_module};

// ============================================================================
// Load Operation Tests
// ============================================================================

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

// ============================================================================
// Store Operation Tests
// ============================================================================

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
fn memory_copy_handles_overlap() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "copy") (param i32 i32 i32)
                local.get 0
                local.get 1
                local.get 2
                memory.copy))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "CopyOverlap").expect("translation succeeds");

    let dec = opcodes::lookup("DEC").unwrap().byte;
    let pickitem = opcodes::lookup("PICKITEM").unwrap().byte;
    let setitem = opcodes::lookup("SETITEM").unwrap().byte;
    let memcpy = opcodes::lookup("MEMCPY").unwrap().byte;

    assert!(
        translation.script.contains(&dec),
        "overlap-safe copy should include backward loop with DEC"
    );
    assert!(
        translation.script.contains(&pickitem),
        "backward branch should read from source"
    );
    assert!(
        translation.script.contains(&setitem),
        "backward branch should write to destination"
    );
    assert!(
        translation.script.contains(&memcpy),
        "forward branch should still use MEMCPY fast path"
    );
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

// ============================================================================
// Memory Control Operations
// ============================================================================

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

// ============================================================================
// Alignment and Offset Tests
// ============================================================================

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

// ============================================================================
// Bulk Memory Operations
// ============================================================================

#[test]
fn translate_memory_fill() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "fill") (param i32 i32 i32)
                local.get 0
                local.get 1
                local.get 2
                memory.fill))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MemoryFill").expect("translation succeeds");

    // memory.fill sets memory region to a byte value
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_memory_copy() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "copy") (param i32 i32 i32)
                local.get 0
                local.get 1
                local.get 2
                memory.copy))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MemoryCopy").expect("translation succeeds");

    // memory.copy copies memory region
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_memory_init() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (data $d "hello")
              (func (export "init") (param i32 i32 i32)
                local.get 0
                local.get 1
                local.get 2
                memory.init $d))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MemoryInit").expect("translation succeeds");

    // memory.init initializes memory from data segment
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_data_drop() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (data $d "test")
              (func (export "drop")
                data.drop $d))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "DataDrop").expect("translation succeeds");

    // data.drop releases data segment
    assert!(!translation.script.is_empty());
}

// ============================================================================
// Memory Bounds Edge Cases
// ============================================================================

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

// ============================================================================
// Complex Memory Patterns
// ============================================================================

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

// ============================================================================
// Env Imports
// ============================================================================

#[test]
fn translate_env_memcpy_call() {
    let wasm = wat::parse_str(
        r#"(module
              (import "env" "memcpy" (func $memcpy (param i32 i32 i32) (result i32)))
              (memory 1)
              (func (export "copy_bytes") (param i32 i32 i32) (result i32)
                local.get 0
                local.get 1
                local.get 2
                call $memcpy))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "EnvMemcpy").expect("translation succeeds");
    let methods = translation
        .manifest
        .value
        .get("abi")
        .and_then(|abi| abi.get("methods"))
        .and_then(|methods| methods.as_array())
        .expect("methods present");
    assert_eq!(
        methods[0].get("returntype").and_then(|v| v.as_str()),
        Some("Integer")
    );
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_env_memset_call() {
    let wasm = wat::parse_str(
        r#"(module
              (import "env" "memset" (func $memset (param i32 i32 i32) (result i32)))
              (memory 1)
              (func (export "clear") (param i32 i32 i32) (result i32)
                local.get 0
                local.get 1
                local.get 2
                call $memset))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "EnvMemset").expect("translation succeeds");
    let methods = translation
        .manifest
        .value
        .get("abi")
        .and_then(|abi| abi.get("methods"))
        .and_then(|methods| methods.as_array())
        .expect("methods present");
    assert_eq!(
        methods[0].get("returntype").and_then(|v| v.as_str()),
        Some("Integer")
    );
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_env_memset_returns_original_dest() {
    let wasm = wat::parse_str(
        r#"(module
              (import "env" "memset" (func $memset (param i32 i32 i32) (result i32)))
              (memory 1)
              (func (export "clear") (param i32 i32 i32) (result i32)
                local.get 0
                local.get 1
                local.get 2
                call $memset))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "EnvMemsetReturn").expect("translation succeeds");

    let initslot = opcodes::lookup("INITSLOT").unwrap().byte;
    let stloc0 = opcodes::lookup("STLOC0").unwrap().byte;
    let stloc1 = opcodes::lookup("STLOC1").unwrap().byte;
    let stloc2 = opcodes::lookup("STLOC2").unwrap().byte;
    let ldloc3 = opcodes::lookup("LDLOC3").unwrap().byte;
    let ret = opcodes::lookup("RET").unwrap().byte;

    // env.memset helper prologue: INITSLOT 4 0, then pop len/value/dest into locals.
    let prologue = [initslot, 4, 0, stloc2, stloc1, stloc0];
    let helper_start = translation
        .script
        .windows(prologue.len())
        .position(|window| window == prologue)
        .expect("env.memset helper prologue present");

    let helper = &translation.script[helper_start..];
    let ret_pos = helper
        .iter()
        .position(|&byte| byte == ret)
        .expect("env.memset helper contains RET");
    assert!(
        ret_pos > 0 && helper[ret_pos - 1] == ldloc3,
        "env.memset helper should return the original destination pointer"
    );
}

#[test]
fn translate_env_memmove_call() {
    let wasm = wat::parse_str(
        r#"(module
              (import "env" "memmove" (func $memmove (param i32 i32 i32) (result i32)))
              (memory 1)
              (func (export "move") (param i32 i32 i32) (result i32)
                local.get 0
                local.get 1
                local.get 2
                call $memmove))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "EnvMemmove").expect("translation succeeds");
    let methods = translation
        .manifest
        .value
        .get("abi")
        .and_then(|abi| abi.get("methods"))
        .and_then(|methods| methods.as_array())
        .expect("methods present");
    assert_eq!(
        methods[0].get("returntype").and_then(|v| v.as_str()),
        Some("Integer")
    );
    assert!(!translation.script.is_empty());
}

#[test]
fn translate_env_double_underscore_mem_aliases() {
    let wasm = wat::parse_str(
        r#"(module
              (import "env" "__memcpy" (func $memcpy (param i32 i32 i32) (result i32)))
              (import "env" "__memmove" (func $memmove (param i32 i32 i32) (result i32)))
              (import "env" "__memset" (func $memset (param i32 i32 i32) (result i32)))
              (memory 1)
              (func (export "aliases")
                i32.const 0
                i32.const 16
                i32.const 4
                call $memcpy
                drop
                i32.const 8
                i32.const 0
                i32.const 4
                call $memmove
                drop
                i32.const 0
                i32.const 255
                i32.const 8
                call $memset
                drop))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "EnvMemAliases").expect("translation succeeds");
    assert!(!translation.script.is_empty());
}
