// Comprehensive tests for Neo N3 syscalls

use neo_devpack::prelude::*;
use neo_syscalls::*;

#[test]
fn test_syscall_registry() {
    let registry = NeoVMSyscallRegistry::getInstance();
    
    // Test syscall lookup
    let get_time = registry.getSyscall("System.Runtime.GetTime");
    assert!(get_time.is_some());
    let syscall = get_time.unwrap();
    assert_eq!(syscall.name, "System.Runtime.GetTime");
    assert_eq!(syscall.hash, 0x68b4c4c1);
    assert_eq!(syscall.return_type, "Integer");
    assert_eq!(syscall.gas_cost, 1);
    
    // Test syscall by hash
    let by_hash = registry.getSyscallByHash(0x68b4c4c1);
    assert!(by_hash.is_some());
    assert_eq!(by_hash.unwrap().name, "System.Runtime.GetTime");
    
    // Test non-existent syscall
    let non_existent = registry.getSyscall("NonExistentSyscall");
    assert!(non_existent.is_none());
    
    // Test non-existent hash
    let non_existent_hash = registry.getSyscallByHash(0x12345678);
    assert!(non_existent_hash.is_none());
}

#[test]
fn test_syscall_function() {
    // Test System.Runtime.GetTime
    let get_time_result = neovm_syscall(0x68b4c4c1, &[]);
    assert!(get_time_result.is_ok());
    let time_value = get_time_result.unwrap();
    assert!(time_value.as_integer().is_some());
    assert_eq!(time_value.as_integer().unwrap().as_i32(), 1640995200);
    
    // Test System.Runtime.CheckWitness
    let account = NeoByteString::from_slice(b"test_account");
    let check_witness_result = neovm_syscall(0x0b5b4b1a, &[NeoValue::from(account)]);
    assert!(check_witness_result.is_ok());
    let witness_value = check_witness_result.unwrap();
    assert!(witness_value.as_boolean().is_some());
    assert!(witness_value.as_boolean().unwrap().as_bool());
    
    // Test System.Runtime.Notify
    let event = NeoString::from_str("TestEvent");
    let state = NeoArray::new();
    let notify_result = neovm_syscall(0x0f4b4b1a, &[NeoValue::from(event), NeoValue::from(state)]);
    assert!(notify_result.is_ok());
    assert!(notify_result.unwrap().is_null());
    
    // Test System.Runtime.Log
    let message = NeoString::from_str("Test message");
    let log_result = neovm_syscall(0x0f4b4b1b, &[NeoValue::from(message)]);
    assert!(log_result.is_ok());
    assert!(log_result.unwrap().is_null());
    
    // Test System.Runtime.GetPlatform
    let platform_result = neovm_syscall(0x0f4b4b1c, &[]);
    assert!(platform_result.is_ok());
    let platform_value = platform_result.unwrap();
    assert!(platform_value.as_string().is_some());
    assert_eq!(platform_value.as_string().unwrap().as_str(), "Neo N3");
    
    // Test System.Runtime.GetTrigger
    let trigger_result = neovm_syscall(0x0f4b4b1d, &[]);
    assert!(trigger_result.is_ok());
    let trigger_value = trigger_result.unwrap();
    assert!(trigger_value.as_integer().is_some());
    assert_eq!(trigger_value.as_integer().unwrap().as_i32(), 0);
    
    // Test System.Runtime.GetInvocationCounter
    let counter_result = neovm_syscall(0x0f4b4b1e, &[]);
    assert!(counter_result.is_ok());
    let counter_value = counter_result.unwrap();
    assert!(counter_value.as_integer().is_some());
    assert_eq!(counter_value.as_integer().unwrap().as_i32(), 1);
    
    // Test System.Runtime.GetRandom
    let random_result = neovm_syscall(0x0f4b4b1f, &[]);
    assert!(random_result.is_ok());
    let random_value = random_result.unwrap();
    assert!(random_value.as_integer().is_some());
    assert_eq!(random_value.as_integer().unwrap().as_i32(), 12345);
    
    // Test System.Runtime.GetNetwork
    let network_result = neovm_syscall(0x0f4b4b20, &[]);
    assert!(network_result.is_ok());
    let network_value = network_result.unwrap();
    assert!(network_value.as_integer().is_some());
    assert_eq!(network_value.as_integer().unwrap().as_i32(), 860833102);
    
    // Test System.Runtime.GetAddressVersion
    let address_version_result = neovm_syscall(0x0f4b4b21, &[]);
    assert!(address_version_result.is_ok());
    let address_version_value = address_version_result.unwrap();
    assert!(address_version_value.as_integer().is_some());
    assert_eq!(address_version_value.as_integer().unwrap().as_i32(), 53);
    
    // Test unknown syscall
    let unknown_result = neovm_syscall(0x12345678, &[]);
    assert!(unknown_result.is_ok());
    assert!(unknown_result.unwrap().is_null());
}

#[test]
fn test_syscall_wrapper() {
    // Test get time
    let time_result = NeoVMSyscall::get_time();
    assert!(time_result.is_ok());
    let time = time_result.unwrap();
    assert_eq!(time.as_i32(), 1640995200);
    
    // Test check witness
    let account = NeoByteString::from_slice(b"test_account");
    let witness_result = NeoVMSyscall::check_witness(&account);
    assert!(witness_result.is_ok());
    assert!(witness_result.unwrap().as_bool());
    
    // Test notify
    let event = NeoString::from_str("TestEvent");
    let state = NeoArray::new();
    let notify_result = NeoVMSyscall::notify(&event, &state);
    assert!(notify_result.is_ok());
    
    // Test log
    let message = NeoString::from_str("Test message");
    let log_result = NeoVMSyscall::log(&message);
    assert!(log_result.is_ok());
    
    // Test get platform
    let platform_result = NeoVMSyscall::get_platform();
    assert!(platform_result.is_ok());
    let platform = platform_result.unwrap();
    assert_eq!(platform.as_str(), "Neo N3");
    
    // Test get trigger
    let trigger_result = NeoVMSyscall::get_trigger();
    assert!(trigger_result.is_ok());
    let trigger = trigger_result.unwrap();
    assert_eq!(trigger.as_i32(), 0);
    
    // Test get invocation counter
    let counter_result = NeoVMSyscall::get_invocation_counter();
    assert!(counter_result.is_ok());
    let counter = counter_result.unwrap();
    assert_eq!(counter.as_i32(), 1);
    
    // Test get random
    let random_result = NeoVMSyscall::get_random();
    assert!(random_result.is_ok());
    let random = random_result.unwrap();
    assert_eq!(random.as_i32(), 12345);
    
    // Test get network
    let network_result = NeoVMSyscall::get_network();
    assert!(network_result.is_ok());
    let network = network_result.unwrap();
    assert_eq!(network.as_i32(), 860833102);
    
    // Test get address version
    let address_version_result = NeoVMSyscall::get_address_version();
    assert!(address_version_result.is_ok());
    let address_version = address_version_result.unwrap();
    assert_eq!(address_version.as_i32(), 53);
    
    // Test get calling script hash
    let calling_hash_result = NeoVMSyscall::get_calling_script_hash();
    assert!(calling_hash_result.is_ok());
    let calling_hash = calling_hash_result.unwrap();
    assert!(!calling_hash.is_empty());
    
    // Test get entry script hash
    let entry_hash_result = NeoVMSyscall::get_entry_script_hash();
    assert!(entry_hash_result.is_ok());
    let entry_hash = entry_hash_result.unwrap();
    assert!(!entry_hash.is_empty());
    
    // Test get executing script hash
    let executing_hash_result = NeoVMSyscall::get_executing_script_hash();
    assert!(executing_hash_result.is_ok());
    let executing_hash = executing_hash_result.unwrap();
    assert!(!executing_hash.is_empty());
    
    // Test get script container
    let container_result = NeoVMSyscall::get_script_container();
    assert!(container_result.is_ok());
    let container = container_result.unwrap();
    assert!(!container.is_empty());
    
    // Test get transaction
    let transaction_result = NeoVMSyscall::get_transaction();
    assert!(transaction_result.is_ok());
    let transaction = transaction_result.unwrap();
    assert!(!transaction.is_empty());
    
    // Test get block
    let block_result = NeoVMSyscall::get_block();
    assert!(block_result.is_ok());
    let block = block_result.unwrap();
    assert!(!block.is_empty());
    
    // Test get block height
    let height_result = NeoVMSyscall::get_block_height();
    assert!(height_result.is_ok());
    let height = height_result.unwrap();
    assert!(height.as_i32() >= 0);
    
    // Test get block hash
    let block_height = NeoInteger::new(100);
    let hash_result = NeoVMSyscall::get_block_hash(block_height);
    assert!(hash_result.is_ok());
    let hash = hash_result.unwrap();
    assert!(!hash.is_empty());
    
    // Test get block header
    let header_result = NeoVMSyscall::get_block_header(block_height);
    assert!(header_result.is_ok());
    let header = header_result.unwrap();
    assert!(!header.is_empty());
    
    // Test get transaction height
    let tx_hash = NeoByteString::from_slice(b"test_transaction_hash");
    let tx_height_result = NeoVMSyscall::get_transaction_height(&tx_hash);
    assert!(tx_height_result.is_ok());
    let tx_height = tx_height_result.unwrap();
    assert!(tx_height.as_i32() >= 0);
    
    // Test get transaction from block
    let tx_index = NeoInteger::new(0);
    let tx_from_block_result = NeoVMSyscall::get_transaction_from_block(block_height, tx_index);
    assert!(tx_from_block_result.is_ok());
    let tx_from_block = tx_from_block_result.unwrap();
    assert!(!tx_from_block.is_empty());
    
    // Test get account
    let account_hash = NeoByteString::from_slice(b"test_account_hash");
    let account_result = NeoVMSyscall::get_account(&account_hash);
    assert!(account_result.is_ok());
    let account = account_result.unwrap();
    assert!(!account.is_empty());
    
    // Test get validators
    let validators_result = NeoVMSyscall::get_validators();
    assert!(validators_result.is_ok());
    let validators = validators_result.unwrap();
    assert!(!validators.is_empty());
    
    // Test get committee
    let committee_result = NeoVMSyscall::get_committee();
    assert!(committee_result.is_ok());
    let committee = committee_result.unwrap();
    assert!(!committee.is_empty());
    
    // Test get next block validators
    let next_validators_result = NeoVMSyscall::get_next_block_validators();
    assert!(next_validators_result.is_ok());
    let next_validators = next_validators_result.unwrap();
    assert!(!next_validators.is_empty());
    
    // Test get candidates
    let candidates_result = NeoVMSyscall::get_candidates();
    assert!(candidates_result.is_ok());
    let candidates = candidates_result.unwrap();
    assert!(!candidates.is_empty());
    
    // Test get gas left
    let gas_left_result = NeoVMSyscall::get_gas_left();
    assert!(gas_left_result.is_ok());
    let gas_left = gas_left_result.unwrap();
    assert!(gas_left.as_i32() >= 0);
    
    // Test get invocation gas
    let invocation_gas_result = NeoVMSyscall::get_invocation_gas();
    assert!(invocation_gas_result.is_ok());
    let invocation_gas = invocation_gas_result.unwrap();
    assert!(invocation_gas.as_i32() >= 0);
    
    // Test get notifications
    let script_hash = NeoByteString::from_slice(b"test_script_hash");
    let notifications_result = NeoVMSyscall::get_notifications(&script_hash);
    assert!(notifications_result.is_ok());
    let notifications = notifications_result.unwrap();
    assert!(!notifications.is_empty());
    
    // Test get all notifications
    let all_notifications_result = NeoVMSyscall::get_all_notifications();
    assert!(all_notifications_result.is_ok());
    let all_notifications = all_notifications_result.unwrap();
    assert!(!all_notifications.is_empty());
    
    // Test get storage context
    let storage_context_result = NeoVMSyscall::get_storage_context();
    assert!(storage_context_result.is_ok());
    let storage_context = storage_context_result.unwrap();
    assert_eq!(storage_context.id(), 0);
    
    // Test get read-only context
    let read_only_context_result = NeoVMSyscall::get_read_only_context();
    assert!(read_only_context_result.is_ok());
    let read_only_context = read_only_context_result.unwrap();
    assert_eq!(read_only_context.id(), 0);
    
    // Test get calling context
    let calling_context_result = NeoVMSyscall::get_calling_context();
    assert!(calling_context_result.is_ok());
    let calling_context = calling_context_result.unwrap();
    assert_eq!(calling_context.id(), 0);
    
    // Test get entry context
    let entry_context_result = NeoVMSyscall::get_entry_context();
    assert!(entry_context_result.is_ok());
    let entry_context = entry_context_result.unwrap();
    assert_eq!(entry_context.id(), 0);
    
    // Test get executing context
    let executing_context_result = NeoVMSyscall::get_executing_context();
    assert!(executing_context_result.is_ok());
    let executing_context = executing_context_result.unwrap();
    assert_eq!(executing_context.id(), 0);
}

#[test]
fn test_syscall_registry_loading() {
    let registry = NeoVMSyscallRegistry::getInstance();
    
    // Test loading from file
    let load_result = registry.loadFromFile("neo_syscalls.json");
    assert!(load_result);
    
    // Test getting all syscall names
    let all_names = registry.getAllSyscallNames();
    assert!(!all_names.is_empty());
    assert!(all_names.contains(&"System.Runtime.GetTime".to_string()));
    assert!(all_names.contains(&"System.Runtime.CheckWitness".to_string()));
    assert!(all_names.contains(&"System.Runtime.Notify".to_string()));
}

#[test]
fn test_syscall_parameters() {
    let registry = NeoVMSyscallRegistry::getInstance();
    
    // Test syscall with parameters
    let check_witness = registry.getSyscall("System.Runtime.CheckWitness");
    assert!(check_witness.is_some());
    let syscall = check_witness.unwrap();
    assert_eq!(syscall.parameters.len(), 1);
    assert_eq!(syscall.parameters[0], "ByteString");
    
    // Test syscall without parameters
    let get_time = registry.getSyscall("System.Runtime.GetTime");
    assert!(get_time.is_some());
    let syscall = get_time.unwrap();
    assert_eq!(syscall.parameters.len(), 0);
    
    // Test syscall with multiple parameters
    let notify = registry.getSyscall("System.Runtime.Notify");
    assert!(notify.is_some());
    let syscall = notify.unwrap();
    assert_eq!(syscall.parameters.len(), 2);
    assert_eq!(syscall.parameters[0], "String");
    assert_eq!(syscall.parameters[1], "Array");
}

#[test]
fn test_syscall_gas_costs() {
    let registry = NeoVMSyscallRegistry::getInstance();
    
    // Test gas costs for different syscalls
    let get_time = registry.getSyscall("System.Runtime.GetTime");
    assert!(get_time.is_some());
    assert_eq!(get_time.unwrap().gas_cost, 1);
    
    let check_witness = registry.getSyscall("System.Runtime.CheckWitness");
    assert!(check_witness.is_some());
    assert_eq!(check_witness.unwrap().gas_cost, 200);
    
    let notify = registry.getSyscall("System.Runtime.Notify");
    assert!(notify.is_some());
    assert_eq!(notify.unwrap().gas_cost, 1);
}

#[test]
fn test_syscall_descriptions() {
    let registry = NeoVMSyscallRegistry::getInstance();
    
    // Test descriptions for different syscalls
    let get_time = registry.getSyscall("System.Runtime.GetTime");
    assert!(get_time.is_some());
    assert_eq!(get_time.unwrap().description, "Get current timestamp");
    
    let check_witness = registry.getSyscall("System.Runtime.CheckWitness");
    assert!(check_witness.is_some());
    assert_eq!(check_witness.unwrap().description, "Check if the specified account is a witness");
    
    let notify = registry.getSyscall("System.Runtime.Notify");
    assert!(notify.is_some());
    assert_eq!(notify.unwrap().description, "Send notification");
}

#[test]
fn test_syscall_return_types() {
    let registry = NeoVMSyscallRegistry::getInstance();
    
    // Test return types for different syscalls
    let get_time = registry.getSyscall("System.Runtime.GetTime");
    assert!(get_time.is_some());
    assert_eq!(get_time.unwrap().return_type, "Integer");
    
    let check_witness = registry.getSyscall("System.Runtime.CheckWitness");
    assert!(check_witness.is_some());
    assert_eq!(check_witness.unwrap().return_type, "Boolean");
    
    let notify = registry.getSyscall("System.Runtime.Notify");
    assert!(notify.is_some());
    assert_eq!(notify.unwrap().return_type, "Void");
    
    let get_platform = registry.getSyscall("System.Runtime.GetPlatform");
    assert!(get_platform.is_some());
    assert_eq!(get_platform.unwrap().return_type, "String");
}
