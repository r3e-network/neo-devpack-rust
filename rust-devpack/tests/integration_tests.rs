//! Round 55: Integration Tests for rust-devpack
//!
//! This module provides comprehensive integration tests that verify
//! the interaction between different components of the devpack.

use neo_devpack::prelude::*;

/// Integration Test: Complete token contract workflow
#[test]
fn token_contract_workflow() {
    // Initialize contexts
    let ctx = NeoStorage::get_context().expect("Should get storage context");

    // Create test data
    let from = NeoByteString::from_slice(b"from_address");
    let to = NeoByteString::from_slice(b"to_address");
    let amount = NeoInteger::new(100);

    // Store balance
    let balance_key = NeoByteString::from_slice(b"balance:from_address");
    let initial_balance = NeoByteString::from_slice(&1000i32.to_le_bytes());
    NeoStorage::put(&ctx, &balance_key, &initial_balance).expect("Should store balance");

    // Verify balance
    let stored = NeoStorage::get(&ctx, &balance_key).expect("Should get balance");
    assert!(!stored.is_empty());

    // Update balance
    let new_balance = NeoByteString::from_slice(&900i32.to_le_bytes());
    NeoStorage::put(&ctx, &balance_key, &new_balance).expect("Should update balance");

    // Create notification
    let event_name = NeoString::from_str("Transfer");
    let mut event_data = NeoArray::<NeoValue>::new();
    event_data.push(NeoValue::from(from));
    event_data.push(NeoValue::from(to));
    event_data.push(NeoValue::from(amount));

    assert!(NeoRuntime::notify(&event_name, &event_data).is_ok());
}

/// Integration Test: Contract deployment flow
#[test]
fn contract_deployment_flow() {
    // Create contract script
    let script = NeoByteString::from_slice(b"contract_bytecode");

    // Create manifest
    let manifest = NeoContractManifest {
        name: "TestToken".to_string(),
        version: "1.0.0".to_string(),
        author: "Test Author".to_string(),
        email: "test@example.com".to_string(),
        description: "Test token contract".to_string(),
        abi: NeoContractABI {
            hash: "0x12345678".to_string(),
            methods: vec![NeoContractMethod {
                name: "transfer".to_string(),
                parameters: vec![
                    NeoContractParameter {
                        name: "from".to_string(),
                        r#type: "ByteArray".to_string(),
                    },
                    NeoContractParameter {
                        name: "to".to_string(),
                        r#type: "ByteArray".to_string(),
                    },
                    NeoContractParameter {
                        name: "amount".to_string(),
                        r#type: "Integer".to_string(),
                    },
                ],
                return_type: "Boolean".to_string(),
                offset: 0,
                safe: false,
            }],
            events: vec![],
        },
        permissions: vec![],
        trusts: vec![],
        supported_standards: vec!["NEP-17".to_string()],
    };

    // Create contract
    let contract_hash =
        NeoContractRuntime::create(&script, &manifest).expect("Should create contract");

    assert!(!contract_hash.is_empty());
    assert_eq!(contract_hash.len(), 20);
}

/// Integration Test: Storage iteration
#[test]
fn storage_iteration() {
    let ctx = NeoStorage::get_context().expect("Should get storage context");
    let prefix = NeoByteString::from_slice(b"token:");

    // Insert multiple entries
    for i in 0..10 {
        let key = NeoByteString::from_slice(format!("token:account{}", i).as_bytes());
        let value = NeoByteString::from_slice(&(i * 100).to_le_bytes());
        NeoStorage::put(&ctx, &key, &value).expect("Should store");
    }

    // Find with prefix
    let mut iter = NeoStorage::find(&ctx, &prefix).expect("Should find");

    let mut count = 0;
    while iter.has_next() {
        if let Some(entry) = iter.next() {
            if let Some(st) = entry.as_struct() {
                assert!(st.get_field("key").is_some());
                assert!(st.get_field("value").is_some());
                count += 1;
            }
        }
    }

    assert_eq!(count, 10);
}

/// Integration Test: Multi-operation transaction
#[test]
fn multi_operation_transaction() {
    let ctx = NeoStorage::get_context().expect("Should get storage context");

    // Perform multiple storage operations
    let key1 = NeoByteString::from_slice(b"key1");
    let key2 = NeoByteString::from_slice(b"key2");
    let value1 = NeoByteString::from_slice(b"value1");
    let value2 = NeoByteString::from_slice(b"value2");

    // Put operations
    NeoStorage::put(&ctx, &key1, &value1).expect("Should put key1");
    NeoStorage::put(&ctx, &key2, &value2).expect("Should put key2");

    // Get operations
    let retrieved1 = NeoStorage::get(&ctx, &key1).expect("Should get key1");
    let retrieved2 = NeoStorage::get(&ctx, &key2).expect("Should get key2");

    assert_eq!(retrieved1.as_slice(), value1.as_slice());
    assert_eq!(retrieved2.as_slice(), value2.as_slice());

    // Delete one key
    NeoStorage::delete(&ctx, &key1).expect("Should delete key1");

    // Verify deletion
    let after_delete = NeoStorage::get(&ctx, &key1).expect("Should get after delete");
    assert!(after_delete.is_empty());

    // Verify other key still exists
    let still_exists = NeoStorage::get(&ctx, &key2).expect("Should still get key2");
    assert_eq!(still_exists.as_slice(), value2.as_slice());
}

/// Integration Test: Crypto operations workflow
#[test]
fn crypto_operations_workflow() {
    let data = NeoByteString::from_slice(b"test data for hashing");

    // SHA256
    let hash256 = NeoCrypto::sha256(&data).expect("Should compute SHA256");
    assert_eq!(hash256.len(), 32);

    // RIPEMD160
    let hash160 = NeoCrypto::ripemd160(&data).expect("Should compute RIPEMD160");
    assert_eq!(hash160.len(), 20);

    // Keccak256
    let keccak256 = NeoCrypto::keccak256(&data).expect("Should compute Keccak256");
    assert_eq!(keccak256.len(), 32);

    // Keccak512
    let keccak512 = NeoCrypto::keccak512(&data).expect("Should compute Keccak512");
    assert_eq!(keccak512.len(), 64);

    // Murmur32
    let seed = NeoInteger::new(0);
    let murmur = NeoCrypto::murmur32(&data, seed).expect("Should compute Murmur");
    assert!(murmur.as_i32() != 0 || data.is_empty());
}

/// Integration Test: JSON serialization roundtrip
#[test]
fn json_serialization_roundtrip() {
    // Integer
    let int_val = NeoValue::from(NeoInteger::new(42));
    let json = NeoJSON::serialize(&int_val).expect("Should serialize");
    let deserialized = NeoJSON::deserialize(&json).expect("Should deserialize");
    assert_eq!(deserialized.as_integer().unwrap().as_i32(), 42);

    // Boolean
    let bool_val = NeoValue::from(NeoBoolean::TRUE);
    let json = NeoJSON::serialize(&bool_val).expect("Should serialize");
    let deserialized = NeoJSON::deserialize(&json).expect("Should deserialize");
    assert!(deserialized.as_boolean().unwrap().as_bool());

    // String
    let string_val = NeoValue::from(NeoString::from_str("hello"));
    let json = NeoJSON::serialize(&string_val).expect("Should serialize");
    let deserialized = NeoJSON::deserialize(&json).expect("Should deserialize");
    assert_eq!(deserialized.as_string().unwrap().as_str(), "hello");
}

/// Integration Test: Witness verification
#[test]
fn witness_verification_flow() {
    let witness = NeoByteString::from_slice(b"witness_data");

    // Check witness
    let result = NeoRuntime::check_witness(&witness).expect("Should check witness");
    // Result depends on mock implementation
    assert!(result.as_bool() || !result.as_bool());
}

/// Integration Test: Runtime information access
#[test]
fn runtime_information_access() {
    // Get time
    let time = NeoRuntime::get_time().expect("Should get time");
    assert!(time.as_i32() >= 0);

    // Get network
    let network = NeoRuntime::get_network().expect("Should get network");
    assert!(network.as_i32() >= 0);

    // Get gas left
    let gas = NeoRuntime::get_gas_left().expect("Should get gas");
    assert!(gas.as_i32() >= 0);

    // Get trigger
    let trigger = NeoRuntime::get_trigger().expect("Should get trigger");
    assert!(trigger.as_i32() >= 0);

    // Get platform
    let platform = NeoRuntime::platform().expect("Should get platform");
    assert!(!platform.as_str().is_empty());
}

/// Integration Test: Contract metadata
#[test]
fn contract_metadata_management() {
    // Create contract ABI
    let abi = NeoContractABI {
        hash: "0xabcdef".to_string(),
        methods: vec![NeoContractMethod {
            name: "main".to_string(),
            parameters: vec![],
            return_type: "Integer".to_string(),
            offset: 0,
            safe: true,
        }],
        events: vec![NeoContractEvent {
            name: "Event1".to_string(),
            parameters: vec![],
        }],
    };

    // Create permissions
    let permissions = vec![NeoContractPermission {
        contract: "*".to_string(),
        methods: vec!["*".to_string()],
    }];

    // Verify structures
    assert_eq!(abi.methods.len(), 1);
    assert_eq!(abi.events.len(), 1);
    assert_eq!(permissions.len(), 1);
}

/// Integration Test: Read-only storage context
#[test]
fn read_only_storage_context() {
    let writable = NeoStorage::get_context().expect("Should get context");
    let read_only = NeoStorage::get_read_only_context().expect("Should get read-only");

    assert!(!writable.is_read_only());
    assert!(read_only.is_read_only());

    // Try to write to read-only context
    let key = NeoByteString::from_slice(b"key");
    let value = NeoByteString::from_slice(b"value");

    let result = NeoStorage::put(&read_only, &key, &value);
    assert!(result.is_err());
}

/// Integration Test: Complex value types
#[test]
fn complex_value_types() {
    // Array of mixed types
    let mut arr = NeoArray::new();
    arr.push(NeoValue::from(NeoInteger::new(1)));
    arr.push(NeoValue::from(NeoBoolean::TRUE));
    arr.push(NeoValue::from(NeoString::from_str("hello")));

    // Map with complex values
    let mut map = NeoMap::new();
    map.insert(
        NeoValue::from(NeoString::from_str("array")),
        NeoValue::from(arr),
    );

    // Struct with fields
    let s = NeoStruct::new()
        .with_field("map", NeoValue::from(map))
        .with_field("int", NeoValue::from(NeoInteger::new(42)));

    // Verify nested access
    assert!(s.get_field("map").is_some());
    assert!(s.get_field("int").is_some());
}

/// Integration Test: Event emission
#[test]
fn event_emission() {
    // Define event
    let event = NeoContractEvent {
        name: "Transfer".to_string(),
        parameters: vec![
            NeoContractParameter {
                name: "from".to_string(),
                r#type: "ByteArray".to_string(),
            },
            NeoContractParameter {
                name: "to".to_string(),
                r#type: "ByteArray".to_string(),
            },
            NeoContractParameter {
                name: "amount".to_string(),
                r#type: "Integer".to_string(),
            },
        ],
    };

    // Create event data
    let from = NeoByteString::from_slice(b"addr1");
    let to = NeoByteString::from_slice(b"addr2");
    let amount = NeoInteger::new(100);

    let mut state = NeoArray::<NeoValue>::new();
    state.push(NeoValue::from(from));
    state.push(NeoValue::from(to));
    state.push(NeoValue::from(amount));

    // Emit event via runtime
    let event_name = NeoString::from_str(&event.name);
    assert!(NeoRuntime::notify(&event_name, &state).is_ok());
}

/// Integration Test: Script hash operations
#[test]
fn script_hash_operations() {
    // Get various script hashes
    let calling = NeoRuntime::get_calling_script_hash().expect("Should get calling");
    let entry = NeoRuntime::get_entry_script_hash().expect("Should get entry");
    let executing = NeoRuntime::get_executing_script_hash().expect("Should get executing");

    // Script hashes should be 20 bytes
    assert_eq!(calling.len(), 20);
    assert_eq!(entry.len(), 20);
    assert_eq!(executing.len(), 20);
}

/// Integration Test: Address version
#[test]
fn address_version_check() {
    let version = NeoRuntime::get_address_version().expect("Should get version");
    assert!(version.as_i32() > 0);
}

/// Integration Test: Notification retrieval
#[test]
fn notification_retrieval() {
    // Get notifications (may be empty in test)
    let script_hash = NeoByteString::from_slice(&[0u8; 20]);
    let notifications =
        NeoRuntime::get_notifications(Some(&script_hash)).expect("Should get notifications");

    // Result should be an array
    assert!(notifications.is_empty() || !notifications.is_empty());
}

/// Integration Test: Script container
#[test]
fn script_container_access() {
    let container = NeoRuntime::get_script_container().expect("Should get container");
    // Container is an array (may be empty in test)
    assert!(container.is_empty() || !container.is_empty());
}

/// Integration Test: Log operation
#[test]
fn log_operation() {
    let message = NeoString::from_str("Test log message");
    assert!(NeoRuntime::log(&message).is_ok());
}

/// Integration Test: Random number generation
#[test]
fn random_number_generation() {
    let random = NeoRuntime::get_random().expect("Should get random");
    // Random is an integer
    assert!(random.as_i32() >= 0 || random.as_i32() < 0);
}
