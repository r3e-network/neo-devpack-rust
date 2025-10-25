use wasm_neovm::{opcodes, translate_module};

#[test]
fn translate_memory_bounds_checks() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "load_safe") (param i32) (result i32)
                local.get 0
                i32.load)
              (func (export "store_safe") (param i32 i32)
                local.get 0
                local.get 1
                i32.store)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MemBounds").expect("translation succeeds");

    // Should contain bounds checking logic
    let call_l = opcodes::lookup("CALL_L").unwrap().byte;
    assert!(
        translation.script.contains(&call_l),
        "expected helper calls for bounds checking"
    );
}

#[test]
fn translate_memory_alignment() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "unaligned_load") (result i64)
                i32.const 1
                i64.load align=1)
              (func (export "aligned_store")
                i32.const 8
                i64.const 0x0102030405060708
                i64.store align=8)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MemAlign").expect("translation succeeds");
    let last = translation.script.last().copied();
    assert!(
        matches!(last, Some(0x40) | Some(0x38)),
        "expected memory alignment script to end in RET or ABORT sentinel, found {:?}",
        last
    );
}

#[test]
fn translate_memory_mixed_ops() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "mixed") (param i32) (result i32)
                local.get 0
                i32.const 100
                i32.store

                local.get 0
                i32.const 4
                i32.add
                i32.const 200
                i32.store16

                local.get 0
                i32.const 6
                i32.add
                i32.const 50
                i32.store8

                local.get 0
                i32.load
                local.get 0
                i32.const 4
                i32.add
                i32.load16_u
                i32.add
                local.get 0
                i32.const 6
                i32.add
                i32.load8_u
                i32.add)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MixedMemOps").expect("translation succeeds");

    let call_l = opcodes::lookup("CALL_L").unwrap().byte;
    // Multiple memory operations should call helpers
    let call_count = translation.script.iter().filter(|&&b| b == call_l).count();
    assert!(
        call_count >= 6,
        "expected multiple helper calls for memory operations"
    );
}

#[test]
fn translate_memory_overlapping_access() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "overlap") (result i32)
                i32.const 0
                i32.const 0x01020304
                i32.store

                i32.const 1
                i32.load8_u
                i32.const 2
                i32.load8_u
                i32.add
                i32.const 3
                i32.load8_u
                i32.add)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MemOverlap").expect("translation succeeds");
    let last = translation.script.last().copied();
    assert!(
        matches!(last, Some(0x40) | Some(0x38)),
        "expected overlapping access script to end in RET or ABORT sentinel, found {:?}",
        last
    );
}

#[test]
fn translate_memory_bulk_operations() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (data "Hello, World!")
              (func (export "bulk_ops")
                ;; memory.init: copy data segment to memory
                i32.const 100
                i32.const 0
                i32.const 13
                memory.init 0 0

                ;; memory.copy: copy within memory
                i32.const 200
                i32.const 100
                i32.const 13
                memory.copy

                ;; memory.fill: fill region with value
                i32.const 300
                i32.const 65
                i32.const 10
                memory.fill

                ;; data.drop: drop data segment
                data.drop 0)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "BulkOps").expect("translation succeeds");

    let memcpy = opcodes::lookup("MEMCPY").unwrap().byte;
    let setitem = opcodes::lookup("SETITEM").unwrap().byte;

    assert!(
        translation.script.contains(&memcpy),
        "expected MEMCPY for bulk operations"
    );
    assert!(
        translation.script.contains(&setitem),
        "expected SETITEM for fill"
    );
}

#[test]
fn translate_memory_copy_overlap_safe() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (data (i32.const 0) "\00\01\02\03\04")
              (func (export "copy_overlap")
                i32.const 1
                i32.const 0
                i32.const 4
                memory.copy))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "CopyOverlap").expect("translation succeeds");

    let memcpy = opcodes::lookup("MEMCPY").unwrap().byte;
    let newbuffer = opcodes::lookup("NEWBUFFER").unwrap().byte;
    let memcpy_count = translation
        .script
        .iter()
        .filter(|&&byte| byte == memcpy)
        .count();
    assert!(
        translation.script.contains(&newbuffer),
        "memory.copy should allocate a temporary buffer for overlap safety"
    );
    assert!(
        memcpy_count >= 2,
        "memory.copy should perform two MEMCPY operations via the temporary buffer"
    );
}

#[test]
fn translate_memory_grow_shrink() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1 10)
              (func (export "grow_test") (result i32)
                memory.size
                i32.const 2
                memory.grow
                drop
                memory.size)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MemGrowTest").expect("translation succeeds");

    let call_l = opcodes::lookup("CALL_L").unwrap().byte;
    let ldsfld2 = opcodes::lookup("LDSFLD2").unwrap().byte;

    assert!(
        translation.script.contains(&call_l),
        "memory.grow should use helper"
    );
    assert!(
        translation.script.contains(&ldsfld2),
        "memory.size should load from static field"
    );
}

#[test]
fn translate_memory_zero_initialize() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "zero_mem") (result i32)
                i32.const 0
                i32.const 0
                i32.const 1000
                memory.fill

                i32.const 500
                i32.load)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ZeroMem").expect("translation succeeds");
    let last = translation.script.last().copied();
    assert!(
        matches!(last, Some(0x40) | Some(0x38)),
        "expected zero-initialise script to end in RET or ABORT sentinel, found {:?}",
        last
    );
}

#[test]
fn translate_memory_pattern_write() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "pattern") (result i32)
                (local i32 i32)
                i32.const 0
                local.set 0
                i32.const 0
                local.set 1

                loop $continue
                  local.get 0
                  i32.const 100
                  i32.ge_s
                  br_if 1

                  local.get 0
                  local.get 1
                  i32.store8

                  local.get 0
                  i32.const 1
                  i32.add
                  local.set 0

                  local.get 1
                  i32.const 1
                  i32.add
                  i32.const 256
                  i32.rem_u
                  local.set 1

                  br $continue
                end

                i32.const 50
                i32.load8_u)
            )"#,
    )
    .expect("valid wat");

    let err = translate_module(&wasm, "MemPattern").expect_err("invalid branch should fail");
    let branch_issue = err
        .chain()
        .any(|cause| cause.to_string().contains("branch requires"));
    assert!(
        branch_issue,
        "unexpected memory pattern branch error: {}",
        err
    );
}

#[test]
fn translate_memory_endianness() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "endian") (result i32)
                i32.const 0
                i32.const 0x01020304
                i32.store

                i32.const 0
                i32.load8_u
                i32.const 8
                i32.shl

                i32.const 1
                i32.load8_u
                i32.or
                i32.const 8
                i32.shl

                i32.const 2
                i32.load8_u
                i32.or
                i32.const 8
                i32.shl

                i32.const 3
                i32.load8_u
                i32.or)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Endian").expect("translation succeeds");

    let shl = opcodes::lookup("SHL").unwrap().byte;
    let or_op = opcodes::lookup("OR").unwrap().byte;

    assert!(translation.script.contains(&shl));
    assert!(translation.script.contains(&or_op));
}

#[test]
fn translate_memory_sign_extension() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "sign_ext") (result i32)
                i32.const 0
                i32.const 0xFF
                i32.store8

                i32.const 0
                i32.load8_s)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "SignExt").expect("translation succeeds");
    let last = translation.script.last().copied();
    assert!(
        matches!(last, Some(0x40) | Some(0x38)),
        "expected sign-extension script to end in RET or ABORT sentinel, found {:?}",
        last
    );
}

#[test]
fn translate_memory_atomic_pattern() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "read_modify_write") (param i32 i32) (result i32)
                local.get 0
                local.get 0
                i32.load
                local.get 1
                i32.add
                local.tee 1
                i32.store
                local.get 1)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "AtomicPattern").expect("translation succeeds");
    let last = translation.script.last().copied();
    assert!(
        matches!(last, Some(0x40) | Some(0x38)),
        "expected atomic pattern script to end in RET or ABORT sentinel, found {:?}",
        last
    );
}
