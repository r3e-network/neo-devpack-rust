// Comprehensive tests for Neo N3 runtime

use neo_devpack::prelude::*;
use neo_runtime::*;

#[test]
fn test_neo_runtime_context() {
    let context = NeoRuntimeContext::new();
    
    // Test context properties
    assert_eq!(context.trigger.as_i32(), 0);
    assert_eq!(context.gas_left.as_i32(), 0);
    assert_eq!(context.invocation_counter.as_i32(), 0);
    assert!(context.calling_script_hash.is_empty());
    assert!(context.entry_script_hash.is_empty());
    assert!(context.executing_script_hash.is_empty());
}

#[test]
fn test_neo_storage_context() {
    let context = NeoStorageContext::new(42, false);
    assert_eq!(context.id(), 42);
    assert!(!context.is_read_only());
    
    let read_only_context = NeoStorageContext::new(1, true);
    assert_eq!(read_only_context.id(), 1);
    assert!(read_only_context.is_read_only());
}

#[test]
fn test_neo_storage_operations() {
    let context = NeoStorageContext::new(1, false);
    let read_only_context = NeoStorageContext::new(2, true);
    
    // Test get operation
    let key = NeoByteString::from_slice(b"test_key");
    let result = NeoStorage::get(&context, &key);
    assert!(result.is_ok());
    assert!(!result.unwrap().is_empty());
    
    // Test put operation
    let value = NeoByteString::from_slice(b"test_value");
    let put_result = NeoStorage::put(&context, &key, &value);
    assert!(put_result.is_ok());
    
    // Test put on read-only context
    let read_only_put_result = NeoStorage::put(&read_only_context, &key, &value);
    assert!(read_only_put_result.is_err());
    assert_eq!(read_only_put_result.unwrap_err(), NeoError::InvalidOperation);
    
    // Test delete operation
    let delete_result = NeoStorage::delete(&context, &key);
    assert!(delete_result.is_ok());
    
    // Test delete on read-only context
    let read_only_delete_result = NeoStorage::delete(&read_only_context, &key);
    assert!(read_only_delete_result.is_err());
    assert_eq!(read_only_delete_result.unwrap_err(), NeoError::InvalidOperation);
    
    // Test find operation
    let prefix = NeoByteString::from_slice(b"test_");
    let find_result = NeoStorage::find(&context, &prefix);
    assert!(find_result.is_ok());
    let iterator = find_result.unwrap();
    assert!(iterator.has_next());
}

#[test]
fn test_neo_contract_operations() {
    let script = NeoByteString::from_slice(b"test_script");
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
    
    // Test create contract
    let create_result = NeoContractStruct::create(&script, &manifest);
    assert!(create_result.is_ok());
    let contract_hash = create_result.unwrap();
    assert!(!contract_hash.is_empty());
    
    // Test update contract
    let script_hash = NeoByteString::from_slice(b"test_script_hash");
    let update_result = NeoContractStruct::update(&script_hash, &script, &manifest);
    assert!(update_result.is_ok());
    
    // Test destroy contract
    let destroy_result = NeoContractStruct::destroy(&script_hash);
    assert!(destroy_result.is_ok());
    
    // Test call contract
    let method = NeoString::from_str("test_method");
    let args = NeoArray::new();
    let call_result = NeoContractStruct::call(&script_hash, &method, &args);
    assert!(call_result.is_ok());
}

#[test]
fn test_neo_crypto_operations() {
    let data = NeoByteString::from_slice(b"test_data");
    
    // Test SHA256
    let sha256_result = NeoCrypto::sha256(&data);
    assert!(sha256_result.is_ok());
    let sha256_hash = sha256_result.unwrap();
    assert_eq!(sha256_hash.len(), 32); // SHA256 produces 32 bytes
    
    // Test RIPEMD160
    let ripemd160_result = NeoCrypto::ripemd160(&data);
    assert!(ripemd160_result.is_ok());
    let ripemd160_hash = ripemd160_result.unwrap();
    assert_eq!(ripemd160_hash.len(), 20); // RIPEMD160 produces 20 bytes
    
    // Test Keccak256
    let keccak256_result = NeoCrypto::keccak256(&data);
    assert!(keccak256_result.is_ok());
    let keccak256_hash = keccak256_result.unwrap();
    assert_eq!(keccak256_hash.len(), 32); // Keccak256 produces 32 bytes
    
    // Test Keccak512
    let keccak512_result = NeoCrypto::keccak512(&data);
    assert!(keccak512_result.is_ok());
    let keccak512_hash = keccak512_result.unwrap();
    assert_eq!(keccak512_hash.len(), 64); // Keccak512 produces 64 bytes
    
    // Test Murmur32
    let seed = NeoInteger::new(12345);
    let murmur32_result = NeoCrypto::murmur32(&data, seed);
    assert!(murmur32_result.is_ok());
    let murmur32_hash = murmur32_result.unwrap();
    assert!(murmur32_hash.as_i32() != 0);
    
    // Test signature verification
    let message = NeoByteString::from_slice(b"test_message");
    let signature = NeoByteString::from_slice(&[0x42; 64]); // 64-byte signature
    let public_key = NeoByteString::from_slice(&[0x02; 33]); // 33-byte public key
    
    let verify_result = NeoCrypto::verify_signature(&message, &signature, &public_key);
    assert!(verify_result.is_ok());
    assert!(verify_result.unwrap().as_bool());
    
    // Test signature verification with invalid lengths
    let invalid_signature = NeoByteString::from_slice(&[0x42; 32]); // 32-byte signature (invalid)
    let invalid_verify_result = NeoCrypto::verify_signature(&message, &invalid_signature, &public_key);
    assert!(invalid_verify_result.is_ok());
    assert!(!invalid_verify_result.unwrap().as_bool());
    
    // Test signature verification with recovery
    let recovery_result = NeoCrypto::verify_signature_with_recovery(&message, &signature);
    assert!(recovery_result.is_ok());
    let recovered_public_key = recovery_result.unwrap();
    assert_eq!(recovered_public_key.len(), 33); // 33-byte public key
}

#[test]
fn test_neo_json_operations() {
    // Test serialization
    let int_value = NeoValue::from(NeoInteger::new(42));
    let int_json = NeoJSON::serialize(&int_value);
    assert!(int_json.is_ok());
    let int_json_string = int_json.unwrap();
    assert!(int_json_string.as_str().contains("integer"));
    assert!(int_json_string.as_str().contains("42"));
    
    let bool_value = NeoValue::from(NeoBoolean::TRUE);
    let bool_json = NeoJSON::serialize(&bool_value);
    assert!(bool_json.is_ok());
    let bool_json_string = bool_json.unwrap();
    assert!(bool_json_string.as_str().contains("boolean"));
    assert!(bool_json_string.as_str().contains("true"));
    
    let string_value = NeoValue::from(NeoString::from_str("hello"));
    let string_json = NeoJSON::serialize(&string_value);
    assert!(string_json.is_ok());
    let string_json_string = string_json.unwrap();
    assert!(string_json_string.as_str().contains("string"));
    assert!(string_json_string.as_str().contains("hello"));
    
    let bs_value = NeoValue::from(NeoByteString::from_slice(b"data"));
    let bs_json = NeoJSON::serialize(&bs_value);
    assert!(bs_json.is_ok());
    let bs_json_string = bs_json.unwrap();
    assert!(bs_json_string.as_str().contains("bytestring"));
    
    let null_value = NeoValue::Null;
    let null_json = NeoJSON::serialize(&null_value);
    assert!(null_json.is_ok());
    let null_json_string = null_json.unwrap();
    assert!(null_json_string.as_str().contains("null"));
    
    // Test deserialization
    let integer_json = NeoString::from_str("{\"type\":\"integer\",\"value\":42}");
    let integer_result = NeoJSON::deserialize(&integer_json);
    assert!(integer_result.is_ok());
    let integer_value = integer_result.unwrap();
    assert!(integer_value.as_integer().is_some());
    assert_eq!(integer_value.as_integer().unwrap().as_i32(), 42);
    
    let boolean_json = NeoString::from_str("{\"type\":\"boolean\",\"value\":true}");
    let boolean_result = NeoJSON::deserialize(&boolean_json);
    assert!(boolean_result.is_ok());
    let boolean_value = boolean_result.unwrap();
    assert!(boolean_value.as_boolean().is_some());
    assert!(boolean_value.as_boolean().unwrap().as_bool());
    
    let string_json = NeoString::from_str("{\"type\":\"string\",\"value\":\"hello\"}");
    let string_result = NeoJSON::deserialize(&string_json);
    assert!(string_result.is_ok());
    let string_value = string_result.unwrap();
    assert!(string_value.as_string().is_some());
    assert_eq!(string_value.as_string().unwrap().as_str(), "parsed");
    
    let null_json = NeoString::from_str("{\"type\":\"null\"}");
    let null_result = NeoJSON::deserialize(&null_json);
    assert!(null_result.is_ok());
    let null_value = null_result.unwrap();
    assert!(null_value.is_null());
}

#[test]
fn test_neo_iterator_operations() {
    let data = vec![
        NeoValue::from(NeoInteger::new(1)),
        NeoValue::from(NeoInteger::new(2)),
        NeoValue::from(NeoInteger::new(3)),
    ];
    
    // Test create from array
    let array = NeoArray::new();
    for value in &data {
        array.push(value.clone());
    }
    let iterator = NeoIteratorFactory::create_from_array(&array);
    assert!(iterator.has_next());
    
    // Test create from map
    let mut map = NeoMap::new();
    map.insert(NeoValue::from(NeoString::from_str("key1")), NeoValue::from(NeoInteger::new(1)));
    map.insert(NeoValue::from(NeoString::from_str("key2")), NeoValue::from(NeoInteger::new(2)));
    let map_iterator = NeoIteratorFactory::create_from_map(&map);
    assert!(map_iterator.has_next());
    
    // Test create from storage
    let context = NeoStorageContext::new(1);
    let prefix = NeoByteString::from_slice(b"test_");
    let storage_iterator = NeoIteratorFactory::create_from_storage(&context, &prefix);
    assert!(storage_iterator.is_ok());
    let iterator = storage_iterator.unwrap();
    assert!(iterator.has_next());
}

#[test]
fn test_neo_runtime_operations() {
    // Test get time
    let time_result = NeoRuntime::get_time();
    assert!(time_result.is_ok());
    let time = time_result.unwrap();
    assert!(time.as_i32() > 0);
    
    // Test check witness
    let account = NeoByteString::from_slice(b"test_account");
    let witness_result = NeoRuntime::check_witness(&account);
    assert!(witness_result.is_ok());
    assert!(witness_result.unwrap().as_bool());
    
    // Test notify
    let event = NeoString::from_str("TestEvent");
    let state = NeoArray::new();
    let notify_result = NeoRuntime::notify(&event, &state);
    assert!(notify_result.is_ok());
    
    // Test log
    let message = NeoString::from_str("Test message");
    let log_result = NeoRuntime::log(&message);
    assert!(log_result.is_ok());
    
    // Test get platform
    let platform_result = NeoRuntime::get_platform();
    assert!(platform_result.is_ok());
    let platform = platform_result.unwrap();
    assert_eq!(platform.as_str(), "Neo N3");
    
    // Test get trigger
    let trigger_result = NeoRuntime::get_trigger();
    assert!(trigger_result.is_ok());
    let trigger = trigger_result.unwrap();
    assert_eq!(trigger.as_i32(), 0);
    
    // Test get invocation counter
    let counter_result = NeoRuntime::get_invocation_counter();
    assert!(counter_result.is_ok());
    let counter = counter_result.unwrap();
    assert_eq!(counter.as_i32(), 1);
    
    // Test get random
    let random_result = NeoRuntime::get_random();
    assert!(random_result.is_ok());
    let random = random_result.unwrap();
    assert_eq!(random.as_i32(), 12345);
    
    // Test get network
    let network_result = NeoRuntime::get_network();
    assert!(network_result.is_ok());
    let network = network_result.unwrap();
    assert_eq!(network.as_i32(), 860833102);
    
    // Test get address version
    let address_version_result = NeoRuntime::get_address_version();
    assert!(address_version_result.is_ok());
    let address_version = address_version_result.unwrap();
    assert_eq!(address_version.as_i32(), 53);
    
    // Test get calling script hash
    let calling_hash_result = NeoRuntime::get_calling_script_hash();
    assert!(calling_hash_result.is_ok());
    let calling_hash = calling_hash_result.unwrap();
    assert!(!calling_hash.is_empty());
    
    // Test get entry script hash
    let entry_hash_result = NeoRuntime::get_entry_script_hash();
    assert!(entry_hash_result.is_ok());
    let entry_hash = entry_hash_result.unwrap();
    assert!(!entry_hash.is_empty());
    
    // Test get executing script hash
    let executing_hash_result = NeoRuntime::get_executing_script_hash();
    assert!(executing_hash_result.is_ok());
    let executing_hash = executing_hash_result.unwrap();
    assert!(!executing_hash.is_empty());
    
    // Test get script container
    let container_result = NeoRuntime::get_script_container();
    assert!(container_result.is_ok());
    let container = container_result.unwrap();
    assert!(!container.is_empty());
    
    // Test get transaction
    let transaction_result = NeoRuntime::get_transaction();
    assert!(transaction_result.is_ok());
    let transaction = transaction_result.unwrap();
    assert!(!transaction.is_empty());
    
    // Test get block
    let block_result = NeoRuntime::get_block();
    assert!(block_result.is_ok());
    let block = block_result.unwrap();
    assert!(!block.is_empty());
    
    // Test get block height
    let height_result = NeoRuntime::get_block_height();
    assert!(height_result.is_ok());
    let height = height_result.unwrap();
    assert!(height.as_i32() >= 0);
    
    // Test get block hash
    let block_height = NeoInteger::new(100);
    let hash_result = NeoRuntime::get_block_hash(block_height);
    assert!(hash_result.is_ok());
    let hash = hash_result.unwrap();
    assert!(!hash.is_empty());
    
    // Test get block header
    let header_result = NeoRuntime::get_block_header(block_height);
    assert!(header_result.is_ok());
    let header = header_result.unwrap();
    assert!(!header.is_empty());
    
    // Test get transaction height
    let tx_hash = NeoByteString::from_slice(b"test_transaction_hash");
    let tx_height_result = NeoRuntime::get_transaction_height(&tx_hash);
    assert!(tx_height_result.is_ok());
    let tx_height = tx_height_result.unwrap();
    assert!(tx_height.as_i32() >= 0);
    
    // Test get transaction from block
    let tx_index = NeoInteger::new(0);
    let tx_from_block_result = NeoRuntime::get_transaction_from_block(block_height, tx_index);
    assert!(tx_from_block_result.is_ok());
    let tx_from_block = tx_from_block_result.unwrap();
    assert!(!tx_from_block.is_empty());
    
    // Test get account
    let account_hash = NeoByteString::from_slice(b"test_account_hash");
    let account_result = NeoRuntime::get_account(&account_hash);
    assert!(account_result.is_ok());
    let account = account_result.unwrap();
    assert!(!account.is_empty());
    
    // Test get validators
    let validators_result = NeoRuntime::get_validators();
    assert!(validators_result.is_ok());
    let validators = validators_result.unwrap();
    assert!(!validators.is_empty());
    
    // Test get committee
    let committee_result = NeoRuntime::get_committee();
    assert!(committee_result.is_ok());
    let committee = committee_result.unwrap();
    assert!(!committee.is_empty());
    
    // Test get next block validators
    let next_validators_result = NeoRuntime::get_next_block_validators();
    assert!(next_validators_result.is_ok());
    let next_validators = next_validators_result.unwrap();
    assert!(!next_validators.is_empty());
    
    // Test get candidates
    let candidates_result = NeoRuntime::get_candidates();
    assert!(candidates_result.is_ok());
    let candidates = candidates_result.unwrap();
    assert!(!candidates.is_empty());
    
    // Test get gas left
    let gas_left_result = NeoRuntime::get_gas_left();
    assert!(gas_left_result.is_ok());
    let gas_left = gas_left_result.unwrap();
    assert!(gas_left.as_i32() >= 0);
    
    // Test get invocation gas
    let invocation_gas_result = NeoRuntime::get_invocation_gas();
    assert!(invocation_gas_result.is_ok());
    let invocation_gas = invocation_gas_result.unwrap();
    assert!(invocation_gas.as_i32() >= 0);
    
    // Test get notifications
    let script_hash = NeoByteString::from_slice(b"test_script_hash");
    let notifications_result = NeoRuntime::get_notifications(&script_hash);
    assert!(notifications_result.is_ok());
    let notifications = notifications_result.unwrap();
    assert!(!notifications.is_empty());
    
    // Test get all notifications
    let all_notifications_result = NeoRuntime::get_all_notifications();
    assert!(all_notifications_result.is_ok());
    let all_notifications = all_notifications_result.unwrap();
    assert!(!all_notifications.is_empty());
    
    // Test get storage context
    let storage_context_result = NeoRuntime::get_storage_context();
    assert!(storage_context_result.is_ok());
    let storage_context = storage_context_result.unwrap();
    assert_eq!(storage_context.id(), 0);
    
    // Test get read-only context
    let read_only_context_result = NeoRuntime::get_read_only_context();
    assert!(read_only_context_result.is_ok());
    let read_only_context = read_only_context_result.unwrap();
    assert_eq!(read_only_context.id(), 0);
    
    // Test get calling context
    let calling_context_result = NeoRuntime::get_calling_context();
    assert!(calling_context_result.is_ok());
    let calling_context = calling_context_result.unwrap();
    assert_eq!(calling_context.id(), 0);
    
    // Test get entry context
    let entry_context_result = NeoRuntime::get_entry_context();
    assert!(entry_context_result.is_ok());
    let entry_context = entry_context_result.unwrap();
    assert_eq!(entry_context.id(), 0);
    
    // Test get executing context
    let executing_context_result = NeoRuntime::get_executing_context();
    assert!(executing_context_result.is_ok());
    let executing_context = executing_context_result.unwrap();
    assert_eq!(executing_context.id(), 0);
}
