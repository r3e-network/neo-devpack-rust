// Memory bulk operation tests (fill, copy, init)

use wasm_neovm::{opcodes, translate_module};

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
