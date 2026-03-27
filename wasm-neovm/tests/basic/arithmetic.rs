// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use wasm_neovm::translate_module;

#[test]
fn translate_i32_bitcounts_fold_and_call_helpers() {
    let clz_wasm = wat::parse_str(
        r#"(module
              (func (export "clz_const") (result i32)
                i32.const 16
                i32.clz))"#,
    )
    .expect("valid wat");

    let clz_translation = translate_module(&clz_wasm, "BitClz").expect("translate clz");
    let call = wasm_neovm::opcodes::lookup("CALL").unwrap().byte;
    let call_l = wasm_neovm::opcodes::lookup("CALL_L").unwrap().byte;
    assert!(!clz_translation.script.contains(&call));
    assert!(!clz_translation.script.contains(&call_l));

    let pushint8 = wasm_neovm::opcodes::lookup("PUSHINT8").unwrap().byte;
    let drop = wasm_neovm::opcodes::lookup("DROP").unwrap().byte;
    assert!(clz_translation
        .script
        .windows(2)
        .any(|w| w == [pushint8, 27]));
    assert!(clz_translation.script.contains(&drop));

    let dynamic_wasm = wat::parse_str(
        r#"(module
              (func (export "ctz") (param i32) (result i32)
                local.get 0
                i32.ctz))"#,
    )
    .expect("valid wat");

    let dynamic_translation = translate_module(&dynamic_wasm, "BitCtz").expect("translate ctz");
    let call_count = dynamic_translation
        .script
        .iter()
        .filter(|&&b| b == call || b == call_l)
        .count();
    assert!(call_count >= 1, "expected at least 1 helper call for ctz");
}

#[test]
fn translate_i64_arithmetic() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "sum") (result i64)
                i64.const 5000000000
                i64.const 7
                i64.add)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Sum64").expect("translation succeeds");
    let push64 = wasm_neovm::opcodes::lookup("PUSHINT64").unwrap().byte;
    assert_eq!(translation.script.first().copied(), Some(push64));
    assert_eq!(&translation.script[1..9], &5000000000i64.to_le_bytes());
    let add = wasm_neovm::opcodes::lookup("ADD").unwrap().byte;
    assert!(translation.script.contains(&add));
    assert_eq!(translation.script.last().copied(), Some(0x40));
}

#[test]
fn translate_i32_rotl_constant() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "rot") (result i32)
                i32.const 0x12
                i32.const 8
                i32.rotl)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Rot32").expect("translation succeeds");
    // 0x12 rotated left by 8 bits -> 0x1200 (PUSHINT16 0x1200, RET).
    assert!(translation.script.ends_with(&[0x01, 0x00, 0x12, 0x40]));
    let drop = wasm_neovm::opcodes::lookup("DROP").unwrap().byte;
    assert!(translation.script.contains(&drop));
}

#[test]
fn translate_i64_rotr_constant() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "rot") (result i64)
                i64.const 0x0102030405060708
                i64.const 16
                i64.rotr)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Rot64").expect("translation succeeds");
    let expected = 0x0102030405060708u64.rotate_right(16).to_le_bytes();
    let mut suffix = vec![0x03];
    suffix.extend_from_slice(&expected);
    suffix.push(0x40);
    assert!(translation.script.ends_with(&suffix));
    let drop = wasm_neovm::opcodes::lookup("DROP").unwrap().byte;
    assert!(translation.script.contains(&drop));
}

#[test]
fn translate_i32_rotl_dynamic() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "rot") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.rotl)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "RotDyn32").expect("translation succeeds");
    assert_eq!(translation.script.last().copied(), Some(0x40));
}

#[test]
fn translate_i64_rotr_dynamic() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "rot") (param i64 i64) (result i64)
                local.get 0
                local.get 1
                i64.rotr)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "RotDyn64").expect("translation succeeds");
    assert_eq!(translation.script.last().copied(), Some(0x40));
}

#[test]
fn translate_i32_bitwise_chain() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "bits") (result i32)
                i32.const 6
                i32.const 3
                i32.and
                i32.const 1
                i32.or
                i32.const 2
                i32.xor)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Bits").expect("translation succeeds");
    let and = wasm_neovm::opcodes::lookup("AND").unwrap().byte;
    let or = wasm_neovm::opcodes::lookup("OR").unwrap().byte;
    let xor = wasm_neovm::opcodes::lookup("XOR").unwrap().byte;
    assert_eq!(
        translation.script,
        vec![0x16, 0x13, and, 0x11, or, 0x12, xor, 0x40]
    );
}

#[test]
fn translate_i32_signed_comparisons() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "cmp_const") (result i32)
                i32.const -1
                i32.const 0
                i32.lt_s)
              (func (export "cmp_dyn") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.gt_s)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "SignedCmp").expect("translation succeeds");

    let gt = wasm_neovm::opcodes::lookup("GT").unwrap().byte;
    let lt = wasm_neovm::opcodes::lookup("LT").unwrap().byte;
    assert!(translation.script.contains(&gt));
    assert!(translation.script.contains(&lt));

    let and = wasm_neovm::opcodes::lookup("AND").unwrap().byte;
    let and_count = translation.script.iter().filter(|&&b| b == and).count();
    // ANDs may be in the shared sign-extend helper rather than inline
    assert!(and_count >= 1, "expected parameter normalisation ANDs (inline or in helper)");
}

#[test]
fn translate_i32_unsigned_comparison_masks_operands() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "cmp") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.le_u)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "UnsignedCmp32").expect("translation succeeds");

    let and = wasm_neovm::opcodes::lookup("AND").unwrap().byte;
    let swap = wasm_neovm::opcodes::lookup("SWAP").unwrap().byte;
    let le = wasm_neovm::opcodes::lookup("LE").unwrap().byte;

    assert!(translation.script.iter().filter(|&&b| b == and).count() >= 2);
    assert!(translation.script.iter().filter(|&&b| b == swap).count() >= 2);
    assert!(translation.script.contains(&le));
}

#[test]
fn translate_i64_unsigned_comparison_masks_operands() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "cmp") (param i64 i64) (result i32)
                local.get 0
                local.get 1
                i64.ge_u
                i64.eqz)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "UnsignedCmp64").expect("translation succeeds");

    let push128 = wasm_neovm::opcodes::lookup("PUSHINT128").unwrap().byte;
    let and = wasm_neovm::opcodes::lookup("AND").unwrap().byte;
    let swap = wasm_neovm::opcodes::lookup("SWAP").unwrap().byte;
    let ge = wasm_neovm::opcodes::lookup("GE").unwrap().byte;

    assert!(translation.script.iter().filter(|&&b| b == push128).count() >= 1);
    assert!(translation.script.iter().filter(|&&b| b == and).count() >= 2);
    assert!(translation.script.iter().filter(|&&b| b == swap).count() >= 2);
    assert!(translation.script.contains(&ge));
}

#[test]
fn translate_i32_wrap_i64_constant() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "wrap") (result i32)
                i64.const 0x1_0000_0001
                i32.wrap_i64)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Wrap").expect("translation succeeds");
    assert!(translation.script.ends_with(&[0x11, 0x40]));
    let drop = wasm_neovm::opcodes::lookup("DROP").unwrap().byte;
    assert!(translation.script.contains(&drop));
}

#[test]
fn translate_i64_extend_i32_signed_dynamic() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "extend") (param i32) (result i64)
                local.get 0
                i64.extend_i32_s)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ExtendS").expect("translation succeeds");
    let and = wasm_neovm::opcodes::lookup("AND").unwrap().byte;
    let xor = wasm_neovm::opcodes::lookup("XOR").unwrap().byte;
    let sub = wasm_neovm::opcodes::lookup("SUB").unwrap().byte;
    // Branchless sign-extend uses XOR-SUB pattern instead of GE-branch
    assert!(translation.script.contains(&and));
    assert!(translation.script.contains(&xor));
    assert!(translation.script.contains(&sub));
    assert_eq!(translation.script.last().copied(), Some(0x40));
}

#[test]
fn translate_i64_extend_i32_unsigned_masks_only() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "extend") (param i32) (result i64)
                local.get 0
                i64.extend_i32_u)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ExtendU").expect("translation succeeds");
    let and = wasm_neovm::opcodes::lookup("AND").unwrap().byte;
    let shr = wasm_neovm::opcodes::lookup("SHR").unwrap().byte;
    assert!(translation.script.contains(&and));
    assert!(!translation.script.contains(&shr));
}

#[test]
fn translate_i32_extend8_sign_constant() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "extend") (result i32)
                i32.const 0xFF
                i32.extend8_s)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Extend8").expect("translation succeeds");
    let pushm1 = wasm_neovm::opcodes::lookup("PUSHM1").unwrap().byte;
    assert!(translation.script.ends_with(&[pushm1, 0x40]));
    let drop = wasm_neovm::opcodes::lookup("DROP").unwrap().byte;
    assert!(translation.script.contains(&drop));
}

#[test]
fn translate_i64_extend32_sign_constant() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "extend") (result i64)
                i64.const 0xFFFF_FFFF
                i64.extend32_s)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Extend32").expect("translation succeeds");
    let pushm1 = wasm_neovm::opcodes::lookup("PUSHM1").unwrap().byte;
    assert!(translation.script.ends_with(&[pushm1, 0x40]));
    let drop = wasm_neovm::opcodes::lookup("DROP").unwrap().byte;
    assert!(translation.script.contains(&drop));
}

#[test]
fn translate_i64_shift_ops() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "shift") (result i64)
                i64.const 8
                i64.const 1
                i64.shl
                i64.const 1
                i64.shr_s)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Shift").expect("translation succeeds");
    let shl = wasm_neovm::opcodes::lookup("SHL").unwrap().byte;
    let shr = wasm_neovm::opcodes::lookup("SHR").unwrap().byte;
    let and = wasm_neovm::opcodes::lookup("AND").unwrap().byte;
    let first_and = translation
        .script
        .iter()
        .position(|&b| b == and)
        .expect("shift masking emits AND");
    let shl_pos = translation.script.iter().position(|&b| b == shl).unwrap();
    assert!(
        first_and < shl_pos,
        "expected shift amount to be masked before SHL"
    );
    assert!(translation.script.contains(&shr));
    assert_eq!(translation.script.last().copied(), Some(0x40));
}

#[test]
fn translate_i32_shr_u() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "shr") (result i32)
                i32.const -1
                i32.const 1
                i32.shr_u)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ShrU").expect("translation succeeds");
    let mut expected = vec![
        wasm_neovm::opcodes::lookup("PUSHM1").unwrap().byte,
        wasm_neovm::opcodes::lookup("PUSH1").unwrap().byte,
        wasm_neovm::opcodes::lookup("PUSHINT8").unwrap().byte,
        0x1F,
        wasm_neovm::opcodes::lookup("AND").unwrap().byte,
        wasm_neovm::opcodes::lookup("SWAP").unwrap().byte,
        wasm_neovm::opcodes::lookup("PUSHINT64").unwrap().byte,
    ];
    expected.extend_from_slice(&0xFFFF_FFFFu64.to_le_bytes());
    expected.extend_from_slice(&[
        wasm_neovm::opcodes::lookup("AND").unwrap().byte,
        wasm_neovm::opcodes::lookup("SWAP").unwrap().byte,
        wasm_neovm::opcodes::lookup("SHR").unwrap().byte,
    ]);
    assert!(
        translation.script.starts_with(&expected),
        "shr_u prefix should match masking and shift sequence"
    );
    assert_eq!(translation.script.last().copied(), Some(0x40));
}

#[test]
fn translate_i32_div_signed() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "divs") (result i32)
                i32.const -6
                i32.const 2
                i32.div_s)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "DivS").expect("translation succeeds");
    let div = wasm_neovm::opcodes::lookup("DIV").unwrap().byte;
    let abort = wasm_neovm::opcodes::lookup("ABORT").unwrap().byte;
    let booland = wasm_neovm::opcodes::lookup("BOOLAND").unwrap().byte;
    assert!(translation.script.contains(&div));
    assert!(translation.script.contains(&abort));
    assert!(translation.script.contains(&booland));
    assert_eq!(translation.script.last().copied(), Some(0x40));
}

#[test]
fn translate_i32_div_unsigned_masks_operands() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "divu") (result i32)
                i32.const -1
                i32.const 3
                i32.div_u)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "DivU").expect("translation succeeds");
    let mut expected = vec![
        wasm_neovm::opcodes::lookup("PUSHM1").unwrap().byte,
        wasm_neovm::opcodes::lookup("PUSH3").unwrap().byte,
        wasm_neovm::opcodes::lookup("PUSHINT64").unwrap().byte,
    ];
    expected.extend_from_slice(&0xFFFF_FFFFu64.to_le_bytes());
    expected.extend_from_slice(&[
        wasm_neovm::opcodes::lookup("AND").unwrap().byte,
        wasm_neovm::opcodes::lookup("SWAP").unwrap().byte,
        wasm_neovm::opcodes::lookup("PUSHINT64").unwrap().byte,
    ]);
    expected.extend_from_slice(&0xFFFF_FFFFu64.to_le_bytes());
    expected.extend_from_slice(&[
        wasm_neovm::opcodes::lookup("AND").unwrap().byte,
        wasm_neovm::opcodes::lookup("SWAP").unwrap().byte,
    ]);
    assert!(
        translation.script.starts_with(&expected),
        "div_u should mask operands before performing the division"
    );
    assert!(translation
        .script
        .contains(&wasm_neovm::opcodes::lookup("DIV").unwrap().byte));
    assert!(translation
        .script
        .contains(&wasm_neovm::opcodes::lookup("ABORT").unwrap().byte));
    assert_eq!(translation.script.last().copied(), Some(0x40));
}

#[test]
fn translate_i32_rem_unsigned_masks_operands() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "remu") (result i32)
                i32.const -1
                i32.const 3
                i32.rem_u)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "RemU").expect("translation succeeds");
    let mut expected = vec![
        wasm_neovm::opcodes::lookup("PUSHM1").unwrap().byte,
        wasm_neovm::opcodes::lookup("PUSH3").unwrap().byte,
        wasm_neovm::opcodes::lookup("PUSHINT64").unwrap().byte,
    ];
    expected.extend_from_slice(&0xFFFF_FFFFu64.to_le_bytes());
    expected.extend_from_slice(&[
        wasm_neovm::opcodes::lookup("AND").unwrap().byte,
        wasm_neovm::opcodes::lookup("SWAP").unwrap().byte,
        wasm_neovm::opcodes::lookup("PUSHINT64").unwrap().byte,
    ]);
    expected.extend_from_slice(&0xFFFF_FFFFu64.to_le_bytes());
    expected.extend_from_slice(&[
        wasm_neovm::opcodes::lookup("AND").unwrap().byte,
        wasm_neovm::opcodes::lookup("SWAP").unwrap().byte,
    ]);
    assert!(
        translation.script.starts_with(&expected),
        "rem_u should mask operands before performing the remainder"
    );
    assert!(translation
        .script
        .contains(&wasm_neovm::opcodes::lookup("MOD").unwrap().byte));
    assert!(translation
        .script
        .contains(&wasm_neovm::opcodes::lookup("ABORT").unwrap().byte));
    assert_eq!(translation.script.last().copied(), Some(0x40));
}

#[test]
fn translate_i64_rem_unsigned_masks_operands() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "remu64") (result i64)
                i64.const -1
                i64.const 3
                i64.rem_u)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "RemU64").expect("translation succeeds");
    let mut expected = vec![
        wasm_neovm::opcodes::lookup("PUSHM1").unwrap().byte,
        wasm_neovm::opcodes::lookup("PUSH3").unwrap().byte,
        wasm_neovm::opcodes::lookup("PUSHINT128").unwrap().byte,
    ];
    expected.extend_from_slice(&(((1u128 << 64) - 1).to_le_bytes()));
    expected.extend_from_slice(&[
        wasm_neovm::opcodes::lookup("AND").unwrap().byte,
        wasm_neovm::opcodes::lookup("SWAP").unwrap().byte,
        wasm_neovm::opcodes::lookup("PUSHINT128").unwrap().byte,
    ]);
    expected.extend_from_slice(&(((1u128 << 64) - 1).to_le_bytes()));
    expected.extend_from_slice(&[
        wasm_neovm::opcodes::lookup("AND").unwrap().byte,
        wasm_neovm::opcodes::lookup("SWAP").unwrap().byte,
    ]);
    assert!(
        translation.script.starts_with(&expected),
        "i64.rem_u should mask operands before performing the remainder"
    );
    assert!(translation
        .script
        .contains(&wasm_neovm::opcodes::lookup("MOD").unwrap().byte));
    assert!(translation
        .script
        .contains(&wasm_neovm::opcodes::lookup("ABORT").unwrap().byte));
    assert_eq!(translation.script.last().copied(), Some(0x40));
}
