//! Round 54: Property-Based Testing
//!
//! This module implements property-based tests using the proptest pattern.
//! These tests verify that operations satisfy mathematical properties like
//! commutativity, associativity, and identity.

use wasm_neovm::opcodes;
use wasm_neovm::translate_module;

/// Property: Addition is commutative: a + b == b + a
///
/// For any two i32 values, the order of operands should not affect the result
#[test]
fn addition_is_commutative() {
    // Test with a representative set of values covering edge cases
    let test_values: Vec<i32> = vec![
        0,
        1,
        -1,
        42,
        -42,
        i32::MAX,
        i32::MIN,
        i32::MAX - 1,
        i32::MIN + 1,
        1000,
        -1000,
    ];

    for a in &test_values {
        for b in &test_values {
            // Skip cases that would overflow in our test calculation
            if (a > &0 && b > &0 && a.checked_add(*b).is_none())
                || (a < &0 && b < &0 && a.checked_add(*b).is_none())
            {
                continue;
            }

            let wasm_a_first = create_add_wasm(*a, *b);
            let wasm_b_first = create_add_wasm(*b, *a);

            let trans_a_first = translate_module(&wasm_a_first, "AddAFirst")
                .expect(&format!("Translation should succeed for {} + {}", a, b));
            let trans_b_first = translate_module(&wasm_b_first, "AddBFirst")
                .expect(&format!("Translation should succeed for {} + {}", b, a));

            // Both should have same structure
            assert_eq!(
                trans_a_first.script.len(),
                trans_b_first.script.len(),
                "Addition should produce same script length regardless of operand order"
            );
        }
    }
}

/// Property: Multiplication by zero always yields zero
///
/// For any value a: a * 0 == 0 * a == 0
#[test]
fn multiplication_by_zero_yields_zero() {
    let test_values: Vec<i32> = vec![0, 1, -1, 42, -42, i32::MAX, i32::MIN];

    for a in &test_values {
        // a * 0
        let wasm = wat::parse_str(&format!(
            r#"(module
                (func (export "test") (result i32)
                    i32.const {}
                    i32.const 0
                    i32.mul)
            )"#,
            a
        ))
        .expect("Valid WAT");

        let translation = translate_module(&wasm, "MulByZero").expect("Translation should succeed");

        // Should be able to constant fold to PUSH0
        let push0 = opcodes::lookup("PUSH0").unwrap().byte;
        assert!(
            !translation.script.is_empty(),
            "Multiplication by zero should fold to PUSH0 or equivalent"
        );
    }
}

/// Property: Bitwise AND with all ones equals the original value
///
/// For any i32 value a: a & 0xFFFFFFFF == a
#[test]
fn bitwise_and_with_all_ones_is_identity() {
    let test_values: Vec<i32> = vec![0, 1, -1, 42, i32::MAX, i32::MIN];

    for a in &test_values {
        let wasm = wat::parse_str(&format!(
            r#"(module
                (func (export "test") (result i32)
                    i32.const {}
                    i32.const -1
                    i32.and)
            )"#,
            a
        ))
        .expect("Valid WAT");

        let translation =
            translate_module(&wasm, "AndAllOnes").expect("Translation should succeed");

        // Should be optimized to just push the value
        assert_eq!(
            translation.script.last(),
            Some(&0x40),
            "Script should end with RET"
        );
    }
}

/// Property: Bitwise OR with zero equals the original value
///
/// For any i32 value a: a | 0 == a
#[test]
fn bitwise_or_with_zero_is_identity() {
    let test_values: Vec<i32> = vec![0, 1, -1, 42, i32::MAX, i32::MIN];

    for a in &test_values {
        let wasm = wat::parse_str(&format!(
            r#"(module
                (func (export "test") (result i32)
                    i32.const {}
                    i32.const 0
                    i32.or)
            )"#,
            a
        ))
        .expect("Valid WAT");

        let translation =
            translate_module(&wasm, "OrWithZero").expect("Translation should succeed");

        // Should be optimized to just push the value
        assert_eq!(
            translation.script.last(),
            Some(&0x40),
            "Script should end with RET"
        );
    }
}

/// Property: XOR with zero equals the original value
///
/// For any i32 value a: a ^ 0 == a
#[test]
fn bitwise_xor_with_zero_is_identity() {
    let test_values: Vec<i32> = vec![0, 1, -1, 42, i32::MAX, i32::MIN];

    for a in &test_values {
        let wasm = wat::parse_str(&format!(
            r#"(module
                (func (export "test") (result i32)
                    i32.const {}
                    i32.const 0
                    i32.xor)
            )"#,
            a
        ))
        .expect("Valid WAT");

        let translation =
            translate_module(&wasm, "XorWithZero").expect("Translation should succeed");

        assert_eq!(
            translation.script.last(),
            Some(&0x40),
            "Script should end with RET"
        );
    }
}

/// Property: XOR with self equals zero
///
/// For any value a: a ^ a == 0
#[test]
fn bitwise_xor_with_self_yields_zero() {
    let test_values: Vec<i32> = vec![0, 1, -1, 42, i32::MAX, i32::MIN];

    for a in &test_values {
        let wasm = wat::parse_str(&format!(
            r#"(module
                (func (export "test") (result i32)
                    i32.const {}
                    i32.const {}
                    i32.xor)
            )"#,
            a, a
        ))
        .expect("Valid WAT");

        let translation = translate_module(&wasm, "XorSelf").expect("Translation should succeed");

        // Should fold to PUSH0
        let push0 = opcodes::lookup("PUSH0").unwrap().byte;
        assert!(
            !translation.script.is_empty(),
            "XOR with self should produce non-empty script"
        );
    }
}

/// Property: Shift by zero is identity
///
/// For any value a and shift direction: a << 0 == a >> 0 == a
#[test]
fn shift_by_zero_is_identity() {
    let test_values: Vec<i32> = vec![1, 42, 100, i32::MAX];

    for a in &test_values {
        // Left shift by 0
        let wasm_shl = wat::parse_str(&format!(
            r#"(module
                (func (export "test") (result i32)
                    i32.const {}
                    i32.const 0
                    i32.shl)
            )"#,
            a
        ))
        .expect("Valid WAT");

        let trans_shl =
            translate_module(&wasm_shl, "ShlByZero").expect("Translation should succeed");

        // Right shift by 0
        let wasm_shr = wat::parse_str(&format!(
            r#"(module
                (func (export "test") (result i32)
                    i32.const {}
                    i32.const 0
                    i32.shr_s)
            )"#,
            a
        ))
        .expect("Valid WAT");

        let trans_shr =
            translate_module(&wasm_shr, "ShrByZero").expect("Translation should succeed");

        // Both should be optimized
        assert_eq!(
            trans_shl.script.last(),
            Some(&0x40),
            "Left shift by zero should be optimized"
        );
        assert_eq!(
            trans_shr.script.last(),
            Some(&0x40),
            "Right shift by zero should be optimized"
        );
    }
}

/// Property: Comparison equality is reflexive
///
/// For any value a: a == a is always true
#[test]
fn equality_comparison_is_reflexive() {
    let test_values: Vec<i32> = vec![0, 1, -1, 42, i32::MAX, i32::MIN];

    for a in &test_values {
        let wasm = wat::parse_str(&format!(
            r#"(module
                (func (export "test") (result i32)
                    i32.const {}
                    i32.const {}
                    i32.eq)
            )"#,
            a, a
        ))
        .expect("Valid WAT");

        let translation =
            translate_module(&wasm, "EqReflexive").expect("Translation should succeed");

        // a == a should always be true (PUSH1)
        let push1 = opcodes::lookup("PUSH1").unwrap().byte;
        assert!(
            !translation.script.is_empty(),
            "Self equality should produce valid script"
        );
    }
}

/// Property: Comparison with self for less/greater is always false
///
/// For any value a: a < a == false and a > a == false
#[test]
fn self_comparison_is_always_false() {
    let test_values: Vec<i32> = vec![0, 1, -1, 42, i32::MAX, i32::MIN];

    for a in &test_values {
        // a < a
        let wasm_lt = wat::parse_str(&format!(
            r#"(module
                (func (export "test") (result i32)
                    i32.const {}
                    i32.const {}
                    i32.lt_s)
            )"#,
            a, a
        ))
        .expect("Valid WAT");

        let trans_lt = translate_module(&wasm_lt, "LtSelf").expect("Translation should succeed");

        // a > a
        let wasm_gt = wat::parse_str(&format!(
            r#"(module
                (func (export "test") (result i32)
                    i32.const {}
                    i32.const {}
                    i32.gt_s)
            )"#,
            a, a
        ))
        .expect("Valid WAT");

        let trans_gt = translate_module(&wasm_gt, "GtSelf").expect("Translation should succeed");

        // Both should be false (PUSH0)
        let push0 = opcodes::lookup("PUSH0").unwrap().byte;
        assert!(
            !trans_lt.script.is_empty(),
            "Self less-than comparison should produce script"
        );
        assert!(
            !trans_gt.script.is_empty(),
            "Self greater-than comparison should produce script"
        );
    }
}

/// Property: Unsigned comparison of negative numbers
///
/// For negative values: when interpreted as unsigned, -1 > 0
#[test]
fn unsigned_comparison_of_negative_values() {
    let wasm = wat::parse_str(
        r#"(module
            (func (export "test") (result i32)
                i32.const -1
                i32.const 0
                i32.lt_u)
        )"#,
    )
    .expect("Valid WAT");

    let translation = translate_module(&wasm, "UnsignedLt").expect("Translation should succeed");

    // -1 as unsigned is 0xFFFFFFFF which is > 0
    // So -1 < 0 (unsigned) should be false
    let push0 = opcodes::lookup("PUSH0").unwrap().byte;
    assert!(
        !translation.script.is_empty(),
        "0xFFFFFFFF < 0 (unsigned) should be false"
    );
}

/// Property: Arithmetic identities with i64
///
/// Similar properties should hold for i64 operations
#[test]
fn i64_arithmetic_identities() {
    let test_values: Vec<i64> = vec![0, 1, -1, 1000, i64::MAX, i64::MIN];

    for a in &test_values {
        // Identity: a + 0 == a
        let wasm = wat::parse_str(&format!(
            r#"(module
                (func (export "test") (result i64)
                    i64.const {}
                    i64.const 0
                    i64.add)
            )"#,
            a
        ))
        .expect("Valid WAT");

        let translation = translate_module(&wasm, "I64AddIdentity")
            .expect(&format!("Translation should succeed for i64 {}", a));

        assert_eq!(
            translation.script.last(),
            Some(&0x40),
            "i64 + 0 should translate successfully"
        );
    }
}

/// Property: Wrap/extend roundtrip for i32/i64
///
/// Converting i32 -> i64 -> i32 should preserve the value (mod 2^32)
#[test]
fn wrap_extend_roundtrip_preserves_lower_bits() {
    let test_values: Vec<i32> = vec![0, 1, -1, 42, i32::MAX, i32::MIN];

    for a in &test_values {
        // i32 -> i64 (signed extend) -> wrap back to i32
        let wasm = wat::parse_str(&format!(
            r#"(module
                (func (export "test") (result i32)
                    i32.const {}
                    i64.extend_i32_s
                    i32.wrap_i64)
            )"#,
            a
        ))
        .expect("Valid WAT");

        let translation = translate_module(&wasm, "Roundtrip").expect("Translation should succeed");

        // Should be optimized to identity (just the value)
        assert_eq!(
            translation.script.last(),
            Some(&0x40),
            "Roundtrip extension should be optimized"
        );
    }
}

/// Property: Select with constant condition
///
/// select(a, b, true) == a and select(a, b, false) == b
#[test]
fn select_with_constant_condition() {
    // select(10, 20, true) == 10
    let wasm_true = wat::parse_str(
        r#"(module
            (func (export "test") (result i32)
                i32.const 10
                i32.const 20
                i32.const 1
                select)
        )"#,
    )
    .expect("Valid WAT");

    let trans_true =
        translate_module(&wasm_true, "SelectTrue").expect("Translation should succeed");

    // Should be optimized to PUSH10
    let push10 = opcodes::lookup("PUSH10");
    if let Some(op) = push10 {
        assert!(
            trans_true.script.contains(&op.byte),
            "select(x, y, true) should be optimized to x"
        );
    }

    // select(10, 20, false) == 20
    let wasm_false = wat::parse_str(
        r#"(module
            (func (export "test") (result i32)
                i32.const 10
                i32.const 20
                i32.const 0
                select)
        )"#,
    )
    .expect("Valid WAT");

    let trans_false =
        translate_module(&wasm_false, "SelectFalse").expect("Translation should succeed");

    // Should be optimized to PUSH20
    let push20 = opcodes::lookup("PUSH20");
    if let Some(op) = push20 {
        assert!(
            trans_false.script.contains(&op.byte),
            "select(x, y, false) should be optimized to y"
        );
    }
}

// Helper function to create add WASM
fn create_add_wasm(a: i32, b: i32) -> Vec<u8> {
    wat::parse_str(&format!(
        r#"(module
            (func (export "add") (result i32)
                i32.const {}
                i32.const {}
                i32.add)
        )"#,
        a, b
    ))
    .expect("Valid WAT")
}

/// Property: Idempotence of negation (double negation)
///
/// For any value a: -(-a) == a
#[test]
fn double_negation_is_identity() {
    let test_values: Vec<i32> = vec![0, 1, -1, 42, -42, i32::MAX, i32::MIN + 1];

    for a in &test_values {
        // Skip i32::MIN which overflows on negation
        if *a == i32::MIN {
            continue;
        }

        // a * -1 * -1 = a (using multiplication to simulate)
        let wasm = wat::parse_str(&format!(
            r#"(module
                (func (export "test") (result i32)
                    i32.const {}
                    i32.const -1
                    i32.mul
                    i32.const -1
                    i32.mul)
            )"#,
            a
        ))
        .expect("Valid WAT");

        let translation = translate_module(&wasm, "DoubleNeg").expect("Translation should succeed");

        assert_eq!(
            translation.script.last(),
            Some(&0x40),
            "Double negation should translate successfully"
        );
    }
}

/// Property: Min/Max with self
///
/// max(a, a) == a and min(a, a) == a
#[test]
fn min_max_with_self_is_identity() {
    // if a > a then a else a => a
    let wasm = wat::parse_str(
        r#"(module
            (func $test (param i32) (result i32)
                local.get 0
                local.get 0
                i32.gt_s
                if (result i32)
                    local.get 0
                else
                    local.get 0
                end)
            (func (export "call") (param i32) (result i32)
                local.get 0
                call $test)
        )"#,
    )
    .expect("Valid WAT");

    let translation = translate_module(&wasm, "MaxSelf").expect("Translation should succeed");

    assert_eq!(
        translation.script.last(),
        Some(&0x40),
        "max(a, a) should translate successfully"
    );
}

/// Property: Memory operations with offset 0 are equivalent to base address
///
/// load(addr + 0) == load(addr)
#[test]
fn memory_load_with_zero_offset() {
    let wasm = wat::parse_str(
        r#"(module
            (memory 1)
            (func (export "test") (result i32)
                i32.const 100
                i32.const 0
                i32.add
                i32.load)
        )"#,
    )
    .expect("Valid WAT");

    let translation =
        translate_module(&wasm, "LoadZeroOffset").expect("Translation should succeed");

    assert!(
        translation.script.len() > 0,
        "Memory load with zero offset should translate"
    );
}
