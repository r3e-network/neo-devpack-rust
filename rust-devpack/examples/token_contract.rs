// Copyright (c) 2025 R3E Network
// Licensed under the MIT License

//! Token Contract Example
//!
//! This example demonstrates a more complex Neo N3 smart contract
//! with storage, events, and advanced functionality.

use neo_devpack::neo_storage;
use neo_devpack::prelude::*;
use neo_devpack::serde::{Deserialize, Serialize};

/// Token Transfer Event
#[neo_event]
pub struct TransferEvent {
    pub from: NeoByteString,
    pub to: NeoByteString,
    pub amount: NeoInteger,
}

/// Token Contract
///
/// A complete token implementation with transfer, balance, and approval functionality
#[neo_contract]
pub struct TokenContract {
    name: NeoString,
    symbol: NeoString,
    decimals: NeoInteger,
    total_supply: NeoInteger,
}

/// Token Storage
#[derive(Default, Serialize, Deserialize)]
#[neo_storage]
pub struct TokenStorage {
    balances: NeoMap<NeoByteString, NeoInteger>,
    allowances: NeoMap<NeoByteString, NeoMap<NeoByteString, NeoInteger>>,
}

impl TokenStorage {
    // The #[neo_storage] macro generates load and save methods
}

impl TokenContract {
    /// Create a new token contract
    pub fn new(
        name: NeoString,
        symbol: NeoString,
        decimals: NeoInteger,
        total_supply: NeoInteger,
    ) -> Self {
        Self {
            name,
            symbol,
            decimals,
            total_supply,
        }
    }

    /// Get token name
    #[neo_method]
    pub fn name(&self) -> NeoResult<NeoString> {
        Ok(self.name.clone())
    }

    /// Get token symbol
    #[neo_method]
    pub fn symbol(&self) -> NeoResult<NeoString> {
        Ok(self.symbol.clone())
    }

    /// Get token decimals
    #[neo_method]
    pub fn decimals(&self) -> NeoResult<NeoInteger> {
        Ok(self.decimals.clone())
    }

    /// Get total supply
    #[neo_method]
    pub fn total_supply(&self) -> NeoResult<NeoInteger> {
        Ok(self.total_supply.clone())
    }

    /// Get balance of an account
    #[neo_method]
    pub fn balance_of(&self, account: &NeoByteString) -> NeoResult<NeoInteger> {
        let storage = TokenStorage::load(&NeoRuntime::get_storage_context()?);
        Ok(storage
            .balances
            .get(account)
            .cloned()
            .unwrap_or(NeoInteger::zero()))
    }

    /// Transfer tokens
    #[neo_method]
    pub fn transfer(&mut self, to: &NeoByteString, amount: NeoInteger) -> NeoResult<NeoBoolean> {
        // Check if amount is positive
        if amount <= NeoInteger::zero() {
            return Ok(NeoBoolean::FALSE);
        }

        // Get caller's address
        let from = NeoRuntime::get_calling_script_hash()?;

        // Check balance
        let balance = self.balance_of(&from)?;
        if balance < amount {
            return Ok(NeoBoolean::FALSE);
        }

        // Update balances
        let mut storage = TokenStorage::load(&NeoRuntime::get_storage_context()?);

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
        // Check if amount is positive
        if amount < NeoInteger::zero() {
            return Ok(NeoBoolean::FALSE);
        }

        // Get caller's address
        let owner = NeoRuntime::get_calling_script_hash()?;

        // Update allowances
        let mut storage = TokenStorage::load(&NeoRuntime::get_storage_context()?);

        let mut owner_allowances = storage
            .allowances
            .get(&owner)
            .cloned()
            .unwrap_or(NeoMap::new());

        owner_allowances.insert(spender.clone(), amount.clone());
        storage.allowances.insert(owner, owner_allowances);

        // Save storage
        storage.save(&NeoRuntime::get_storage_context()?)?;

        Ok(NeoBoolean::TRUE)
    }

    /// Get allowance
    #[neo_method]
    pub fn allowance(
        &self,
        owner: &NeoByteString,
        spender: &NeoByteString,
    ) -> NeoResult<NeoInteger> {
        let storage = TokenStorage::load(&NeoRuntime::get_storage_context()?);

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
        // Check if amount is positive
        if amount <= NeoInteger::zero() {
            return Ok(NeoBoolean::FALSE);
        }

        // Get caller's address
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
        let mut storage = TokenStorage::load(&NeoRuntime::get_storage_context()?);

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
}

/// Contract deployment entry point
pub fn deploy_contract() -> NeoResult<()> {
    // Initialize token with initial supply
    let total_supply = NeoInteger::new(1000000); // 1 million tokens
    let _contract = TokenContract::new(
        NeoString::from_str("Neo Token"),
        NeoString::from_str("NEO"),
        NeoInteger::new(8), // 8 decimals
        total_supply.clone(),
    );

    // Distribute initial supply to deployer
    let deployer = NeoRuntime::get_calling_script_hash()?;
    let mut storage = TokenStorage::load(&NeoRuntime::get_storage_context()?);
    storage.balances.insert(deployer, total_supply.clone());
    storage.save(&NeoRuntime::get_storage_context()?)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation() {
        let contract = TokenContract::new(
            NeoString::from_str("Test Token"),
            NeoString::from_str("TEST"),
            NeoInteger::new(8),
            NeoInteger::new(1000000),
        );

        assert_eq!(contract.name().unwrap().as_str(), "Test Token");
        assert_eq!(contract.symbol().unwrap().as_str(), "TEST");
        assert_eq!(contract.decimals().unwrap().as_i32_saturating(), 8);
        assert_eq!(
            contract.total_supply().unwrap().as_i32_saturating(),
            1000000
        );
    }

    #[test]
    fn test_token_operations() {
        let mut contract = TokenContract::new(
            NeoString::from_str("Test Token"),
            NeoString::from_str("TEST"),
            NeoInteger::new(8),
            NeoInteger::new(1000000),
        );

        let account1 = NeoByteString::from_slice(b"account1");
        let account2 = NeoByteString::from_slice(b"account2");

        // Test balance (should be 0 initially)
        assert_eq!(
            contract.balance_of(&account1).unwrap().as_i32_saturating(),
            0
        );

        // Test transfer (should fail due to insufficient balance)
        let transfer_result = contract.transfer(&account2, NeoInteger::new(100));
        assert_eq!(transfer_result.unwrap().as_bool(), false);
    }
}

/// Main function for the token contract example
pub fn main() -> NeoResult<()> {
    let contract = TokenContract::new(
        NeoString::from_str("MyToken"),
        NeoString::from_str("MTK"),
        NeoInteger::new(8),
        NeoInteger::new(1000000),
    );

    // Test basic token operations
    let account = NeoByteString::from_slice(b"account1");
    let balance = contract.balance_of(&account)?;

    let event = NeoString::from_str("TokenInitialized");
    let state = NeoArray::from_vec(vec![
        NeoValue::String(NeoString::from_str("Token contract initialized")),
        NeoValue::Integer(balance),
    ]);
    NeoRuntime::notify(&event, &state)?;

    Ok(())
}
