//! Hello World Neo N3 Smart Contract
//! 
//! This example demonstrates the basic structure of a Neo N3 smart contract
//! using the neo-devpack Rust SDK.

use neo_devpack::prelude::*;
use neo_devpack::{neo_serialize};

/// Hello World Contract
/// 
/// A simple contract that demonstrates basic Neo N3 functionality
#[neo_contract]
pub struct HelloWorld {
    greeting: NeoString,
    counter: NeoInteger,
}

impl HelloWorld {
    /// Create a new HelloWorld contract
    pub fn new() -> Self {
        Self {
            greeting: NeoString::from_str("Hello, Neo N3!"),
            counter: NeoInteger::ZERO,
        }
    }
    
    /// Get the greeting message
    #[neo_method]
    pub fn get_greeting(&self) -> NeoResult<NeoString> {
        Ok(self.greeting.clone())
    }
    
    /// Set the greeting message
    #[neo_method]
    pub fn set_greeting(&mut self, greeting: NeoString) -> NeoResult<()> {
        self.greeting = greeting;
        Ok(())
    }
    
    /// Get the current counter value
    #[neo_method]
    pub fn get_counter(&self) -> NeoResult<NeoInteger> {
        Ok(self.counter)
    }
    
    /// Increment the counter
    #[neo_method]
    pub fn increment_counter(&mut self) -> NeoResult<NeoInteger> {
        self.counter = self.counter + NeoInteger::ONE;
        Ok(self.counter)
    }
    
    /// Say hello with the current greeting
    #[neo_method]
    pub fn say_hello(&self) -> NeoResult<NeoString> {
        let message = NeoString::from_str(&format!("{} Counter: {}", 
            self.greeting.as_str(), 
            self.counter.as_i32()
        ));
        Ok(message)
    }
}

/// Contract deployment entry point
pub fn deploy() -> NeoResult<()> {
    let _contract = HelloWorld::new();
    // Store contract in storage
    Ok(())
}

/// Contract update entry point
pub fn update() -> NeoResult<()> {
    // Update contract logic
    Ok(())
}

/// Contract destroy entry point
pub fn destroy() -> NeoResult<()> {
    // Clean up contract resources
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hello_world_creation() {
        let contract = HelloWorld::new();
        assert_eq!(contract.get_greeting().unwrap().as_str(), "Hello, Neo N3!");
        assert_eq!(contract.get_counter().unwrap().as_i32(), 0);
    }
    
    #[test]
    fn test_hello_world_operations() {
        let mut contract = HelloWorld::new();
        
        // Test greeting
        let new_greeting = NeoString::from_str("Hello, Rust!");
        contract.set_greeting(new_greeting.clone()).unwrap();
        assert_eq!(contract.get_greeting().unwrap().as_str(), "Hello, Rust!");
        
        // Test counter
        contract.increment_counter().unwrap();
        assert_eq!(contract.get_counter().unwrap().as_i32(), 1);
        
        // Test say_hello
        let hello = contract.say_hello().unwrap();
        assert!(hello.as_str().contains("Hello, Rust!"));
        assert!(hello.as_str().contains("Counter: 1"));
    }
}

/// Main function for the hello world example
pub fn main() -> NeoResult<()> {
    let contract = HelloWorld::new();
    let message = contract.say_hello()?;
    let event = NeoString::from_str("HelloWorld");
    let state = NeoArray::from_vec(vec![
        NeoValue::String(message),
    ]);
    NeoRuntime::notify(&event, &state)?;
    Ok(())
}
