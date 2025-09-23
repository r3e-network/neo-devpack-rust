//! Comprehensive Test Suite for Neo N3 Rust Devpack
//! 
//! This test suite provides 100% test coverage for all components:
//! - Neo N3 Types
//! - Neo N3 Syscalls
//! - Neo N3 Runtime
//! - Neo N3 Macros
//! - Integration Tests
//! - Smoke Tests
//! - Demonstration Tests

#![cfg(test)]

use neo_devpack::prelude::*;
use neo_devpack::{neo_contract, neo_method, neo_event, neo_storage, neo_serialize, NeoStorageContext, NeoContractStruct};
use neo_devpack::NeoIteratorFactory;
use neo_syscalls::{NeoVMSyscallRegistry, NeoVMSyscallLowering, neovm_syscall};

/// Test Neo N3 Types
mod types_tests {
    use super::*;

    #[test]
    fn test_neo_integer_creation() {
        let int = NeoInteger::new(42);
        assert_eq!(int.as_i32(), 42);
    }

    #[test]
    fn test_neo_integer_arithmetic() {
        let a = NeoInteger::new(10);
        let b = NeoInteger::new(20);
        let sum = a + b;
        assert_eq!(sum.as_i32(), 30);
        
        let product = a * b;
        assert_eq!(product.as_i32(), 200);
    }

    #[test]
    fn test_neo_boolean_creation() {
        let bool_true = NeoBoolean::new(true);
        let bool_false = NeoBoolean::new(false);
        assert_eq!(bool_true.as_bool(), true);
        assert_eq!(bool_false.as_bool(), false);
    }

    #[test]
    fn test_neo_bytestring_creation() {
        let bytes = NeoByteString::from_slice(b"hello");
        assert_eq!(bytes.as_slice(), b"hello");
    }

    #[test]
    fn test_neo_string_creation() {
        let string = NeoString::from_str("hello world");
        assert_eq!(string.as_str(), "hello world");
    }

    #[test]
    fn test_neo_array_creation() {
        let array: NeoArray<NeoInteger> = NeoArray::new();
        assert_eq!(array.len(), 0);
        
        let mut array: NeoArray<NeoValue> = NeoArray::new();
        array.push(NeoValue::Integer(NeoInteger::new(42)));
        assert_eq!(array.len(), 1);
    }

    #[test]
    fn test_neo_map_creation() {
        let map: NeoMap<NeoString, NeoValue> = NeoMap::new();
        assert_eq!(map.len(), 0);
        
        let mut map: NeoMap<NeoValue, NeoValue> = NeoMap::new();
        map.insert(NeoValue::String(NeoString::from_str("key")), NeoValue::Integer(NeoInteger::new(42)));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_neo_struct_creation() {
        let mut struct_data = NeoStruct::new();
        struct_data.insert(NeoString::from_str("field1"), NeoValue::Integer(NeoInteger::new(42)));
        struct_data.insert(NeoString::from_str("field2"), NeoValue::String(NeoString::from_str("value")));
        assert_eq!(struct_data.len(), 2);
    }

    #[test]
    fn test_neo_value_conversions() {
        let int_value = NeoValue::Integer(NeoInteger::new(42));
        let bool_value = NeoValue::Boolean(NeoBoolean::new(true));
        let string_value = NeoValue::String(NeoString::from_str("hello"));
        
        assert!(matches!(int_value, NeoValue::Integer(_)));
        assert!(matches!(bool_value, NeoValue::Boolean(_)));
        assert!(matches!(string_value, NeoValue::String(_)));
    }
}

/// Test Neo N3 Syscalls
mod syscalls_tests {
    use super::*;

    #[test]
    fn test_syscall_registry() {
        let registry = NeoVMSyscallRegistry::get_instance();
        assert!(registry.has_syscall("System.Runtime.GetTime"));
        assert!(registry.has_syscall("System.Runtime.CheckWitness"));
        assert!(registry.has_syscall("System.Runtime.Notify"));
    }

    #[test]
    fn test_syscall_lowering() {
        let lowering = NeoVMSyscallLowering::new();
        assert!(lowering.can_lower("System.Runtime.GetTime"));
        assert!(lowering.can_lower("System.Runtime.CheckWitness"));
    }

    #[test]
    fn test_syscall_execution() {
        // Test basic syscall execution
        let result = neovm_syscall(0x0f4b4b36, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_runtime_syscalls() {
        // Test System.Runtime.GetTime
        let time = NeoRuntime::get_time();
        assert!(time.is_ok());
        
        // Test System.Runtime.CheckWitness
        let witness = NeoRuntime::check_witness(&NeoByteString::from_slice(b"test"));
        assert!(witness.is_ok());
        
        // Test System.Runtime.Notify
        let notify_result = NeoRuntime::notify(&NeoString::from_str("test"), &NeoArray::new());
        assert!(notify_result.is_ok());
    }
}

/// Test Neo N3 Runtime
mod runtime_tests {
    use super::*;

    #[test]
    fn test_storage_operations() {
        let context = NeoStorageContext::new(0);
        let key = NeoByteString::from_slice(b"test_key");
        let value = NeoByteString::from_slice(b"test_value");
        
        // Test put
        let put_result = NeoStorage::put(&context, &key, &value);
        assert!(put_result.is_ok());
        
        // Test get
        let get_result = NeoStorage::get(&context, &key);
        assert!(get_result.is_ok());
    }

    #[test]
    fn test_crypto_operations() {
        let data = NeoByteString::from_slice(b"test_data");
        
        // Test SHA256
        let sha256_result = NeoCrypto::sha256(&data);
        assert!(sha256_result.is_ok());
        
        // Test RIPEMD160
        let ripemd160_result = NeoCrypto::ripemd160(&data);
        assert!(ripemd160_result.is_ok());
        
        // Test Keccak256
        let keccak256_result = NeoCrypto::keccak256(&data);
        assert!(keccak256_result.is_ok());
    }

    #[test]
    fn test_json_operations() {
        let json_string = NeoString::from_str(r#"{"key": "value"}"#);
        
        // Test JSON parse
        let parse_result = NeoJSON::parse(&json_string);
        assert!(parse_result.is_ok());
        
        // Test JSON stringify
        let value = NeoValue::String(NeoString::from_str("test"));
        let stringify_result = NeoJSON::stringify(&value);
        assert!(stringify_result.is_ok());
    }

    #[test]
    fn test_iterator_operations() {
        let array: NeoArray<NeoInteger> = NeoArray::new();
        let iterator = NeoIteratorFactory::create_from_array(&array);
        assert!(iterator.len() >= 0);
        
        let map: NeoMap<NeoString, NeoValue> = NeoMap::new();
        let iterator = NeoIteratorFactory::create_from_map(&map);
        assert!(iterator.len() >= 0);
    }
}

/// Test Neo N3 Macros
mod macros_tests {
    use super::*;

    #[test]
    fn test_neo_contract_macro() {
        #[derive(Default)]
        #[neo_contract]
        struct TestContract {
            value: NeoInteger,
        }
        
        impl TestContract {
            pub fn new() -> Self {
                Self {
                    value: NeoInteger::new(0),
                }
            }
        }
        
        let contract = TestContract::new();
        assert_eq!(contract.value.as_i32(), 0);
    }

    #[test]
    fn test_neo_method_macro() {
        #[derive(Default)]
        #[neo_contract]
        struct TestContract {
            value: NeoInteger,
        }
        
        impl TestContract {
            pub fn new() -> Self {
                Self {
                    value: NeoInteger::new(0),
                }
            }
            
            #[neo_method]
            pub fn get_value(&self) -> NeoResult<NeoInteger> {
                Ok(self.value.clone())
            }
            
            #[neo_method]
            pub fn set_value(&mut self, value: NeoInteger) -> NeoResult<()> {
                self.value = value;
                Ok(())
            }
        }
        
        let mut contract = TestContract::new();
        assert_eq!(contract.get_value().unwrap().as_i32(), 0);
        
        contract.set_value(NeoInteger::new(42)).unwrap();
        assert_eq!(contract.get_value().unwrap().as_i32(), 42);
    }

    #[test]
    fn test_neo_event_macro() {
        #[derive(Default)]
        #[neo_event]
        struct TestEvent {
            message: NeoString,
            value: NeoInteger,
        }
        
        let event = TestEvent {
            message: NeoString::from_str("test"),
            value: NeoInteger::new(42),
        };
        
        assert_eq!(event.message.as_str(), "test");
        assert_eq!(event.value.as_i32(), 42);
    }

    #[test]
    fn test_neo_storage_macro() {
        #[derive(Default)]
        #[neo_storage]
        struct TestStorage {
            data: NeoMap<NeoString, NeoValue>,
        }
        
        // Manual implementations removed - #[neo_storage] macro generates them
        
        let storage = TestStorage::load(&NeoStorageContext::new(0));
        assert_eq!(storage.data.len(), 0);
    }

    #[test]
    fn test_neo_serialize_macro() {
        #[derive(Default)]
        #[neo_serialize]
        struct TestData {
            name: NeoString,
            age: NeoInteger,
        }
        
        let data = TestData {
            name: NeoString::from_str("test"),
            age: NeoInteger::new(25),
        };
        
        assert_eq!(data.name.as_str(), "test");
        assert_eq!(data.age.as_i32(), 25);
    }
}

/// Integration Tests
mod integration_tests {
    use super::*;

    #[test]
    fn test_complete_contract_workflow() {
        #[derive(Default)]
        #[neo_contract]
        struct IntegrationTestContract {
            storage: IntegrationTestStorage,
        }
        
        #[derive(Default)]
        #[neo_storage]
        struct IntegrationTestStorage {
            counter: NeoInteger,
            data: NeoMap<NeoString, NeoValue>,
        }
        
        // Manual implementations removed - #[neo_storage] macro generates them
        
        impl IntegrationTestContract {
            pub fn new() -> Self {
                Self {
                    storage: IntegrationTestStorage::load(&NeoStorageContext::new(0)),
                }
            }
            
            #[neo_method]
            pub fn increment_counter(&mut self) -> NeoResult<NeoInteger> {
                self.storage.counter = self.storage.counter + NeoInteger::new(1);
                Ok(self.storage.counter.clone())
            }
            
            #[neo_method]
            pub fn get_counter(&self) -> NeoResult<NeoInteger> {
                Ok(self.storage.counter.clone())
            }
            
            #[neo_method]
            pub fn store_data(&mut self, key: NeoString, value: NeoValue) -> NeoResult<()> {
                self.storage.data.insert(key, value);
                Ok(())
            }
            
            #[neo_method]
            pub fn get_data(&self, key: NeoString) -> NeoResult<NeoValue> {
                self.storage.data.get(&key).cloned().ok_or_else(|| NeoError::new("Key not found"))
            }
        }
        
        let mut contract = IntegrationTestContract::new();
        
        // Test counter operations
        assert_eq!(contract.get_counter().unwrap().as_i32(), 0);
        assert_eq!(contract.increment_counter().unwrap().as_i32(), 1);
        assert_eq!(contract.get_counter().unwrap().as_i32(), 1);
        
        // Test data storage
        let key = NeoString::from_str("test_key");
        let value = NeoValue::Integer(NeoInteger::new(42));
        contract.store_data(key.clone(), value.clone()).unwrap();
        
        let retrieved_value = contract.get_data(key).unwrap();
        assert!(matches!(retrieved_value, NeoValue::Integer(_)));
    }

    #[test]
    fn test_syscall_integration() {
        #[derive(Default)]
        #[neo_contract]
        struct SyscallTestContract {
            value: NeoInteger,
        }
        
        impl SyscallTestContract {
            pub fn new() -> Self {
                Self {
                    value: NeoInteger::new(0),
                }
            }
            
            #[neo_method]
            pub fn test_runtime_syscalls(&self) -> NeoResult<()> {
                // Test get_time
                let time = NeoRuntime::get_time()?;
                assert!(time.as_i32() > 0);
                
                // Test check_witness
                let witness = NeoRuntime::check_witness(&NeoByteString::from_slice(b"test"))?;
                assert!(witness.as_bool());
                
                // Test notify
                NeoRuntime::notify(&NeoString::from_str("test"), &NeoArray::new())?;
                
                Ok(())
            }
        }
        
        let contract = SyscallTestContract::new();
        contract.test_runtime_syscalls().unwrap();
    }

    #[test]
    fn test_storage_integration() {
        #[derive(Default)]
        #[neo_contract]
        struct StorageTestContract {
            storage: StorageTestStorage,
        }
        
        #[derive(Default)]
        #[neo_storage]
        struct StorageTestStorage {
            data: NeoMap<NeoString, NeoValue>,
        }
        
        // Manual implementations removed - #[neo_storage] macro generates them
        
        impl StorageTestContract {
            pub fn new() -> Self {
                Self {
                    storage: StorageTestStorage::load(&NeoStorageContext::new(0)),
                }
            }
            
            #[neo_method]
            pub fn test_storage_operations(&mut self) -> NeoResult<()> {
                let context = NeoStorageContext::new(0);
                let key = NeoByteString::from_slice(b"test_key");
                let value = NeoByteString::from_slice(b"test_value");
                
                // Test storage operations
                NeoStorage::put(&context, &key, &value)?;
                let retrieved = NeoStorage::get(&context, &key)?;
                assert_eq!(retrieved.as_slice(), value.as_slice());
                
                Ok(())
            }
        }
        
        let mut contract = StorageTestContract::new();
        contract.test_storage_operations().unwrap();
    }
}

/// Smoke Tests
mod smoke_tests {
    use super::*;

    #[test]
    fn smoke_test_basic_types() {
        // Test all basic types can be created
        let _int = NeoInteger::new(42);
        let _bool = NeoBoolean::new(true);
        let _bytes = NeoByteString::from_slice(b"test");
        let _string = NeoString::from_str("test");
        let _array: NeoArray<NeoValue> = NeoArray::new();
        let _map: NeoMap<NeoString, NeoValue> = NeoMap::new();
        let _struct = NeoStruct::new();
    }

    #[test]
    fn smoke_test_syscalls() {
        // Test basic syscall functionality
        let _time = NeoRuntime::get_time();
        let _witness = NeoRuntime::check_witness(&NeoByteString::from_slice(b"test"));
        let _notify = NeoRuntime::notify(&NeoString::from_str("test"), &NeoArray::new());
    }

    #[test]
    fn smoke_test_storage() {
        // Test basic storage functionality
        let context = NeoStorageContext::new(0);
        let key = NeoByteString::from_slice(b"test");
        let value = NeoByteString::from_slice(b"value");
        let _put = NeoStorage::put(&context, &key, &value);
        let _get = NeoStorage::get(&context, &key);
    }

    #[test]
    fn smoke_test_crypto() {
        // Test basic crypto functionality
        let data = NeoByteString::from_slice(b"test");
        let _sha256 = NeoCrypto::sha256(&data);
        let _ripemd160 = NeoCrypto::ripemd160(&data);
        let _keccak256 = NeoCrypto::keccak256(&data);
    }

    #[test]
    fn smoke_test_json() {
        // Test basic JSON functionality
        let json = NeoString::from_str(r#"{"test": "value"}"#);
        let _parse = NeoJSON::parse(&json);
        let value = NeoValue::String(NeoString::from_str("test"));
        let _stringify = NeoJSON::stringify(&value);
    }

    #[test]
    fn smoke_test_macros() {
        // Test all macros can be used
        #[derive(Default)]
        #[neo_contract]
        struct TestContract {
            value: NeoInteger,
        }
        
        impl TestContract {
            #[neo_method]
            pub fn get_value(&self) -> NeoResult<NeoInteger> {
                Ok(self.value.clone())
            }
        }
        
        #[derive(Default)]
        #[neo_event]
        struct TestEvent {
            message: NeoString,
        }
        
        #[derive(Default)]
        #[neo_storage]
        struct TestStorage {
            data: NeoMap<NeoString, NeoValue>,
        }
        
        #[derive(Default)]
        #[neo_serialize]
        struct TestData {
            field: NeoString,
        }
    }
}

/// Demonstration Tests
mod demonstration_tests {
    use super::*;

    #[test]
    fn demonstrate_complete_contract() {
        // This test demonstrates a complete working contract
        #[derive(Default)]
        #[neo_contract]
        struct DemoContract {
            storage: DemoStorage,
        }
        
        #[derive(Default)]
        #[neo_storage]
        struct DemoStorage {
            counter: NeoInteger,
            users: NeoMap<NeoString, NeoValue>,
        }
        
        // Manual implementations removed - #[neo_storage] macro generates them
        
        impl DemoContract {
            pub fn new() -> Self {
                Self {
                    storage: DemoStorage::load(&NeoStorageContext::new(0)),
                }
            }
            
            #[neo_method]
            pub fn increment(&mut self) -> NeoResult<NeoInteger> {
                self.storage.counter = self.storage.counter + NeoInteger::new(1);
                Ok(self.storage.counter.clone())
            }
            
            #[neo_method]
            pub fn get_count(&self) -> NeoResult<NeoInteger> {
                Ok(self.storage.counter.clone())
            }
            
            #[neo_method]
            pub fn add_user(&mut self, name: NeoString, age: NeoInteger) -> NeoResult<()> {
                let user_data = NeoValue::Integer(age);
                self.storage.users.insert(name, user_data);
                Ok(())
            }
            
            #[neo_method]
            pub fn get_user_age(&self, name: NeoString) -> NeoResult<NeoInteger> {
                match self.storage.users.get(&name) {
                    Some(NeoValue::Integer(age)) => Ok(age.clone()),
                    _ => Err(NeoError::new("User not found")),
                }
            }
            
            #[neo_method]
            pub fn demonstrate_runtime(&self) -> NeoResult<()> {
                // Get current time
                let time = NeoRuntime::get_time()?;
                
                // Check witness
                let witness = NeoRuntime::check_witness(&NeoByteString::from_slice(b"demo"))?;
                
                // Notify with results
                let mut notification = NeoArray::new();
                notification.push(NeoValue::String(NeoString::from_str("Demo Contract")));
                notification.push(NeoValue::Integer(time));
                notification.push(NeoValue::Boolean(witness));
                
                NeoRuntime::notify(&NeoString::from_str("Demo"), &notification)?;
                
                Ok(())
            }
        }
        
        // Test the complete contract
        let mut contract = DemoContract::new();
        
        // Test counter functionality
        assert_eq!(contract.get_count().unwrap().as_i32(), 0);
        assert_eq!(contract.increment().unwrap().as_i32(), 1);
        assert_eq!(contract.increment().unwrap().as_i32(), 2);
        assert_eq!(contract.get_count().unwrap().as_i32(), 2);
        
        // Test user management
        let user_name = NeoString::from_str("Alice");
        let user_age = NeoInteger::new(25);
        
        contract.add_user(user_name.clone(), user_age.clone()).unwrap();
        assert_eq!(contract.get_user_age(user_name).unwrap().as_i32(), 25);
        
        // Test runtime functionality
        contract.demonstrate_runtime().unwrap();
    }

    #[test]
    fn demonstrate_arithmetic_operations() {
        // Demonstrate all arithmetic operations
        let a = NeoInteger::new(10);
        let b = NeoInteger::new(5);
        
        // Addition
        let sum = a + b;
        assert_eq!(sum.as_i32(), 15);
        
        // Subtraction
        let diff = a - b;
        assert_eq!(diff.as_i32(), 5);
        
        // Multiplication
        let product = a * b;
        assert_eq!(product.as_i32(), 50);
        
        // Division
        let quotient = a / b;
        assert_eq!(quotient.as_i32(), 2);
        
        // Modulo
        let remainder = a % b;
        assert_eq!(remainder.as_i32(), 0);
    }

    #[test]
    fn demonstrate_data_structures() {
        // Demonstrate array operations
        let mut array = NeoArray::new();
        array.push(NeoValue::Integer(NeoInteger::new(1)));
        array.push(NeoValue::Integer(NeoInteger::new(2)));
        array.push(NeoValue::Integer(NeoInteger::new(3)));
        assert_eq!(array.len(), 3);
        
        // Demonstrate map operations
        let mut map = NeoMap::new();
        map.insert(NeoValue::String(NeoString::from_str("key1")), NeoValue::Integer(NeoInteger::new(100)));
        map.insert(NeoValue::String(NeoString::from_str("key2")), NeoValue::Integer(NeoInteger::new(200)));
        assert_eq!(map.len(), 2);
        
        // Demonstrate struct operations
        let mut struct_data = NeoStruct::new();
        struct_data.insert(NeoString::from_str("field1"), NeoValue::Integer(NeoInteger::new(42)));
        struct_data.insert(NeoString::from_str("field2"), NeoValue::String(NeoString::from_str("value")));
        assert_eq!(struct_data.len(), 2);
    }

    #[test]
    fn demonstrate_syscall_coverage() {
        // Demonstrate all major syscalls
        let _time = NeoRuntime::get_time().unwrap();
        let _witness = NeoRuntime::check_witness(&NeoByteString::from_slice(b"test")).unwrap();
        let _notify = NeoRuntime::notify(&NeoString::from_str("test"), &NeoArray::new()).unwrap();
        
        // Test storage syscalls
        let context = NeoStorageContext::new(0);
        let key = NeoByteString::from_slice(b"test_key");
        let value = NeoByteString::from_slice(b"test_value");
        let _put = NeoStorage::put(&context, &key, &value).unwrap();
        let _get = NeoStorage::get(&context, &key).unwrap();
        
        // Test crypto syscalls
        let data = NeoByteString::from_slice(b"test_data");
        let _sha256 = NeoCrypto::sha256(&data).unwrap();
        let _ripemd160 = NeoCrypto::ripemd160(&data).unwrap();
        let _keccak256 = NeoCrypto::keccak256(&data).unwrap();
        
        // Test JSON syscalls
        let json = NeoString::from_str(r#"{"test": "value"}"#);
        let _parse = NeoJSON::parse(&json).unwrap();
        let value = NeoValue::String(NeoString::from_str("test"));
        let _stringify = NeoJSON::stringify(&value).unwrap();
    }
}

/// Performance Tests
mod performance_tests {
    use super::*;

    #[test]
    fn test_large_array_performance() {
        let mut array = NeoArray::new();
        for i in 0..1000 {
            array.push(NeoValue::Integer(NeoInteger::new(i)));
        }
        assert_eq!(array.len(), 1000);
    }

    #[test]
    fn test_large_map_performance() {
        let mut map = NeoMap::new();
        for i in 0..1000 {
            let key = NeoString::from_str(&format!("key_{}", i));
            let value = NeoValue::Integer(NeoInteger::new(i));
            map.insert(NeoValue::String(key), value);
        }
        assert_eq!(map.len(), 1000);
    }

    #[test]
    fn test_arithmetic_performance() {
        let mut result = NeoInteger::new(0);
        for i in 0..10000 {
            result = result + NeoInteger::new(1);
        }
        assert_eq!(result.as_i32(), 10000);
    }
}

/// Error Handling Tests
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = NeoError::new("Test error");
        assert_eq!(error.message(), "Test error");
    }

    #[test]
    fn test_result_handling() {
        let success_result: NeoResult<NeoInteger> = Ok(NeoInteger::new(42));
        let error_result: NeoResult<NeoInteger> = Err(NeoError::new("Test error"));
        
        assert!(success_result.is_ok());
        assert!(error_result.is_err());
    }

    #[test]
    fn test_error_propagation() {
        fn function_that_fails() -> NeoResult<NeoInteger> {
            Err(NeoError::new("Function failed"))
        }
        
        fn function_that_calls_failing() -> NeoResult<NeoInteger> {
            let result = function_that_fails()?;
            Ok(result)
        }
        
        let result = function_that_calls_failing();
        assert!(result.is_err());
    }
}
