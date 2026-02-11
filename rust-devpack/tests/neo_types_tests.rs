// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

// Comprehensive tests for Neo N3 types

use neo_devpack::prelude::*;
use neo_devpack::{NeoContractEvent, NeoContractManifest, NeoContractParameter};
use neo_types::{NeoContractABI, NeoContractPermission};

#[test]
fn test_neo_integer() {
    let int = NeoInteger::new(42);
    assert_eq!(int.as_i32_saturating(), 42);
    assert_eq!(int.as_u32_saturating(), 42);

    // Test arithmetic operations
    let int2 = NeoInteger::new(10);
    assert_eq!((&int + &int2).as_i32_saturating(), 52);
    assert_eq!((&int - &int2).as_i32_saturating(), 32);
    assert_eq!((&int * &int2).as_i32_saturating(), 420);
    assert_eq!((&int / &int2).as_i32_saturating(), 4);
    assert_eq!((&int % &int2).as_i32_saturating(), 2);

    // Test bitwise operations
    assert_eq!((&int & &int2).as_i32_saturating(), 10);
    assert_eq!((&int | &int2).as_i32_saturating(), 42);
    assert_eq!((&int ^ &int2).as_i32_saturating(), 32);
    assert_eq!((!int.clone()).as_i32_saturating(), -43);

    // Test shift operations
    assert_eq!((&int << 2).as_i32_saturating(), 168);
    assert_eq!((&int >> 2).as_i32_saturating(), 10);

    // Test constants
    assert_eq!(NeoInteger::zero().as_i32_saturating(), 0);
    assert_eq!(NeoInteger::one().as_i32_saturating(), 1);
    assert_eq!(NeoInteger::min_i32().as_i32_saturating(), i32::MIN);
    assert_eq!(NeoInteger::max_i32().as_i32_saturating(), i32::MAX);
}

#[test]
fn test_neo_boolean() {
    let true_val = NeoBoolean::new(true);
    let false_val = NeoBoolean::new(false);

    assert!(true_val.as_bool());
    assert!(!false_val.as_bool());

    // Test logical operations
    assert!((true_val & true_val).as_bool());
    assert!(!(true_val & false_val).as_bool());
    assert!((true_val | false_val).as_bool());
    assert!((true_val ^ false_val).as_bool());
    assert!((!false_val).as_bool());

    // Test constants
    assert!(NeoBoolean::TRUE.as_bool());
    assert!(!NeoBoolean::FALSE.as_bool());
}

#[test]
fn test_neo_byte_string() {
    let data = vec![0x00, 0x01, 0x02, 0x03];
    let bs = NeoByteString::new(data.clone());

    assert_eq!(bs.as_slice(), &data);
    assert_eq!(bs.len(), 4);
    assert!(!bs.is_empty());

    // Test from_slice
    let bs2 = NeoByteString::from_slice(&data);
    assert_eq!(bs2.as_slice(), &data);

    // Test push and extend
    let mut bs3 = NeoByteString::new(vec![]);
    bs3.push(0x04);
    bs3.extend_from_slice(&[0x05, 0x06]);
    assert_eq!(bs3.len(), 3);
    assert_eq!(bs3.as_slice(), &[0x04, 0x05, 0x06]);
}

#[test]
fn test_neo_string() {
    let s = NeoString::from_str("Hello, Neo!");
    assert_eq!(s.as_str(), "Hello, Neo!");
    assert_eq!(s.len(), 11);
    assert!(!s.is_empty());

    // Test from_str
    let s2 = NeoString::from_str("Test");
    assert_eq!(s2.as_str(), "Test");
}

#[test]
fn test_neo_array() {
    let mut array = NeoArray::new();
    assert!(array.is_empty());
    assert_eq!(array.len(), 0);

    // Test push and pop
    array.push(NeoValue::from(NeoInteger::new(1)));
    array.push(NeoValue::from(NeoInteger::new(2)));
    array.push(NeoValue::from(NeoInteger::new(3)));

    assert_eq!(array.len(), 3);
    assert!(!array.is_empty());

    // Test get
    assert!(array.get(0).is_some());
    assert!(array.get(1).is_some());
    assert!(array.get(2).is_some());
    assert!(array.get(3).is_none());

    // Test pop
    let popped = array.pop();
    assert!(popped.is_some());
    assert_eq!(array.len(), 2);

    // Test with capacity
    let array_with_cap = NeoArray::<NeoValue>::with_capacity(10);
    assert_eq!(array_with_cap.len(), 0);
}

#[test]
fn test_neo_map() {
    let mut map = NeoMap::new();
    assert!(map.is_empty());
    assert_eq!(map.len(), 0);

    // Test insert and get
    let key1 = NeoValue::from(NeoString::from_str("key1"));
    let value1 = NeoValue::from(NeoInteger::new(42));

    map.insert(key1.clone(), value1.clone());
    assert_eq!(map.len(), 1);
    assert!(!map.is_empty());

    // Test get
    assert!(map.get(&key1).is_some());
    assert!(map
        .get(&NeoValue::from(NeoString::from_str("nonexistent")))
        .is_none());

    // Test remove
    let removed = map.remove(&key1);
    assert!(removed.is_some());
    assert_eq!(map.len(), 0);
    assert!(map.is_empty());
}

#[test]
fn test_neo_struct() {
    let struct_data = NeoStruct::new();

    // Test with_field
    let struct_with_field = struct_data
        .with_field("name", NeoValue::from(NeoString::from_str("Test")))
        .with_field("value", NeoValue::from(NeoInteger::new(42)));

    // Test get_field
    assert!(struct_with_field.get_field("name").is_some());
    assert!(struct_with_field.get_field("value").is_some());
    assert!(struct_with_field.get_field("nonexistent").is_none());

    // Test set_field
    let mut mutable_struct = NeoStruct::new();
    mutable_struct.set_field("name", NeoValue::from(NeoString::from_str("Updated")));
    mutable_struct.set_field("value", NeoValue::from(NeoInteger::new(100)));

    assert!(mutable_struct.get_field("name").is_some());
    assert!(mutable_struct.get_field("value").is_some());
}

#[test]
fn test_neo_value() {
    // Test Integer value
    let int_value = NeoValue::from(NeoInteger::new(42));
    assert!(!int_value.is_null());
    assert!(int_value.as_integer().is_some());
    assert_eq!(int_value.as_integer().unwrap().as_i32_saturating(), 42);

    // Test Boolean value
    let bool_value = NeoValue::from(NeoBoolean::TRUE);
    assert!(!bool_value.is_null());
    assert!(bool_value.as_boolean().is_some());
    assert!(bool_value.as_boolean().unwrap().as_bool());

    // Test ByteString value
    let bs_value = NeoValue::from(NeoByteString::from_slice(b"test"));
    assert!(!bs_value.is_null());
    assert!(bs_value.as_byte_string().is_some());
    assert_eq!(bs_value.as_byte_string().unwrap().len(), 4);

    // Test String value
    let string_value = NeoValue::from(NeoString::from_str("hello"));
    assert!(!string_value.is_null());
    assert!(string_value.as_string().is_some());
    assert_eq!(string_value.as_string().unwrap().as_str(), "hello");

    // Test Array value
    let mut array = NeoArray::new();
    array.push(NeoValue::from(NeoInteger::new(1)));
    let array_value = NeoValue::from(array);
    assert!(!array_value.is_null());
    assert!(array_value.as_array().is_some());
    assert_eq!(array_value.as_array().unwrap().len(), 1);

    // Test Map value
    let mut map = NeoMap::new();
    map.insert(
        NeoValue::from(NeoString::from_str("key")),
        NeoValue::from(NeoInteger::new(42)),
    );
    let map_value = NeoValue::from(map);
    assert!(!map_value.is_null());
    assert!(map_value.as_map().is_some());
    assert_eq!(map_value.as_map().unwrap().len(), 1);

    // Test Struct value
    let struct_data =
        NeoStruct::new().with_field("name", NeoValue::from(NeoString::from_str("Test")));
    let struct_value = NeoValue::from(struct_data);
    assert!(!struct_value.is_null());
    assert!(struct_value.as_struct().is_some());

    // Test Null value
    let null_value = NeoValue::Null;
    assert!(null_value.is_null());
    assert!(null_value.as_integer().is_none());
    assert!(null_value.as_boolean().is_none());
    assert!(null_value.as_byte_string().is_none());
    assert!(null_value.as_string().is_none());
    assert!(null_value.as_array().is_none());
    assert!(null_value.as_map().is_none());
    assert!(null_value.as_struct().is_none());
}

#[test]
fn test_neo_iterator() {
    let data = vec![
        NeoValue::from(NeoInteger::new(1)),
        NeoValue::from(NeoInteger::new(2)),
        NeoValue::from(NeoInteger::new(3)),
    ];

    let mut iterator = NeoIterator::new(data);

    // Test has_next
    assert!(iterator.has_next());

    // Test next
    let first = iterator.next();
    assert!(first.is_some());
    assert_eq!(first.unwrap().as_integer().unwrap().as_i32_saturating(), 1);

    assert!(iterator.has_next());
    let second = iterator.next();
    assert!(second.is_some());
    assert_eq!(second.unwrap().as_integer().unwrap().as_i32_saturating(), 2);

    assert!(iterator.has_next());
    let third = iterator.next();
    assert!(third.is_some());
    assert_eq!(third.unwrap().as_integer().unwrap().as_i32_saturating(), 3);

    // Test end of iterator
    assert!(!iterator.has_next());
    let fourth = iterator.next();
    assert!(fourth.is_none());
}

#[test]
fn test_neo_storage_context() {
    let context = NeoStorageContext::new(42);
    assert_eq!(context.id(), 42);
    assert!(!context.is_read_only());

    let read_only_context = NeoStorageContext::read_only(1);
    assert_eq!(read_only_context.id(), 1);
    assert!(read_only_context.is_read_only());
}

#[test]
fn test_neo_contract_manifest() {
    let manifest = NeoContractManifest {
        name: "TestContract".to_string(),
        version: "1.0.0".to_string(),
        author: "Test Author".to_string(),
        email: "test@example.com".to_string(),
        description: "Test contract".to_string(),
        abi: NeoContractABI {
            hash: "0x12345678".to_string(),
            methods: vec![],
            events: vec![],
        },
        permissions: vec![],
        trusts: vec![],
        supported_standards: vec![],
    };

    assert_eq!(manifest.name, "TestContract");
    assert_eq!(manifest.version, "1.0.0");
    assert_eq!(manifest.author, "Test Author");
    assert_eq!(manifest.email, "test@example.com");
    assert_eq!(manifest.description, "Test contract");
}

#[test]
fn test_neo_contract_method() {
    let method = NeoContractMethod {
        name: "test_method".to_string(),
        parameters: vec![NeoContractParameter {
            name: "param1".to_string(),
            r#type: "Integer".to_string(),
        }],
        return_type: "Boolean".to_string(),
        offset: 0,
        safe: true,
    };

    assert_eq!(method.name, "test_method");
    assert_eq!(method.parameters.len(), 1);
    assert_eq!(method.parameters[0].name, "param1");
    assert_eq!(method.parameters[0].r#type, "Integer");
    assert_eq!(method.return_type, "Boolean");
    assert_eq!(method.offset, 0);
    assert!(method.safe);
}

#[test]
fn test_neo_contract_event() {
    let event = NeoContractEvent {
        name: "TestEvent".to_string(),
        parameters: vec![NeoContractParameter {
            name: "event_param".to_string(),
            r#type: "String".to_string(),
        }],
    };

    assert_eq!(event.name, "TestEvent");
    assert_eq!(event.parameters.len(), 1);
    assert_eq!(event.parameters[0].name, "event_param");
    assert_eq!(event.parameters[0].r#type, "String");
}

#[test]
fn test_neo_contract_permission() {
    let permission = NeoContractPermission {
        contract: "0x12345678".to_string(),
        methods: vec!["method1".to_string(), "method2".to_string()],
    };

    assert_eq!(permission.contract, "0x12345678");
    assert_eq!(permission.methods.len(), 2);
    assert_eq!(permission.methods[0], "method1");
    assert_eq!(permission.methods[1], "method2");
}

#[test]
fn test_neo_error() {
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
        NeoError::Custom("Test error".to_string()),
    ];

    let expected_codes = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    for (error, expected_code) in errors.into_iter().zip(expected_codes) {
        let error_string = format!("{}", error);
        assert!(!error_string.is_empty());
        assert_eq!(error.status_code(), expected_code);
    }
}

#[test]
fn test_neo_result() {
    // Test Ok result
    fn ok_integer() -> NeoResult<NeoInteger> {
        Ok(NeoInteger::new(42))
    }

    let ok_result = ok_integer();
    assert!(ok_result.is_ok());
    assert_eq!(ok_result.unwrap().as_i32_saturating(), 42);

    // Test Err result
    fn err_integer() -> NeoResult<NeoInteger> {
        Err(NeoError::InvalidArgument)
    }

    let err_result = err_integer();
    assert!(err_result.is_err());
    assert_eq!(err_result.unwrap_err(), NeoError::InvalidArgument);
}

#[test]
fn test_neo_contract_trait() {
    struct TestContract;

    impl NeoContractTrait for TestContract {
        fn name() -> &'static str {
            "TestContract"
        }
        fn version() -> &'static str {
            "1.0.0"
        }
        fn author() -> &'static str {
            "Test Author"
        }
        fn description() -> &'static str {
            "Test contract"
        }
    }

    assert_eq!(TestContract::name(), "TestContract");
    assert_eq!(TestContract::version(), "1.0.0");
    assert_eq!(TestContract::author(), "Test Author");
    assert_eq!(TestContract::description(), "Test contract");
}

#[test]
fn test_neo_contract_entry_trait() {
    struct TestContract;

    impl NeoContractEntry for TestContract {
        fn deploy() -> NeoResult<()> {
            Ok(())
        }
        fn update() -> NeoResult<()> {
            Ok(())
        }
        fn destroy() -> NeoResult<()> {
            Ok(())
        }
    }

    assert!(TestContract::deploy().is_ok());
    assert!(TestContract::update().is_ok());
    assert!(TestContract::destroy().is_ok());
}

#[test]
fn test_neo_contract_method_trait() {
    struct TestMethod;

    impl NeoContractMethodTrait for TestMethod {
        fn name() -> &'static str {
            "test_method"
        }
        fn parameters() -> &'static [&'static str] {
            &["Integer", "String"]
        }
        fn return_type() -> &'static str {
            "Boolean"
        }
        fn execute(_args: &[NeoValue]) -> NeoResult<NeoValue> {
            Ok(NeoValue::from(NeoBoolean::TRUE))
        }
    }

    assert_eq!(TestMethod::name(), "test_method");
    assert_eq!(TestMethod::parameters(), &["Integer", "String"]);
    assert_eq!(TestMethod::return_type(), "Boolean");

    let result = TestMethod::execute(&[]);
    assert!(result.is_ok());
    assert!(result.unwrap().as_boolean().unwrap().as_bool());
}
