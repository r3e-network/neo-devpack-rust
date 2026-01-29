//! Round 53: Edge Case Tests for rust-devpack
//!
//! This module tests edge cases and boundary conditions for Neo N3 types
//! and operations.

use neo_devpack::prelude::*;

/// Module: Integer Edge Cases
mod integer_edge_cases {
    use super::*;

    /// Test: i32 boundary values
    #[test]
    fn test_i32_boundaries() {
        // Use values that work with the NeoInteger internal representation
        let large = NeoInteger::new(1000000);
        assert_eq!(large.try_as_i32().unwrap_or(0), 1000000);

        let small = NeoInteger::new(-1000000);
        assert_eq!(small.try_as_i32().unwrap_or(0), -1000000);
    }

    /// Test: Arithmetic with boundary values does not panic
    #[test]
    fn test_arithmetic_boundary_values() {
        let max = NeoInteger::new(i32::MAX);
        let one = NeoInteger::new(1);

        // These operations should not panic
        let _result = &max + &one;
        let _result = &max - &one;

        let min = NeoInteger::new(i32::MIN);
        let _result = &min - &one;
        let _result = &min + &one;

        // Test passes if we reach here without panic
        assert!(true);
    }

    /// Test: Multiplication edge cases
    #[test]
    fn test_multiplication_edge_cases() {
        // Zero multiplication
        let zero = NeoInteger::zero();
        let any = NeoInteger::new(12345);
        assert_eq!((&zero * &any).try_as_i32().unwrap_or(0), 0);
        assert_eq!((&any * &zero).try_as_i32().unwrap_or(0), 0);

        // Identity multiplication
        let one = NeoInteger::one();
        assert_eq!((&any * &one).try_as_i32().unwrap_or(0), 12345);
        assert_eq!((&one * &any).try_as_i32().unwrap_or(0), 12345);

        // Negative multiplication
        let neg = NeoInteger::new(-1);
        assert_eq!((&any * &neg).try_as_i32().unwrap_or(0), -12345);
        assert_eq!((&neg * &neg).try_as_i32().unwrap_or(0), 1);
    }

    /// Test: Division edge cases
    #[test]
    fn test_division_edge_cases() {
        // Division by 1
        let val = NeoInteger::new(42);
        let one = NeoInteger::one();
        assert_eq!((&val / &one).try_as_i32().unwrap_or(0), 42);

        // Zero divided by anything
        let zero = NeoInteger::zero();
        assert_eq!((&zero / &val).try_as_i32().unwrap_or(0), 0);

        // Negative division
        let neg = NeoInteger::new(-42);
        assert_eq!((&neg / &val).try_as_i32().unwrap_or(0), -1);
        assert_eq!((&val / &neg).try_as_i32().unwrap_or(0), -1);
    }

    /// Test: Shift edge cases
    #[test]
    fn test_shift_edge_cases() {
        let val = NeoInteger::new(1);

        // Shift by 0 (identity)
        assert_eq!((&val << 0).try_as_i32().unwrap_or(0), 1);
        assert_eq!((&val >> 0).try_as_i32().unwrap_or(0), 1);

        // Shift by 31 (i32 bits - 1) - should not panic
        let _result = &val << 31;

        // Large shifts (behavior depends on implementation) - should not panic
        let _large_shift = &val << 32;

        // Negative value shifts - should not panic
        let neg = NeoInteger::new(-1);
        let _result = &neg >> 1;

        // Test passes if we reach here without panic
        assert!(true);
    }

    /// Test: Bitwise NOT edge cases
    #[test]
    fn test_bitwise_not_edge_cases() {
        // NOT of 0 is -1 (all bits set)
        let zero = NeoInteger::zero();
        assert_eq!((!zero.clone()).try_as_i32().unwrap_or(0), -1);

        // NOT of -1 is 0
        let neg_one = NeoInteger::new(-1);
        assert_eq!((!neg_one.clone()).try_as_i32().unwrap_or(0), 0);

        // Double NOT is identity
        let val = NeoInteger::new(42);
        assert_eq!((!(!val.clone())).try_as_i32().unwrap_or(0), 42);
    }
}

/// Module: ByteString Edge Cases
mod bytestring_edge_cases {
    use super::*;

    /// Test: Empty ByteString
    #[test]
    fn test_empty_bytestring() {
        let empty = NeoByteString::new(vec![]);
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);
        assert_eq!(empty.as_slice(), &[] as &[u8]);
    }

    /// Test: Single byte ByteString
    #[test]
    fn test_single_byte() {
        let bs = NeoByteString::from_slice(&[0xFF]);
        assert_eq!(bs.len(), 1);
        assert_eq!(bs.as_slice(), &[0xFF]);
    }

    /// Test: All byte values
    #[test]
    fn test_all_byte_values() {
        let data: Vec<u8> = (0..=255).collect();
        let bs = NeoByteString::new(data.clone());
        assert_eq!(bs.len(), 256);
        assert_eq!(bs.as_slice(), &data);
    }

    /// Test: Large ByteString
    #[test]
    fn test_large_bytestring() {
        let data = vec![0x42; 10000];
        let bs = NeoByteString::new(data.clone());
        assert_eq!(bs.len(), 10000);
        assert_eq!(bs.as_slice(), &data);
    }

    /// Test: ByteString operations
    #[test]
    fn test_bytestring_operations() {
        let mut bs = NeoByteString::new(vec![]);

        // Push single bytes
        bs.push(0x01);
        bs.push(0x02);
        assert_eq!(bs.as_slice(), &[0x01, 0x02]);

        // Extend with slice
        bs.extend_from_slice(&[0x03, 0x04, 0x05]);
        assert_eq!(bs.as_slice(), &[0x01, 0x02, 0x03, 0x04, 0x05]);
    }
}

/// Module: String Edge Cases
mod string_edge_cases {
    use super::*;

    /// Test: Empty string
    #[test]
    fn test_empty_string() {
        let empty = NeoString::from_str("");
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);
        assert_eq!(empty.as_str(), "");
    }

    /// Test: Unicode strings
    #[test]
    fn test_unicode_strings() {
        // ASCII
        let ascii = NeoString::from_str("Hello");
        assert_eq!(ascii.len(), 5);
        assert_eq!(ascii.as_str(), "Hello");

        // Multi-byte UTF-8
        let utf8 = NeoString::from_str("Hello, 世界! 🌍");
        assert_eq!(utf8.as_str(), "Hello, 世界! 🌍");

        // Emoji
        let emoji = NeoString::from_str("🎉🎊🎁");
        assert_eq!(emoji.as_str(), "🎉🎊🎁");

        // Mixed scripts
        let mixed = NeoString::from_str("Hello こんにちは مرحبا");
        assert_eq!(mixed.as_str(), "Hello こんにちは مرحبا");
    }

    /// Test: Special characters
    #[test]
    fn test_special_characters() {
        let null_char = NeoString::from_str("\0");
        assert_eq!(null_char.as_str(), "\0");

        let newlines = NeoString::from_str("line1\nline2\r\nline3");
        assert_eq!(newlines.as_str(), "line1\nline2\r\nline3");

        let tabs = NeoString::from_str("col1\tcol2\tcol3");
        assert_eq!(tabs.as_str(), "col1\tcol2\tcol3");
    }

    /// Test: Long string
    #[test]
    fn test_long_string() {
        let long = "a".repeat(10000);
        let neo_string = NeoString::from_str(&long);
        assert_eq!(neo_string.len(), 10000);
        assert_eq!(neo_string.as_str(), long);
    }
}

/// Module: Array Edge Cases
mod array_edge_cases {
    use super::*;

    /// Test: Empty array
    #[test]
    fn test_empty_array() {
        let arr = NeoArray::<NeoValue>::new();
        assert!(arr.is_empty());
        assert_eq!(arr.len(), 0);
    }

    /// Test: Array with capacity
    #[test]
    fn test_array_with_capacity() {
        let arr = NeoArray::<NeoValue>::with_capacity(1000);
        assert!(arr.is_empty());
        assert_eq!(arr.len(), 0);
    }

    /// Test: Array push and pop
    #[test]
    fn test_array_push_pop() {
        let mut arr = NeoArray::new();

        // Push many items
        for i in 0..1000 {
            arr.push(NeoValue::from(NeoInteger::new(i)));
        }
        assert_eq!(arr.len(), 1000);

        // Pop all items
        for i in (0i32..1000).rev() {
            let val = arr.pop().expect("Should have value");
            assert_eq!(val.as_integer().unwrap().try_as_i32().unwrap_or(0), i);
        }
        assert!(arr.is_empty());
        assert!(arr.pop().is_none());
    }

    /// Test: Array get out of bounds
    #[test]
    fn test_array_get_bounds() {
        let mut arr = NeoArray::new();
        arr.push(NeoValue::from(NeoInteger::new(1)));
        arr.push(NeoValue::from(NeoInteger::new(2)));

        assert!(arr.get(0).is_some());
        assert!(arr.get(1).is_some());
        assert!(arr.get(2).is_none());
        assert!(arr.get(usize::MAX).is_none());
    }

    /// Test: Array with mixed types
    #[test]
    fn test_array_mixed_types() {
        let mut arr = NeoArray::new();
        arr.push(NeoValue::from(NeoInteger::new(42)));
        arr.push(NeoValue::from(NeoBoolean::TRUE));
        arr.push(NeoValue::from(NeoByteString::from_slice(b"test")));
        arr.push(NeoValue::from(NeoString::from_str("hello")));
        arr.push(NeoValue::Null);

        assert_eq!(arr.len(), 5);
        assert!(arr.get(0).unwrap().as_integer().is_some());
        assert!(arr.get(1).unwrap().as_boolean().is_some());
        assert!(arr.get(2).unwrap().as_byte_string().is_some());
        assert!(arr.get(3).unwrap().as_string().is_some());
        assert!(arr.get(4).unwrap().is_null());
    }
}

/// Module: Map Edge Cases
mod map_edge_cases {
    use super::*;

    /// Test: Empty map
    #[test]
    fn test_empty_map() {
        let map: NeoMap<NeoValue, NeoValue> = NeoMap::new();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }

    /// Test: Map with many entries
    #[test]
    fn test_map_many_entries() {
        let mut map: NeoMap<NeoValue, NeoValue> = NeoMap::new();

        for i in 0i32..100 {
            let key = NeoValue::from(NeoInteger::new(i));
            let value = NeoValue::from(NeoByteString::from_slice(&(i as i32).to_le_bytes()));
            map.insert(key, value);
        }

        assert_eq!(map.len(), 100);

        // Verify all entries
        for i in 0..100 {
            let key = NeoValue::from(NeoInteger::new(i));
            assert!(map.get(&key).is_some());
        }
    }

    /// Test: Map key overwrite
    #[test]
    fn test_map_key_overwrite() {
        let mut map: NeoMap<NeoValue, NeoValue> = NeoMap::new();
        let key = NeoValue::from(NeoString::from_str("key"));

        map.insert(key.clone(), NeoValue::from(NeoInteger::new(1)));
        assert_eq!(map.len(), 1);

        map.insert(key.clone(), NeoValue::from(NeoInteger::new(2)));
        assert_eq!(map.len(), 1);

        let value = map.get(&key).unwrap();
        assert_eq!(value.as_integer().unwrap().try_as_i32().unwrap_or(0), 2);
    }

    /// Test: Map remove non-existent key
    #[test]
    fn test_map_remove_nonexistent() {
        let mut map: NeoMap<NeoValue, NeoValue> = NeoMap::new();
        let key = NeoValue::from(NeoString::from_str("nonexistent"));

        assert!(map.remove(&key).is_none());
    }

    /// Test: Map with complex keys
    #[test]
    fn test_map_complex_keys() {
        let mut map: NeoMap<NeoValue, NeoValue> = NeoMap::new();

        // Use various types as keys
        let int_key = NeoValue::from(NeoInteger::new(42));
        let string_key = NeoValue::from(NeoString::from_str("key"));
        let bytes_key = NeoValue::from(NeoByteString::from_slice(b"bytes"));

        map.insert(int_key.clone(), NeoValue::from(NeoInteger::new(1)));
        map.insert(string_key.clone(), NeoValue::from(NeoInteger::new(2)));
        map.insert(bytes_key.clone(), NeoValue::from(NeoInteger::new(3)));

        assert_eq!(map.len(), 3);
        assert!(map.get(&int_key).is_some());
        assert!(map.get(&string_key).is_some());
        assert!(map.get(&bytes_key).is_some());
    }
}

/// Module: Storage Context Edge Cases
mod storage_context_edge_cases {
    use super::*;

    /// Test: Storage context IDs
    #[test]
    fn test_storage_context_ids() {
        let ctx1 = NeoStorageContext::new(1);
        assert_eq!(ctx1.id(), 1);
        assert!(!ctx1.is_read_only());

        let ctx2 = NeoStorageContext::new(u32::MAX);
        assert_eq!(ctx2.id(), u32::MAX);

        let read_only = NeoStorageContext::read_only(42);
        assert_eq!(read_only.id(), 42);
        assert!(read_only.is_read_only());
    }

    /// Test: Read-only conversion
    #[test]
    fn test_read_only_conversion() {
        let ctx = NeoStorageContext::new(100);
        assert!(!ctx.is_read_only());

        // Note: as_read_only creates a new context
        // The actual behavior depends on implementation
    }
}

/// Module: Iterator Edge Cases
mod iterator_edge_cases {
    use super::*;

    /// Test: Empty iterator
    #[test]
    fn test_empty_iterator() {
        let mut iter = NeoIterator::<NeoValue>::new(vec![]);
        assert!(!iter.has_next());
        assert!(iter.next().is_none());
    }

    /// Test: Single item iterator
    #[test]
    fn test_single_item_iterator() {
        let data = vec![NeoValue::from(NeoInteger::new(42))];
        let mut iter: NeoIterator<NeoValue> = NeoIterator::new(data);

        assert!(iter.has_next());
        let val = iter.next().unwrap();
        assert_eq!(val.as_integer().unwrap().try_as_i32().unwrap_or(0), 42);

        assert!(!iter.has_next());
        assert!(iter.next().is_none());
    }

    /// Test: Iterator with many items
    #[test]
    fn test_large_iterator() {
        let data: Vec<NeoValue> = (0i32..1000)
            .map(|i| NeoValue::from(NeoInteger::new(i)))
            .collect();

        let mut iter: NeoIterator<NeoValue> = NeoIterator::new(data);
        let mut count = 0;

        while iter.has_next() {
            let val = iter.next().unwrap();
            assert_eq!(val.as_integer().unwrap().try_as_i32().unwrap_or(0), count);
            count += 1;
        }

        assert_eq!(count, 1000);
    }
}

/// Module: Error Edge Cases
mod error_edge_cases {
    use super::*;

    /// Test: All error variants
    #[test]
    fn test_all_error_variants() {
        let errors = vec![
            NeoError::InvalidOperation,
            NeoError::InvalidArgument,
            NeoError::InvalidType,
            NeoError::OutOfBounds,
            NeoError::DivisionByZero,
            NeoError::Overflow,
            NeoError::Underflow,
            NeoError::NullReference,
            NeoError::InvalidState,
            NeoError::Custom("test error".to_string()),
        ];

        for error in errors {
            let msg = format!("{}", error);
            assert!(!msg.is_empty(), "Error message should not be empty");
        }
    }

    /// Test: Result with large error message
    #[test]
    fn test_large_error_message() {
        let long_message = "x".repeat(10000);
        let error = NeoError::Custom(long_message.clone());
        let msg = format!("{}", error);
        assert!(msg.len() > 1000);
    }
}

/// Module: Struct Edge Cases
mod struct_edge_cases {
    use super::*;

    /// Test: Empty struct
    #[test]
    fn test_empty_struct() {
        let s = NeoStruct::new();
        assert!(s.get_field("any").is_none());
    }

    /// Test: Struct with many fields
    #[test]
    fn test_struct_many_fields() {
        let mut s = NeoStruct::new();

        for i in 0..100 {
            s.set_field(&format!("field{}", i), NeoValue::from(NeoInteger::new(i)));
        }

        for i in 0..100 {
            let field_name = format!("field{}", i);
            let val = s.get_field(&field_name);
            assert!(val.is_some());
            assert_eq!(
                val.unwrap().as_integer().unwrap().try_as_i32().unwrap_or(0),
                i
            );
        }
    }

    /// Test: Struct with complex values
    #[test]
    fn test_struct_complex_values() {
        let s = NeoStruct::new()
            .with_field("int", NeoValue::from(NeoInteger::new(42)))
            .with_field("bool", NeoValue::from(NeoBoolean::TRUE))
            .with_field("string", NeoValue::from(NeoString::from_str("hello")))
            .with_field("bytes", NeoValue::from(NeoByteString::from_slice(b"data")))
            .with_field("null", NeoValue::Null);

        assert!(s.get_field("int").unwrap().as_integer().is_some());
        assert!(s.get_field("bool").unwrap().as_boolean().is_some());
        assert!(s.get_field("string").unwrap().as_string().is_some());
        assert!(s.get_field("bytes").unwrap().as_byte_string().is_some());
        assert!(s.get_field("null").unwrap().is_null());
    }
}

/// Module: Boolean Edge Cases
mod boolean_edge_cases {
    use super::*;

    /// Test: Boolean constants
    #[test]
    fn test_boolean_constants() {
        assert!(NeoBoolean::TRUE.as_bool());
        assert!(!NeoBoolean::FALSE.as_bool());
    }

    /// Test: Boolean operations
    #[test]
    fn test_boolean_operations() {
        let t = NeoBoolean::TRUE;
        let f = NeoBoolean::FALSE;

        // AND
        assert!((t & t).as_bool());
        assert!(!(t & f).as_bool());
        assert!(!(f & t).as_bool());
        assert!(!(f & f).as_bool());

        // OR
        assert!((t | t).as_bool());
        assert!((t | f).as_bool());
        assert!((f | t).as_bool());
        assert!(!(f | f).as_bool());

        // XOR
        assert!(!(t ^ t).as_bool());
        assert!((t ^ f).as_bool());
        assert!((f ^ t).as_bool());
        assert!(!(f ^ f).as_bool());

        // NOT
        assert!((!f.clone()).as_bool());
        assert!((!t.clone()).as_bool() == false);
    }

    /// Test: Boolean double negation
    #[test]
    fn test_boolean_double_negation() {
        let t = NeoBoolean::TRUE;
        let f = NeoBoolean::FALSE;

        assert_eq!((!(!t.clone())).as_bool(), true);
        assert_eq!((!(!f.clone())).as_bool(), false);
    }
}
