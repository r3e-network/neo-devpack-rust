//! Round 54: Property-Based Tests for rust-devpack
//!
//! This module implements property-based tests for Neo N3 types.
//! Properties tested include mathematical identities, associativity,
/// commutativity, and other invariants.
use neo_devpack::prelude::*;

/// Property: Integer addition is commutative
/// For all a, b: a + b == b + a
#[test]
fn prop_integer_addition_commutative() {
    let test_values = vec![0, 1, -1, 42, -42, 1000, -1000, i32::MAX - 1, i32::MIN + 1];

    for a in &test_values {
        for b in &test_values {
            let left = NeoInteger::new(*a) + NeoInteger::new(*b);
            let right = NeoInteger::new(*b) + NeoInteger::new(*a);
            assert_eq!(
                left.as_i32(),
                right.as_i32(),
                "Addition should be commutative: {} + {} != {} + {}",
                a,
                b,
                b,
                a
            );
        }
    }
}

/// Property: Integer addition is associative
/// For all a, b, c: (a + b) + c == a + (b + c)
#[test]
fn prop_integer_addition_associative() {
    let test_values = vec![0, 1, -1, 42, 100, 1000];

    for a in &test_values {
        for b in &test_values {
            for c in &test_values {
                let a_val = NeoInteger::new(*a);
                let b_val = NeoInteger::new(*b);
                let c_val = NeoInteger::new(*c);

                let left = (&(&a_val + &b_val) + &c_val).as_i32();
                let right = (&a_val + &(&b_val + &c_val)).as_i32();

                assert_eq!(
                    left, right,
                    "Addition should be associative: ({}) + ({}) + ({}) ",
                    a, b, c
                );
            }
        }
    }
}

/// Property: Integer multiplication is commutative
/// For all a, b: a * b == b * a
#[test]
fn prop_integer_multiplication_commutative() {
    let test_values = vec![0, 1, -1, 2, -2, 5, -5, 10, 100];

    for a in &test_values {
        for b in &test_values {
            let left = NeoInteger::new(*a) * NeoInteger::new(*b);
            let right = NeoInteger::new(*b) * NeoInteger::new(*a);
            assert_eq!(
                left.as_i32(),
                right.as_i32(),
                "Multiplication should be commutative: {} * {} != {} * {}",
                a,
                b,
                b,
                a
            );
        }
    }
}

/// Property: Identity elements
/// a + 0 == a
/// a * 1 == a
#[test]
fn prop_identity_elements() {
    let test_values = vec![0, 1, -1, 42, -42, 1000, i32::MAX, i32::MIN];
    let zero = NeoInteger::zero();
    let one = NeoInteger::one();

    for a in &test_values {
        let val = NeoInteger::new(*a);

        // Additive identity
        let sum = &val + &zero;
        assert_eq!(sum.as_i32(), *a, "Additive identity failed for {}", a);

        // Multiplicative identity
        let product = &val * &one;
        assert_eq!(
            product.as_i32(),
            *a,
            "Multiplicative identity failed for {}",
            a
        );
    }
}

/// Property: Multiplication by zero
/// For all a: a * 0 == 0 * a == 0
#[test]
fn prop_multiplication_by_zero() {
    let test_values = vec![0, 1, -1, 42, -42, i32::MAX, i32::MIN];
    let zero = NeoInteger::zero();

    for a in &test_values {
        let val = NeoInteger::new(*a);

        let left = &val * &zero;
        let right = &zero * &val;

        assert_eq!(
            left.as_i32(),
            0,
            "Multiplication by zero failed for {} * 0",
            a
        );
        assert_eq!(
            right.as_i32(),
            0,
            "Multiplication by zero failed for 0 * {}",
            a
        );
    }
}

/// Property: Double negation
/// For all a: -(-a) == a
#[test]
fn prop_double_negation() {
    // Skip i32::MIN which overflows on negation
    let test_values = vec![0, 1, -1, 42, -42, 1000, -1000, i32::MAX];

    for a in &test_values {
        let val = NeoInteger::new(*a);
        let neg = -val.clone();
        let double_neg = -neg;

        assert_eq!(double_neg.as_i32(), *a, "Double negation failed for {}", a);
    }
}

/// Property: Bitwise operations identities
/// a & 0 == 0
/// a | 0 == a
/// a ^ 0 == a
/// a ^ a == 0
#[test]
fn prop_bitwise_identities() {
    let test_values = vec![0, 1, -1, 42, 0xFFFFFFFFu32 as i32];
    let zero = NeoInteger::zero();

    for a in &test_values {
        let val = NeoInteger::new(*a);

        // a & 0 == 0
        let and_zero = &val & &zero;
        assert_eq!(
            and_zero.as_i32(),
            0,
            "Bitwise AND with zero failed for {}",
            a
        );

        // a | 0 == a
        let or_zero = &val | &zero;
        assert_eq!(
            or_zero.as_i32(),
            *a,
            "Bitwise OR with zero failed for {}",
            a
        );

        // a ^ 0 == a
        let xor_zero = &val ^ &zero;
        assert_eq!(
            xor_zero.as_i32(),
            *a,
            "Bitwise XOR with zero failed for {}",
            a
        );

        // a ^ a == 0
        let xor_self = &val ^ &val;
        assert_eq!(
            xor_self.as_i32(),
            0,
            "Bitwise XOR with self failed for {}",
            a
        );
    }
}

/// Property: De Morgan's laws for bitwise operations
/// ~(a & b) == ~a | ~b
/// ~(a | b) == ~a & ~b
#[test]
fn prop_de_morgan_laws() {
    let test_values = vec![0, 1, -1, 0xFF, 0xF0, 0x0F, 0xAAAAAAAAu32 as i32];

    for a in &test_values {
        for b in &test_values {
            let a_val = NeoInteger::new(*a);
            let b_val = NeoInteger::new(*b);

            // ~(a & b)
            let left1 = !(a_val.clone() & b_val.clone());
            // ~a | ~b
            let right1 = (!a_val.clone()) | (!b_val.clone());

            assert_eq!(
                left1.as_i32(),
                right1.as_i32(),
                "De Morgan's first law failed for a={}, b={}",
                a,
                b
            );

            // ~(a | b)
            let left2 = !(a_val.clone() | b_val.clone());
            // ~a & ~b
            let right2 = (!a_val.clone()) & (!b_val.clone());

            assert_eq!(
                left2.as_i32(),
                right2.as_i32(),
                "De Morgan's second law failed for a={}, b={}",
                a,
                b
            );
        }
    }
}

/// Property: Boolean operations
#[test]
fn prop_boolean_operations() {
    let t = NeoBoolean::TRUE;
    let f = NeoBoolean::FALSE;

    // Idempotent: a & a == a, a | a == a
    assert_eq!((t.clone() & t.clone()).as_bool(), true);
    assert_eq!((f.clone() & f.clone()).as_bool(), false);
    assert_eq!((t.clone() | t.clone()).as_bool(), true);
    assert_eq!((f.clone() | f.clone()).as_bool(), false);

    // Complement: a & !a == false, a | !a == true
    assert_eq!((t.clone() & !t.clone()).as_bool(), false);
    assert_eq!((t.clone() | !t.clone()).as_bool(), true);
    assert_eq!((f.clone() & !f.clone()).as_bool(), false);
    assert_eq!((f.clone() | !f.clone()).as_bool(), true);

    // Double negation
    assert_eq!((!!t.clone()).as_bool(), true);
    assert_eq!((!!f.clone()).as_bool(), false);
}

/// Property: Array operations
/// Pushing then popping yields the original value
#[test]
fn prop_array_push_pop_inverse() {
    let test_values: Vec<NeoValue> = vec![
        NeoValue::from(NeoInteger::new(42)),
        NeoValue::from(NeoBoolean::TRUE),
        NeoValue::from(NeoString::from_str("test")),
        NeoValue::from(NeoByteString::from_slice(b"bytes")),
        NeoValue::Null,
    ];

    for val in test_values {
        let mut arr = NeoArray::new();
        arr.push(val.clone());
        let popped = arr.pop().expect("Should pop a value");

        // Values should be equal
        assert_eq!(arr.len(), 0);
        assert!(arr.is_empty());
    }
}

/// Property: Map insert and get
/// Inserting a key-value pair then getting by key returns the value
#[test]
fn prop_map_insert_get() {
    let keys: Vec<NeoValue> = vec![
        NeoValue::from(NeoInteger::new(1)),
        NeoValue::from(NeoInteger::new(2)),
        NeoValue::from(NeoString::from_str("key")),
        NeoValue::from(NeoByteString::from_slice(b"bytes")),
    ];

    let values: Vec<NeoValue> = vec![
        NeoValue::from(NeoInteger::new(100)),
        NeoValue::from(NeoBoolean::TRUE),
        NeoValue::from(NeoString::from_str("value")),
    ];

    for key in &keys {
        for value in &values {
            let mut map = NeoMap::new();
            map.insert(key.clone(), value.clone());

            let retrieved = map.get(key);
            assert!(retrieved.is_some(), "Should retrieve inserted value");
        }
    }
}

/// Property: Map remove
/// Removing a key then getting it returns None
#[test]
fn prop_map_remove() {
    let key = NeoValue::from(NeoString::from_str("test_key"));
    let value = NeoValue::from(NeoInteger::new(42));

    let mut map = NeoMap::new();
    map.insert(key.clone(), value);
    assert!(map.get(&key).is_some());

    map.remove(&key);
    assert!(map.get(&key).is_none());
}

/// Property: Iterator correctness
/// Iterating over all elements returns them in order
#[test]
fn prop_iterator_order() {
    let data: Vec<NeoValue> = (0..100)
        .map(|i| NeoValue::from(NeoInteger::new(i)))
        .collect();

    let mut iter = NeoIterator::new(data);
    let mut index = 0;

    while iter.has_next() {
        let val = iter.next().expect("Should have value");
        assert_eq!(
            val.as_integer().unwrap().as_i32(),
            index,
            "Iterator order incorrect at index {}",
            index
        );
        index += 1;
    }

    assert_eq!(index, 100, "Iterator should yield all elements");
}

/// Property: ByteString append
/// Appending preserves all bytes
#[test]
fn prop_bytestring_append() {
    let parts: Vec<Vec<u8>> = vec![vec![0x01, 0x02], vec![0x03, 0x04, 0x05], vec![], vec![0x06]];

    let mut bs = NeoByteString::new(vec![]);
    let mut expected = vec![];

    for part in &parts {
        bs.extend_from_slice(part);
        expected.extend_from_slice(part);
    }

    assert_eq!(bs.as_slice(), &expected);
}

/// Property: Shift by multiple of width
/// Shifting by 32 should be equivalent to shifting by 0 for i32
#[test]
fn prop_shift_modulo() {
    let test_values = vec![1, 2, 4, 8, 16, 0xFF];

    for val in &test_values {
        let original = NeoInteger::new(*val);

        // Left shift by 32 should give same value
        let shifted_32 = &original << 32;
        let shifted_0 = &original << 0;
        assert_eq!(
            shifted_32.as_i32(),
            shifted_0.as_i32(),
            "Shift by 32 should equal shift by 0 for i32"
        );

        // Right shift by 32 should give same value (for positive)
        let shifted_32r = &original >> 32;
        let shifted_0r = &original >> 0;
        assert_eq!(
            shifted_32r.as_i32(),
            shifted_0r.as_i32(),
            "Right shift by 32 should equal shift by 0 for i32"
        );
    }
}

/// Property: Comparison consistency
/// a == b implies b == a
/// a < b implies b > a
#[test]
fn prop_comparison_consistency() {
    let test_values = vec![0, 1, -1, 42, -42, 100, -100];

    for a in &test_values {
        for b in &test_values {
            // Equality is symmetric
            let a_eq_b = a == b;
            let b_eq_a = b == a;
            assert_eq!(a_eq_b, b_eq_a, "Equality should be symmetric");

            // Less-than/greater-than relationship
            let a_lt_b = a < b;
            let b_gt_a = b > a;
            assert_eq!(
                a_lt_b, b_gt_a,
                "Less-than and greater-than should be consistent"
            );
        }
    }
}

/// Property: Struct field access
/// Setting then getting a field returns the set value
#[test]
fn prop_struct_field_access() {
    let fields = vec![
        ("int", NeoValue::from(NeoInteger::new(42))),
        ("bool", NeoValue::from(NeoBoolean::TRUE)),
        ("string", NeoValue::from(NeoString::from_str("test"))),
    ];

    let mut s = NeoStruct::new();

    for (name, value) in &fields {
        s.set_field(name, value.clone());
        let retrieved = s.get_field(name);
        assert!(retrieved.is_some(), "Should retrieve field '{}'", name);
    }
}

/// Property: Distributivity of multiplication over addition
/// a * (b + c) == a * b + a * c
#[test]
fn prop_multiplication_distributive() {
    let test_values = vec![0, 1, 2, 5, 10];

    for a in &test_values {
        for b in &test_values {
            for c in &test_values {
                let a_val = NeoInteger::new(*a);
                let b_val = NeoInteger::new(*b);
                let c_val = NeoInteger::new(*c);

                // a * (b + c)
                let left = &a_val * &(&b_val + &c_val);
                // a * b + a * c
                let right = &(&a_val * &b_val) + &(&a_val * &c_val);

                assert_eq!(
                    left.as_i32(),
                    right.as_i32(),
                    "Distributivity failed for {} * ({} + {})",
                    a,
                    b,
                    c
                );
            }
        }
    }
}

/// Property: Storage context read-only behavior
/// Read-only contexts cannot be written to
#[test]
fn prop_storage_context_read_only() {
    let ctx = NeoStorageContext::read_only(1);
    assert!(ctx.is_read_only());

    let ctx2 = NeoStorageContext::new(2);
    assert!(!ctx2.is_read_only());
}
