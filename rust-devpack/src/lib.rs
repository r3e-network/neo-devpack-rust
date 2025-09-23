//! Neo N3 Rust Development Pack
//! 
//! Complete Rust SDK for Neo N3 smart contract development

#![cfg_attr(not(feature = "std"), no_std)]

// Re-export specific items to avoid conflicts
pub use neo_types::{
    NeoInteger, NeoBoolean, NeoByteString, NeoString,
    NeoArray, NeoMap, NeoStruct, NeoValue, NeoResult, NeoError,
    NeoContract as NeoContractStruct, NeoContractEntry, NeoContractMethod,
    NeoStorageContext, NeoContractManifest, NeoContractParameter, NeoContractEvent,
    NeoContractMethodTrait
};
pub use neo_syscalls::*;
pub use neo_runtime::{
    NeoRuntime, NeoStorage, NeoCrypto, NeoJSON, NeoIterator as NeoRuntimeIterator, NeoIteratorFactory
};
pub use neo_macros::*;

/// Neo N3 Prelude - commonly used items
pub mod prelude {
    pub use crate::{
        NeoInteger, NeoBoolean, NeoByteString, NeoString,
        NeoArray, NeoMap, NeoStruct, NeoValue, NeoResult, NeoError,
        NeoContractStruct as NeoContract, NeoContractEntry, NeoContractMethod,
        NeoRuntime, NeoStorage, NeoCrypto, NeoJSON, NeoRuntimeIterator as NeoIterator,
        neo_contract, neo_method, neo_event, neo_entry,
    };
}

/// Neo N3 Contract Example
/// 
/// ```rust
/// use neo_devpack::prelude::*;
/// 
/// #[neo_contract]
/// pub struct HelloWorld {
///     greeting: NeoString,
/// }
/// 
/// impl HelloWorld {
///     #[neo_method]
///     pub fn say_hello(&self) -> NeoResult<NeoString> {
///         Ok(self.greeting.clone())
///     }
/// }
/// ```
pub struct ExampleContract;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_neo_types() {
        let int = NeoInteger::new(42);
        assert_eq!(int.as_i32(), 42);
        
        let bool_val = NeoBoolean::new(true);
        assert_eq!(bool_val.as_bool(), true);
        
        let string = NeoString::from_str("Hello, Neo!");
        assert_eq!(string.as_str(), "Hello, Neo!");
    }
}
