// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

// Complete Neo N3 Smart Contract Example
// Demonstrates all features of the Neo N3 Rust devpack

use neo_devpack::neo_storage;
use neo_devpack::prelude::*;
use neo_devpack::serde::{Deserialize, Serialize};

/// Complete Smart Contract
///
/// This example demonstrates a complete smart contract with all Neo N3 features:
/// - Contract definition and methods
/// - Storage operations
/// - Event emission
/// - System call integration
/// - Error handling
/// - Testing

#[neo_contract]
pub struct CompleteContract {
    name: NeoString,
    version: NeoString,
    owner: NeoByteString,
    total_supply: NeoInteger,
    decimals: NeoInteger,
}

/// Contract Storage
#[derive(Default, Serialize, Deserialize)]
#[neo_storage]
pub struct CompleteStorage {
    balances: NeoMap<NeoByteString, NeoInteger>,
    allowances: NeoMap<NeoByteString, NeoMap<NeoByteString, NeoInteger>>,
    metadata: NeoMap<NeoString, NeoValue>,
    counters: NeoMap<NeoString, NeoInteger>,
}

impl CompleteStorage {
    // The #[neo_storage] macro generates load and save methods
}

/// Transfer Event
#[neo_event]
pub struct TransferEvent {
    pub from: NeoByteString,
    pub to: NeoByteString,
    pub amount: NeoInteger,
}

/// Approval Event
#[neo_event]
pub struct ApprovalEvent {
    pub owner: NeoByteString,
    pub spender: NeoByteString,
    pub amount: NeoInteger,
}

/// Contract Update Event
#[neo_event]
pub struct ContractUpdateEvent {
    pub old_version: NeoString,
    pub new_version: NeoString,
    pub updated_by: NeoByteString,
}

impl CompleteContract {
    /// Create a new complete contract
    pub fn new(
        name: NeoString,
        version: NeoString,
        total_supply: NeoInteger,
        decimals: NeoInteger,
    ) -> Self {
        Self {
            name,
            version,
            owner: NeoByteString::new(vec![]),
            total_supply,
            decimals,
        }
    }

    /// Initialize the contract
    #[neo_method]
    pub fn initialize(&mut self) -> NeoResult<()> {
        // Set owner to caller
        self.owner = NeoRuntime::get_calling_script_hash()?;

        // Reset storage to a clean deployment state.
        let mut storage = CompleteStorage::default();

        // Set initial metadata
        storage.metadata.insert(
            NeoString::from_str("name"),
            NeoValue::from(self.name.clone()),
        );
        storage.metadata.insert(
            NeoString::from_str("version"),
            NeoValue::from(self.version.clone()),
        );
        storage.metadata.insert(
            NeoString::from_str("total_supply"),
            NeoValue::from(self.total_supply.clone()),
        );
        storage.metadata.insert(
            NeoString::from_str("decimals"),
            NeoValue::from(self.decimals.clone()),
        );

        // Initialize counters
        storage
            .counters
            .insert(NeoString::from_str("transfer_count"), NeoInteger::zero());
        storage
            .counters
            .insert(NeoString::from_str("approval_count"), NeoInteger::zero());

        // Distribute initial supply to owner
        storage
            .balances
            .insert(self.owner.clone(), self.total_supply.clone());

        // Save storage
        storage.save(&NeoRuntime::get_storage_context()?)?;

        // Emit initialization event
        let event = ContractUpdateEvent {
            old_version: NeoString::from_str("0.0.0"),
            new_version: self.version.clone(),
            updated_by: self.owner.clone(),
        };
        event.emit()?;

        Ok(())
    }

    /// Get contract name
    #[neo_method]
    pub fn name(&self) -> NeoResult<NeoString> {
        Ok(self.name.clone())
    }

    /// Get contract version
    #[neo_method]
    pub fn version(&self) -> NeoResult<NeoString> {
        Ok(self.version.clone())
    }

    /// Get contract owner
    #[neo_method]
    pub fn owner(&self) -> NeoResult<NeoByteString> {
        Ok(self.owner.clone())
    }

    /// Get total supply
    #[neo_method]
    pub fn total_supply(&self) -> NeoResult<NeoInteger> {
        Ok(self.total_supply.clone())
    }

    /// Get decimals
    #[neo_method]
    pub fn decimals(&self) -> NeoResult<NeoInteger> {
        Ok(self.decimals.clone())
    }

    /// Get balance of an account
    #[neo_method]
    pub fn balance_of(&self, account: &NeoByteString) -> NeoResult<NeoInteger> {
        let storage = CompleteStorage::load(&NeoRuntime::get_storage_context()?);
        Ok(storage
            .balances
            .get(account)
            .cloned()
            .unwrap_or(NeoInteger::zero()))
    }

    /// Transfer tokens
    #[neo_method]
    pub fn transfer(&mut self, to: &NeoByteString, amount: NeoInteger) -> NeoResult<NeoBoolean> {
        // Validate amount
        if amount <= NeoInteger::zero() {
            return Ok(NeoBoolean::FALSE);
        }

        // Get caller
        let from = NeoRuntime::get_calling_script_hash()?;

        // Check balance
        let balance = self.balance_of(&from)?;
        if balance < amount {
            return Ok(NeoBoolean::FALSE);
        }

        // Update balances
        let mut storage = CompleteStorage::load(&NeoRuntime::get_storage_context()?);

        // Subtract from sender
        let new_from_balance = balance - amount.clone();
        storage.balances.insert(from.clone(), new_from_balance);

        // Add to receiver
        let to_balance = storage
            .balances
            .get(to)
            .cloned()
            .unwrap_or(NeoInteger::zero());
        let new_to_balance = to_balance + amount.clone();
        storage.balances.insert(to.clone(), new_to_balance);

        // Update transfer counter
        let transfer_count = storage
            .counters
            .get(&NeoString::from_str("transfer_count"))
            .cloned()
            .unwrap_or(NeoInteger::zero());
        storage.counters.insert(
            NeoString::from_str("transfer_count"),
            transfer_count + NeoInteger::one(),
        );

        // Save storage
        storage.save(&NeoRuntime::get_storage_context()?)?;

        // Emit transfer event
        let event = TransferEvent {
            from,
            to: to.clone(),
            amount: amount.clone(),
        };
        event.emit()?;

        Ok(NeoBoolean::TRUE)
    }

    /// Approve spender
    #[neo_method]
    pub fn approve(
        &mut self,
        spender: &NeoByteString,
        amount: NeoInteger,
    ) -> NeoResult<NeoBoolean> {
        // Validate amount
        if amount < NeoInteger::zero() {
            return Ok(NeoBoolean::FALSE);
        }

        // Get caller
        let owner = NeoRuntime::get_calling_script_hash()?;

        // Update allowances
        let mut storage = CompleteStorage::load(&NeoRuntime::get_storage_context()?);

        let mut owner_allowances = storage
            .allowances
            .get(&owner)
            .cloned()
            .unwrap_or(NeoMap::new());

        owner_allowances.insert(spender.clone(), amount.clone());
        storage.allowances.insert(owner.clone(), owner_allowances);

        // Update approval counter
        let approval_count = storage
            .counters
            .get(&NeoString::from_str("approval_count"))
            .cloned()
            .unwrap_or(NeoInteger::zero());
        storage.counters.insert(
            NeoString::from_str("approval_count"),
            approval_count + NeoInteger::one(),
        );

        // Save storage
        storage.save(&NeoRuntime::get_storage_context()?)?;

        // Emit approval event
        let event = ApprovalEvent {
            owner,
            spender: spender.clone(),
            amount: amount.clone(),
        };
        event.emit()?;

        Ok(NeoBoolean::TRUE)
    }

    /// Get allowance
    #[neo_method]
    pub fn allowance(
        &self,
        owner: &NeoByteString,
        spender: &NeoByteString,
    ) -> NeoResult<NeoInteger> {
        let storage = CompleteStorage::load(&NeoRuntime::get_storage_context()?);

        if let Some(owner_allowances) = storage.allowances.get(owner) {
            Ok(owner_allowances
                .get(spender)
                .cloned()
                .unwrap_or(NeoInteger::zero()))
        } else {
            Ok(NeoInteger::zero())
        }
    }

    /// Transfer from (approved transfer)
    #[neo_method]
    pub fn transfer_from(
        &mut self,
        from: &NeoByteString,
        to: &NeoByteString,
        amount: NeoInteger,
    ) -> NeoResult<NeoBoolean> {
        // Validate amount
        if amount <= NeoInteger::zero() {
            return Ok(NeoBoolean::FALSE);
        }

        // Get caller
        let spender = NeoRuntime::get_calling_script_hash()?;

        // Check allowance
        let allowance = self.allowance(from, &spender)?;
        if allowance < amount {
            return Ok(NeoBoolean::FALSE);
        }

        // Check balance
        let balance = self.balance_of(from)?;
        if balance < amount {
            return Ok(NeoBoolean::FALSE);
        }

        // Update balances
        let mut storage = CompleteStorage::load(&NeoRuntime::get_storage_context()?);

        // Subtract from sender
        let new_from_balance = balance - amount.clone();
        storage.balances.insert(from.clone(), new_from_balance);

        // Add to receiver
        let to_balance = storage
            .balances
            .get(to)
            .cloned()
            .unwrap_or(NeoInteger::zero());
        let new_to_balance = to_balance + amount.clone();
        storage.balances.insert(to.clone(), new_to_balance);

        // Update allowance
        let mut owner_allowances = storage
            .allowances
            .get(from)
            .cloned()
            .unwrap_or(NeoMap::new());

        let new_allowance = allowance - amount.clone();
        owner_allowances.insert(spender, new_allowance);
        storage.allowances.insert(from.clone(), owner_allowances);

        // Update transfer counter
        let transfer_count = storage
            .counters
            .get(&NeoString::from_str("transfer_count"))
            .cloned()
            .unwrap_or(NeoInteger::zero());
        storage.counters.insert(
            NeoString::from_str("transfer_count"),
            transfer_count + NeoInteger::one(),
        );

        // Save storage
        storage.save(&NeoRuntime::get_storage_context()?)?;

        // Emit transfer event
        let event = TransferEvent {
            from: from.clone(),
            to: to.clone(),
            amount: amount.clone(),
        };
        event.emit()?;

        Ok(NeoBoolean::TRUE)
    }

    /// Get transfer count
    #[neo_method]
    pub fn get_transfer_count(&self) -> NeoResult<NeoInteger> {
        let storage = CompleteStorage::load(&NeoRuntime::get_storage_context()?);
        Ok(storage
            .counters
            .get(&NeoString::from_str("transfer_count"))
            .cloned()
            .unwrap_or(NeoInteger::zero()))
    }

    /// Get approval count
    #[neo_method]
    pub fn get_approval_count(&self) -> NeoResult<NeoInteger> {
        let storage = CompleteStorage::load(&NeoRuntime::get_storage_context()?);
        Ok(storage
            .counters
            .get(&NeoString::from_str("approval_count"))
            .cloned()
            .unwrap_or(NeoInteger::zero()))
    }

    /// Get contract metadata
    #[neo_method]
    pub fn get_metadata(&self, key: &NeoString) -> NeoResult<NeoValue> {
        let storage = CompleteStorage::load(&NeoRuntime::get_storage_context()?);
        Ok(storage.metadata.get(key).cloned().unwrap_or(NeoValue::Null))
    }

    /// Set contract metadata (owner only)
    #[neo_method]
    pub fn set_metadata(&mut self, key: NeoString, value: NeoValue) -> NeoResult<NeoBoolean> {
        // Check if caller is owner
        let caller = NeoRuntime::get_calling_script_hash()?;
        if caller != self.owner {
            return Ok(NeoBoolean::FALSE);
        }

        // Update metadata
        let mut storage = CompleteStorage::load(&NeoRuntime::get_storage_context()?);
        storage.metadata.insert(key, value);
        storage.save(&NeoRuntime::get_storage_context()?)?;

        Ok(NeoBoolean::TRUE)
    }

    /// Update contract (owner only)
    #[neo_method]
    pub fn update_contract(&mut self, new_version: NeoString) -> NeoResult<NeoBoolean> {
        // Check if caller is owner
        let caller = NeoRuntime::get_calling_script_hash()?;
        if caller != self.owner {
            return Ok(NeoBoolean::FALSE);
        }

        // Update version
        let old_version = self.version.clone();
        self.version = new_version.clone();

        // Update storage
        let mut storage = CompleteStorage::load(&NeoRuntime::get_storage_context()?);
        storage.metadata.insert(
            NeoString::from_str("version"),
            NeoValue::from(new_version.clone()),
        );
        storage.save(&NeoRuntime::get_storage_context()?)?;

        // Emit update event
        let event = ContractUpdateEvent {
            old_version,
            new_version,
            updated_by: caller,
        };
        event.emit()?;

        Ok(NeoBoolean::TRUE)
    }

    /// Get contract statistics
    #[neo_method]
    pub fn get_statistics(&self) -> NeoResult<NeoValue> {
        let storage = CompleteStorage::load(&NeoRuntime::get_storage_context()?);

        let mut stats = NeoStruct::new();
        stats.set_field("total_supply", NeoValue::from(self.total_supply.clone()));
        stats.set_field("decimals", NeoValue::from(self.decimals.clone()));
        stats.set_field(
            "transfer_count",
            NeoValue::from(
                storage
                    .counters
                    .get(&NeoString::from_str("transfer_count"))
                    .cloned()
                    .unwrap_or(NeoInteger::zero()),
            ),
        );
        stats.set_field(
            "approval_count",
            NeoValue::from(
                storage
                    .counters
                    .get(&NeoString::from_str("approval_count"))
                    .cloned()
                    .unwrap_or(NeoInteger::zero()),
            ),
        );
        stats.set_field(
            "balance_count",
            NeoValue::from(NeoInteger::new(storage.balances.len() as i32)),
        );
        stats.set_field(
            "allowance_count",
            NeoValue::from(NeoInteger::new(storage.allowances.len() as i32)),
        );

        Ok(NeoValue::from(stats))
    }

    /// Emergency pause (owner only)
    #[neo_method]
    pub fn emergency_pause(&mut self) -> NeoResult<NeoBoolean> {
        // Check if caller is owner
        let caller = NeoRuntime::get_calling_script_hash()?;
        if caller != self.owner {
            return Ok(NeoBoolean::FALSE);
        }

        // Set emergency flag
        let mut storage = CompleteStorage::load(&NeoRuntime::get_storage_context()?);
        storage.metadata.insert(
            NeoString::from_str("emergency_paused"),
            NeoValue::from(NeoBoolean::TRUE),
        );
        storage.save(&NeoRuntime::get_storage_context()?)?;

        // Log emergency pause
        NeoRuntime::log(&NeoString::from_str("Contract emergency paused"))?;

        Ok(NeoBoolean::TRUE)
    }

    /// Emergency unpause (owner only)
    #[neo_method]
    pub fn emergency_unpause(&mut self) -> NeoResult<NeoBoolean> {
        // Check if caller is owner
        let caller = NeoRuntime::get_calling_script_hash()?;
        if caller != self.owner {
            return Ok(NeoBoolean::FALSE);
        }

        // Remove emergency flag
        let mut storage = CompleteStorage::load(&NeoRuntime::get_storage_context()?);
        storage.metadata.insert(
            NeoString::from_str("emergency_paused"),
            NeoValue::from(NeoBoolean::FALSE),
        );
        storage.save(&NeoRuntime::get_storage_context()?)?;

        // Log emergency unpause
        NeoRuntime::log(&NeoString::from_str("Contract emergency unpaused"))?;

        Ok(NeoBoolean::TRUE)
    }

    /// Check if contract is paused
    #[neo_method]
    pub fn is_paused(&self) -> NeoResult<NeoBoolean> {
        let storage = CompleteStorage::load(&NeoRuntime::get_storage_context()?);
        let paused = storage
            .metadata
            .get(&NeoString::from_str("emergency_paused"))
            .cloned()
            .unwrap_or(NeoValue::from(NeoBoolean::FALSE));

        Ok(paused.as_boolean().unwrap_or(NeoBoolean::FALSE))
    }
}

/// Contract deployment entry point
pub fn deploy_contract() -> NeoResult<()> {
    let contract = CompleteContract::new(
        NeoString::from_str("CompleteContract"),
        NeoString::from_str("1.0.0"),
        NeoInteger::new(1000000), // 1 million tokens
        NeoInteger::new(8),       // 8 decimals
    );

    // Initialize contract
    let mut initialized_contract = contract;
    initialized_contract.initialize()?;

    // Log deployment
    NeoRuntime::log(&NeoString::from_str(
        "CompleteContract deployed successfully",
    ))?;

    Ok(())
}

/// Contract update entry point
pub fn update_contract() -> NeoResult<()> {
    // Update contract logic
    NeoRuntime::log(&NeoString::from_str("CompleteContract updated"))?;
    Ok(())
}

/// Contract destroy entry point
pub fn destroy_contract() -> NeoResult<()> {
    // Clean up contract resources
    NeoRuntime::log(&NeoString::from_str("CompleteContract destroyed"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, MutexGuard, OnceLock};

    fn test_lock() -> MutexGuard<'static, ()> {
        static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        TEST_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("example test lock poisoned")
    }

    #[test]
    fn test_complete_contract_creation() {
        let _guard = test_lock();
        let contract = CompleteContract::new(
            NeoString::from_str("TestContract"),
            NeoString::from_str("1.0.0"),
            NeoInteger::new(1000000),
            NeoInteger::new(8),
        );

        assert_eq!(contract.name().unwrap().as_str(), "TestContract");
        assert_eq!(contract.version().unwrap().as_str(), "1.0.0");
        assert_eq!(
            contract.total_supply().unwrap().as_i32_saturating(),
            1000000
        );
        assert_eq!(contract.decimals().unwrap().as_i32_saturating(), 8);
    }

    #[test]
    fn test_complete_contract_operations() {
        let _guard = test_lock();
        let mut contract = CompleteContract::new(
            NeoString::from_str("TestContract"),
            NeoString::from_str("1.0.0"),
            NeoInteger::new(1000000),
            NeoInteger::new(8),
        );

        // Test initialization
        contract.initialize().unwrap();

        // Test balance operations
        let owner = contract.owner().unwrap();
        let account2 = NeoByteString::from_slice(b"account2");

        // Test transfer
        let transfer_result = contract.transfer(&account2, NeoInteger::new(100));
        assert_eq!(transfer_result.unwrap().as_bool(), true);

        // Test approval
        let approval_result = contract.approve(&owner, NeoInteger::new(50));
        assert_eq!(approval_result.unwrap().as_bool(), true);

        // Test allowance
        let allowance = contract.allowance(&owner, &owner);
        assert_eq!(allowance.unwrap().as_i32_saturating(), 50);

        // Test transfer from
        let transfer_from_result = contract.transfer_from(&owner, &account2, NeoInteger::new(25));
        assert_eq!(transfer_from_result.unwrap().as_bool(), true);

        // Test statistics
        let stats = contract.get_statistics().unwrap();
        assert!(!stats.is_null());
    }

    #[test]
    fn test_complete_contract_metadata() {
        let _guard = test_lock();
        let mut contract = CompleteContract::new(
            NeoString::from_str("TestContract"),
            NeoString::from_str("1.0.0"),
            NeoInteger::new(1000000),
            NeoInteger::new(8),
        );

        contract.initialize().unwrap();

        // Test metadata operations
        let key = NeoString::from_str("test_key");
        let value = NeoValue::from(NeoString::from_str("test_value"));

        // Test set metadata
        let set_result = contract.set_metadata(key.clone(), value.clone());
        assert_eq!(set_result.unwrap().as_bool(), true);

        // Test get metadata
        let get_result = contract.get_metadata(&key);
        assert!(get_result.is_ok());
        assert!(!get_result.unwrap().is_null());
    }

    #[test]
    fn test_complete_contract_emergency() {
        let _guard = test_lock();
        let mut contract = CompleteContract::new(
            NeoString::from_str("TestContract"),
            NeoString::from_str("1.0.0"),
            NeoInteger::new(1000000),
            NeoInteger::new(8),
        );

        contract.initialize().unwrap();

        // Test emergency pause
        let pause_result = contract.emergency_pause();
        assert_eq!(pause_result.unwrap().as_bool(), true);

        // Test is paused
        let is_paused = contract.is_paused();
        assert_eq!(is_paused.unwrap().as_bool(), true);

        // Test emergency unpause
        let unpause_result = contract.emergency_unpause();
        assert_eq!(unpause_result.unwrap().as_bool(), true);

        // Test is not paused
        let is_not_paused = contract.is_paused();
        assert_eq!(is_not_paused.unwrap().as_bool(), false);
    }

    #[test]
    fn test_complete_contract_update() {
        let _guard = test_lock();
        let mut contract = CompleteContract::new(
            NeoString::from_str("TestContract"),
            NeoString::from_str("1.0.0"),
            NeoInteger::new(1000000),
            NeoInteger::new(8),
        );

        contract.initialize().unwrap();

        // Test contract update
        let new_version = NeoString::from_str("2.0.0");
        let update_result = contract.update_contract(new_version.clone());
        assert_eq!(update_result.unwrap().as_bool(), true);

        // Test version update
        let updated_version = contract.version();
        assert_eq!(updated_version.unwrap().as_str(), "2.0.0");
    }
}

/// Main function for the complete contract example
pub fn main() -> NeoResult<()> {
    let contract = CompleteContract::new(
        NeoString::from_str("CompleteContract"),
        NeoString::from_str("1.0.0"),
        NeoInteger::new(1000000),
        NeoInteger::new(8),
    );

    // Test basic contract operations
    let name = contract.name()?;
    let version = contract.version()?;

    let event = NeoString::from_str("CompleteContractInitialized");
    let state = NeoArray::from_vec(vec![
        NeoValue::String(NeoString::from_str("Complete contract initialized")),
        NeoValue::String(name),
        NeoValue::String(version),
    ]);
    NeoRuntime::notify(&event, &state)?;

    Ok(())
}
