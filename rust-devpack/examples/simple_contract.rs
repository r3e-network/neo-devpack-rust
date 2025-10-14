//! Simple Neo N3 Contract Example
//!
//! This example demonstrates basic Neo N3 smart contract functionality
//! without complex macros.

use neo_devpack::*;

/// Simple storage struct
pub struct SimpleStorage {
    pub value: NeoInteger,
    pub name: NeoString,
}

impl Default for SimpleStorage {
    fn default() -> Self {
        Self {
            value: NeoInteger::ZERO,
            name: NeoString::from_str("default"),
        }
    }
}

impl SimpleStorage {
    pub fn load(_context: &NeoStorageContext) -> Self {
        // For now, return a default instance
        // In a real implementation, this would load from storage
        Self::default()
    }

    pub fn save(&self, _context: &NeoStorageContext) -> NeoResult<()> {
        // For now, just return Ok
        // In a real implementation, this would save to storage
        Ok(())
    }
}

/// Simple Neo N3 Contract
pub struct SimpleContract {
    storage: SimpleStorage,
}

impl SimpleContract {
    pub fn new() -> Self {
        Self {
            storage: SimpleStorage::default(),
        }
    }

    /// Get the current value
    pub fn get_value(&self) -> NeoResult<NeoInteger> {
        Ok(self.storage.value)
    }

    /// Set a new value
    pub fn set_value(&mut self, value: NeoInteger) -> NeoResult<()> {
        self.storage.value = value;
        Ok(())
    }

    /// Get the name
    pub fn get_name(&self) -> NeoResult<NeoString> {
        Ok(self.storage.name.clone())
    }

    /// Set a new name
    pub fn set_name(&mut self, name: NeoString) -> NeoResult<()> {
        self.storage.name = name;
        Ok(())
    }

    /// Add two numbers
    pub fn add(&self, a: NeoInteger, b: NeoInteger) -> NeoResult<NeoInteger> {
        Ok(NeoInteger::new(a.as_i32() + b.as_i32()))
    }

    /// Multiply two numbers
    pub fn multiply(&self, a: NeoInteger, b: NeoInteger) -> NeoResult<NeoInteger> {
        Ok(NeoInteger::new(a.as_i32() * b.as_i32()))
    }
}

/// Main function for testing
fn main() {
    println!("Simple Neo N3 Contract Example");

    // Create a new contract
    let mut contract = SimpleContract::new();

    // Test basic operations
    println!("Initial value: {}", contract.get_value().unwrap().as_i32());
    println!("Initial name: {}", contract.get_name().unwrap().as_str());

    // Set new values
    contract.set_value(NeoInteger::new(42)).unwrap();
    contract.set_name(NeoString::from_str("Hello Neo")).unwrap();

    println!("New value: {}", contract.get_value().unwrap().as_i32());
    println!("New name: {}", contract.get_name().unwrap().as_str());

    // Test arithmetic operations
    let result = contract
        .add(NeoInteger::new(10), NeoInteger::new(20))
        .unwrap();
    println!("10 + 20 = {}", result.as_i32());

    let result = contract
        .multiply(NeoInteger::new(5), NeoInteger::new(6))
        .unwrap();
    println!("5 * 6 = {}", result.as_i32());

    // Test storage operations
    let context = NeoStorageContext::new(1);
    let storage = SimpleStorage::load(&context);
    storage.save(&context).unwrap();

    println!("Storage operations completed successfully!");

    // Test syscalls
    let time = NeoVMSyscall::get_time().unwrap();
    println!("Current time: {}", time.as_i32());

    let witness = NeoVMSyscall::check_witness(&NeoByteString::from_slice(b"test")).unwrap();
    println!("Witness check: {}", witness.as_bool());

    println!("All tests passed successfully!");
}
