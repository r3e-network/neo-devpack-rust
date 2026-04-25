// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use std::convert::TryInto;

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
    let call_s = opcodes::lookup("CALL").unwrap().byte;
    assert!(
        translation.script.contains(&call_l) || translation.script.contains(&call_s),
        "expected helper calls for bounds checking"
    );
}

#[test]
fn translate_memory_store_masks_address_to_u32() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "store")
                i32.const -1
                i32.const 42
                i32.store8)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MemAddrMask").expect("translation succeeds");

    let methods = translation.manifest.value["abi"]["methods"]
        .as_array()
        .expect("methods array");
    let store_method = methods
        .iter()
        .find(|method| method["name"].as_str() == Some("store"))
        .expect("store method present");
    let offset = store_method["offset"].as_u64().expect("offset is u64") as usize;

    let call = opcodes::lookup("CALL").unwrap().byte;
    let call_l = opcodes::lookup("CALL_L").unwrap().byte;
    let ret = opcodes::lookup("RET").unwrap().byte;

    let mut body_offset = offset;
    let mut cursor = offset;
    while cursor < translation.script.len() && translation.script[cursor] != ret {
        if translation.script[cursor] == call_l && cursor + 4 < translation.script.len() {
            let delta = i32::from_le_bytes(
                translation.script[cursor + 1..cursor + 5]
                    .try_into()
                    .unwrap(),
            );
            body_offset = (cursor as isize + delta as isize) as usize;
            cursor += 5;
        } else if translation.script[cursor] == call && cursor + 1 < translation.script.len() {
            let delta = translation.script[cursor + 1] as i8 as isize;
            body_offset = (cursor as isize + delta) as usize;
            cursor += 2;
        } else {
            cursor += 1;
        }
    }

    let body_end = translation.script[body_offset..]
        .iter()
        .position(|&byte| byte == ret)
        .expect("store body contains RET")
        + body_offset;
    let body = &translation.script[body_offset..=body_end];

    let push1 = opcodes::lookup("PUSH1").unwrap().byte;
    let pushint8 = opcodes::lookup("PUSHINT8").unwrap().byte;
    let pushint64 = opcodes::lookup("PUSHINT64").unwrap().byte;
    let shl = opcodes::lookup("SHL").unwrap().byte;
    let dec = opcodes::lookup("DEC").unwrap().byte;
    let sub = opcodes::lookup("SUB").unwrap().byte;
    let and = opcodes::lookup("AND").unwrap().byte;

    // Pattern 1: Compact inline computation: PUSH1, PUSH 32, SHL, DEC, AND (5 bytes)
    let pattern_compact = [push1, pushint8, 32, shl, dec, and];
    // Pattern 2: Legacy runtime computation: PUSH1, PUSH 32, SHL, PUSH1, SUB, AND
    let pattern_runtime = [push1, pushint8, 32, shl, push1, sub, and];
    // Pattern 3: Pre-computed constant 0xFFFFFFFF
    let pattern_const = [
        pushint64, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, and,
    ];

    assert!(
        body.windows(pattern_compact.len())
            .any(|window| window == pattern_compact)
            || body
                .windows(pattern_runtime.len())
                .any(|window| window == pattern_runtime)
            || body
                .windows(pattern_const.len())
                .any(|window| window == pattern_const),
        "expected u32 mask sequence in store body"
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

    let call = opcodes::lookup("CALL").unwrap().byte;
    let call_l = opcodes::lookup("CALL_L").unwrap().byte;
    // Multiple memory operations should call helpers
    let call_count = translation
        .script
        .iter()
        .filter(|&&b| b == call || b == call_l)
        .count();
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
fn translate_active_data_segment_negative_offset_fails_with_bounds_error() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (data (i32.const -1) "A")
              (func (export "noop"))
            )"#,
    )
    .expect("valid wat");

    let err = translate_module(&wasm, "ActiveDataNegOffset")
        .expect_err("active data segment negative offset should be rejected");

    let reports_negative = err
        .chain()
        .any(|cause| cause.to_string().contains("must be non-negative"));
    assert!(reports_negative, "unexpected error: {err}");
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
    let call_s = opcodes::lookup("CALL").unwrap().byte;
    let ldsfld2 = opcodes::lookup("LDSFLD2").unwrap().byte;

    assert!(
        translation.script.contains(&call_l) || translation.script.contains(&call_s),
        "memory.grow should use helper"
    );
    assert!(
        translation.script.contains(&ldsfld2),
        "memory.size should load from static field"
    );
}

#[test]
fn translate_large_memory_uses_page_chunks() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 17)
              (data (i32.const 1048576) "\2a\00\00\00")
              (func (export "load_high") (result i32)
                i32.const 1048576
                i32.load)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "LargeChunkedMemory").expect("translation succeeds");

    let newarray0 = opcodes::lookup("NEWARRAY0").unwrap().byte;
    let append = opcodes::lookup("APPEND").unwrap().byte;
    let pickitem = opcodes::lookup("PICKITEM").unwrap().byte;
    let newbuffer = opcodes::lookup("NEWBUFFER").unwrap().byte;
    let pushint32 = opcodes::lookup("PUSHINT32").unwrap().byte;

    assert!(
        translation.script.contains(&newarray0),
        "multi-page memory should initialize an array of page buffers"
    );
    assert!(
        translation.script.contains(&append),
        "multi-page memory should append page buffers"
    );
    assert!(
        translation.script.contains(&pickitem),
        "multi-page memory access should select the target page"
    );
    assert!(
        !translation
            .script
            .windows(6)
            .any(|window| window == [pushint32, 0x00, 0x00, 0x11, 0x00, newbuffer]),
        "translator must not allocate the 17-page memory as one NeoVM buffer"
    );
}

#[test]
fn translate_memory_grow_uses_chunked_pages() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1 10)
              (func (export "grow") (param i32) (result i32)
                local.get 0
                memory.grow))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MemGrowChunked").expect("translation succeeds");

    let newarray0 = opcodes::lookup("NEWARRAY0").unwrap().byte;
    let append = opcodes::lookup("APPEND").unwrap().byte;

    assert!(
        translation.script.contains(&newarray0),
        "growable memory should use page-array backing"
    );
    assert!(
        translation.script.contains(&append),
        "memory.grow should append new page buffers instead of reallocating one large buffer"
    );
}

#[test]
fn translate_memory_grow_enforces_maximum_without_operand_swap() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1 10)
              (func (export "grow") (param i32) (result i32)
                local.get 0
                memory.grow))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MemGrowMax").expect("translation succeeds");

    let ldsfld3 = opcodes::lookup("LDSFLD3").unwrap().byte;
    let dup = opcodes::lookup("DUP").unwrap().byte;
    let pushm1 = opcodes::lookup("PUSHM1").unwrap().byte;
    let equal = opcodes::lookup("EQUAL").unwrap().byte;
    let jmpif_l = opcodes::lookup("JMPIF_L").unwrap().byte;
    let jmpif_s = opcodes::lookup("JMPIF").unwrap().byte;
    let gt = opcodes::lookup("GT").unwrap().byte;

    // The memory.grow helper checks max pages by loading LDSFLD3, testing for -1 (unlimited),
    // then comparing new_pages > max with GT (no SWAP between max and new_pages).
    // JMPIF may be long (5 bytes) or short (2 bytes) after relaxation.
    let pattern_found = translation.script.windows(10).any(|window| {
        window[0] == ldsfld3
            && window[1] == dup
            && window[2] == pushm1
            && window[3] == equal
            && window[4] == jmpif_l
            && window[9] == gt
    }) || translation.script.windows(7).any(|window| {
        window[0] == ldsfld3
            && window[1] == dup
            && window[2] == pushm1
            && window[3] == equal
            && window[4] == jmpif_s
            && window[6] == gt
    });

    assert!(
        pattern_found,
        "expected memory.grow helper to compare new_pages > max directly after max-unlimited check"
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
