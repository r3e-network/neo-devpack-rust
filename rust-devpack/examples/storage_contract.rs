//! Storage Contract Example
//!
//! This example demonstrates Neo N3 storage operations and data persistence.

use neo_devpack::prelude::*;
use neo_devpack::serde::{Deserialize, Serialize};
use neo_devpack::{neo_serialize, neo_storage};

/// User Data Structure
#[derive(Clone, Default, Serialize, Deserialize)]
#[neo_serialize]
pub struct UserData {
    pub name: NeoString,
    pub age: NeoInteger,
    pub email: NeoString,
    pub balance: NeoInteger,
}

/// Storage Contract
///
/// Demonstrates various storage operations and data management
#[neo_contract]
pub struct StorageContract {
    owner: NeoByteString,
    user_count: NeoInteger,
}

/// Contract Storage
#[derive(Default, Serialize, Deserialize)]
#[neo_storage]
pub struct ContractStorage {
    users: NeoMap<NeoByteString, UserData>,
    settings: NeoMap<NeoString, NeoValue>,
    counters: NeoMap<NeoString, NeoInteger>,
}

impl ContractStorage {
    // The #[neo_storage] macro generates load and save methods
}

impl StorageContract {
    /// Create a new storage contract
    pub fn new() -> Self {
        Self {
            owner: NeoByteString::new(vec![]),
            user_count: NeoInteger::zero(),
        }
    }

    /// Initialize the contract
    #[neo_method]
    pub fn initialize(&mut self) -> NeoResult<()> {
        // Set owner
        self.owner = NeoRuntime::get_calling_script_hash()?;
        self.user_count = NeoInteger::zero();

        // Initialize storage
        let context = NeoRuntime::get_storage_context()?;
        let mut storage = ContractStorage::load(&context);
        storage
            .counters
            .insert(NeoString::from_str("user_count"), NeoInteger::zero());
        storage.save(&context)?;

        Ok(())
    }

    /// Register a new user
    #[neo_method]
    pub fn register_user(
        &mut self,
        user_id: &NeoByteString,
        user_data: UserData,
    ) -> NeoResult<NeoBoolean> {
        let context = NeoRuntime::get_storage_context()?;
        let mut storage = ContractStorage::load(&context);
        if storage.users.get(user_id).is_some() {
            return Ok(NeoBoolean::FALSE);
        }

        // Store user
        storage.users.insert(user_id.clone(), user_data);

        // Update user count
        let current_count = storage
            .counters
            .get(&NeoString::from_str("user_count"))
            .cloned()
            .unwrap_or(NeoInteger::zero());
        let new_count = current_count + NeoInteger::one();
        storage
            .counters
            .insert(NeoString::from_str("user_count"), new_count.clone());
        self.user_count = new_count.clone();

        // Save storage
        storage.save(&context)?;

        Ok(NeoBoolean::TRUE)
    }

    /// Add a new user
    #[neo_method]
    pub fn add_user(
        &mut self,
        user_id: &NeoByteString,
        name: NeoString,
        age: NeoInteger,
        email: NeoString,
    ) -> NeoResult<NeoBoolean> {
        let context = NeoRuntime::get_storage_context()?;
        let mut storage = ContractStorage::load(&context);
        if storage.users.get(user_id).is_some() {
            return Ok(NeoBoolean::FALSE);
        }

        // Create user data
        let user_data = UserData {
            name,
            age,
            email,
            balance: NeoInteger::zero(),
        };

        // Store user
        storage.users.insert(user_id.clone(), user_data);

        // Update user count
        let current_count = storage
            .counters
            .get(&NeoString::from_str("user_count"))
            .cloned()
            .unwrap_or(NeoInteger::zero());
        let new_count = current_count + NeoInteger::one();
        storage
            .counters
            .insert(NeoString::from_str("user_count"), new_count.clone());
        self.user_count = new_count;

        // Save storage
        storage.save(&context)?;

        Ok(NeoBoolean::TRUE)
    }

    /// Get user data
    #[neo_method]
    pub fn get_user(&self, user_id: &NeoByteString) -> NeoResult<NeoValue> {
        let context = NeoRuntime::get_storage_context()?;
        let storage = ContractStorage::load(&context);

        if let Some(user_data) = storage.users.get(user_id) {
            // Create a struct representation
            let mut user_struct = NeoStruct::new();
            user_struct.set_field("name", NeoValue::from(user_data.name.clone()));
            user_struct.set_field("age", NeoValue::from(user_data.age.clone()));
            user_struct.set_field("email", NeoValue::from(user_data.email.clone()));
            user_struct.set_field("balance", NeoValue::from(user_data.balance.clone()));

            Ok(NeoValue::from(user_struct))
        } else {
            Ok(NeoValue::Null)
        }
    }

    /// Update user balance
    #[neo_method]
    pub fn update_user_balance(
        &mut self,
        user_id: &NeoByteString,
        new_balance: NeoInteger,
    ) -> NeoResult<NeoBoolean> {
        let context = NeoRuntime::get_storage_context()?;
        let mut storage = ContractStorage::load(&context);

        if let Some(user_data) = storage.users.get_mut(user_id) {
            user_data.balance = new_balance;
            storage.save(&context)?;
            Ok(NeoBoolean::TRUE)
        } else {
            Ok(NeoBoolean::FALSE)
        }
    }

    /// Get user count
    #[neo_method]
    pub fn get_user_count(&self) -> NeoResult<NeoInteger> {
        Ok(self.user_count.clone())
    }

    /// Set a setting
    #[neo_method]
    pub fn set_setting(&mut self, key: NeoString, value: NeoValue) -> NeoResult<()> {
        let context = NeoRuntime::get_storage_context()?;
        let mut storage = ContractStorage::load(&context);
        storage.settings.insert(key, value);
        storage.save(&context)?;
        Ok(())
    }

    /// Get a setting
    #[neo_method]
    pub fn get_setting(&self, key: &NeoString) -> NeoResult<NeoValue> {
        let context = NeoRuntime::get_storage_context()?;
        let storage = ContractStorage::load(&context);
        Ok(storage.settings.get(key).cloned().unwrap_or(NeoValue::Null))
    }

    /// Increment a counter
    #[neo_method]
    pub fn increment_counter(&mut self, counter_name: NeoString) -> NeoResult<NeoInteger> {
        let context = NeoRuntime::get_storage_context()?;
        let mut storage = ContractStorage::load(&context);

        let current_value = storage
            .counters
            .get(&counter_name)
            .cloned()
            .unwrap_or(NeoInteger::zero());
        let new_value = current_value + NeoInteger::one();
        let persisted = new_value.clone();

        storage.counters.insert(counter_name, persisted);
        storage.save(&context)?;

        Ok(new_value)
    }

    /// Get counter value
    #[neo_method]
    pub fn get_counter(&self, counter_name: &NeoString) -> NeoResult<NeoInteger> {
        let context = NeoRuntime::get_storage_context()?;
        let storage = ContractStorage::load(&context);
        Ok(storage
            .counters
            .get(counter_name)
            .cloned()
            .unwrap_or(NeoInteger::zero()))
    }

    /// Clear all data (owner only)
    #[neo_method]
    pub fn clear_all_data(&mut self) -> NeoResult<NeoBoolean> {
        // Check if caller is owner
        let caller = NeoRuntime::get_calling_script_hash()?;
        if caller != self.owner {
            return Ok(NeoBoolean::FALSE);
        }

        // Clear storage
        let context = NeoRuntime::get_storage_context()?;
        let mut storage = ContractStorage::load(&context);
        storage.users = NeoMap::new();
        storage.settings = NeoMap::new();
        storage.counters = NeoMap::new();
        storage.save(&context)?;

        Ok(NeoBoolean::TRUE)
    }
}

/// Contract deployment entry point
pub fn deploy_contract() -> NeoResult<()> {
    let mut contract = StorageContract::new();
    contract.initialize()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_contract_creation() {
        let contract = StorageContract::new();
        assert_eq!(contract.get_user_count().unwrap().as_i32(), 0);
    }

    #[test]
    fn test_user_operations() {
        let mut contract = StorageContract::new();
        contract.initialize().unwrap();

        let user_id = NeoByteString::from_slice(b"user1");
        let name = NeoString::from_str("John Doe");
        let age = NeoInteger::new(30);
        let email = NeoString::from_str("john@example.com");

        // Add user
        let result = contract.add_user(&user_id, name.clone(), age, email.clone());
        assert_eq!(result.unwrap().as_bool(), true);

        // Check user count
        assert_eq!(contract.get_user_count().unwrap().as_i32(), 1);

        // Get user data
        let user_data = contract.get_user(&user_id).unwrap();
        assert!(!user_data.is_null());

        // Update user balance
        let new_balance = NeoInteger::new(1000);
        let update_result = contract.update_user_balance(&user_id, new_balance);
        assert_eq!(update_result.unwrap().as_bool(), true);
    }

    #[test]
    fn test_settings_and_counters() {
        let mut contract = StorageContract::new();
        contract.initialize().unwrap();

        // Test settings
        let setting_key = NeoString::from_str("max_users");
        let setting_value = NeoValue::from(NeoInteger::new(1000));
        contract
            .set_setting(setting_key.clone(), setting_value)
            .unwrap();

        let retrieved_value = contract.get_setting(&setting_key).unwrap();
        assert!(!retrieved_value.is_null());

        // Test counters
        let counter_name = NeoString::from_str("visits");
        let initial_value = contract.get_counter(&counter_name).unwrap();
        assert_eq!(initial_value.as_i32(), 0);

        let incremented_value = contract.increment_counter(counter_name.clone()).unwrap();
        assert_eq!(incremented_value.as_i32(), 1);

        let final_value = contract.get_counter(&counter_name).unwrap();
        assert_eq!(final_value.as_i32(), 1);
    }
}

/// Main function for the storage contract example
pub fn main() -> NeoResult<()> {
    let mut contract = StorageContract::new();

    // Test user registration
    let user_data = UserData {
        name: NeoString::from_str("Alice"),
        email: NeoString::from_str("alice@example.com"),
        age: NeoInteger::new(25),
        balance: NeoInteger::new(1000),
    };

    let result = contract.register_user(&NeoByteString::from_slice(b"user1"), user_data.clone())?;

    let event = NeoString::from_str("UserRegistered");
    let state = NeoArray::from_vec(vec![
        NeoValue::String(NeoString::from_str("User registered")),
        NeoValue::Boolean(result),
    ]);
    NeoRuntime::notify(&event, &state)?;

    Ok(())
}
