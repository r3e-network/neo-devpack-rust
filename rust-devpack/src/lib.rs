// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! Neo N3 Rust Development Pack
//!
//! Complete Rust SDK for Neo N3 smart contract development

pub mod codec;
pub mod native_contracts;
pub mod standards;
pub mod storage;
pub mod utils;

// Re-export specific items to avoid conflicts
pub use native_contracts::*;
pub use neo_macros::*;
pub use neo_runtime::{
    NeoContractRuntime, NeoCrypto, NeoJSON, NeoRuntime, NeoRuntimeContext, NeoStorage, RawStorage,
    RawStorageGet,
};
pub use neo_syscalls::*;
pub use neo_types::{
    Hash160, Hash256, NeoArray, NeoBoolean, NeoByteString, NeoContract, NeoContractABI,
    NeoContractEntry, NeoContractEvent, NeoContractManifest, NeoContractMethod,
    NeoContractMethodTrait, NeoContractParameter, NeoContractPermission, NeoError, NeoInteger,
    NeoIterator, NeoMap, NeoResult, NeoStorageContext, NeoString, NeoStruct, NeoValue,
};

pub use serde;
pub use standards::*;

/// Neo N3 Prelude - commonly used items
pub mod prelude {
    pub use crate::{
        native_contracts::*, neo_contract, neo_entry, neo_event, neo_manifest_overlay, neo_method,
        neo_permission, neo_safe, neo_safe_methods, neo_supported_standards, neo_trusts, serde,
        standards::*, Hash160, Hash256, NeoArray, NeoBoolean, NeoByteString, NeoContract,
        NeoContractABI, NeoContractEntry, NeoContractEvent, NeoContractManifest, NeoContractMethod,
        NeoContractMethodTrait, NeoContractParameter, NeoContractPermission, NeoContractRuntime,
        NeoCrypto, NeoError, NeoInteger, NeoIterator, NeoJSON, NeoMap, NeoResult, NeoRuntime,
        NeoRuntimeContext, NeoStorage, NeoStorageContext, NeoString, NeoStruct, NeoValue,
        RawStorage, RawStorageGet,
    };
}

/// Neo N3 Contract Examples
///
/// Basic Hello-World pattern showing how to expose a method that simply
/// returns contract state. This shows canonical devpack syntax for contract methods and metadata:
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
///
/// A more complete storage-backed counter that demonstrates manifest overlays,
/// permissions, witnesses and event emission:
///
/// ```rust
/// use neo_devpack::prelude::*;
///
/// #[neo_event]
/// pub struct CounterIncreased {
///     pub account: NeoByteString,
///     pub new_value: NeoInteger,
/// }
///
/// neo_manifest_overlay!(r#"{
///     "name": "FamousCounter",
///     "features": { "storage": true }
/// }"#);
/// neo_permission!("*", ["balanceOf"]);
/// neo_supported_standards!(["NEP-17"]);
///
/// #[neo_contract]
/// pub struct FamousCounter;
///
/// impl FamousCounter {
///     #[neo_method]
///     pub fn increment(&self, caller: NeoByteString) -> NeoResult<NeoInteger> {
///         if !NeoRuntime::check_witness(&caller)?.as_bool() {
///             return Err(NeoError::InvalidOperation);
///         }
///
///         let context = NeoStorage::get_context()?;
///         let counter_key = NeoByteString::from_slice(b"counter");
///         NeoStorage::put(&context, &counter_key, &NeoByteString::from_slice(b"1"))?;
///
///         CounterIncreased {
///             account: caller.clone(),
///             new_value: NeoInteger::new(1),
///         }
///         .emit()?;
///
///         Ok(NeoInteger::new(1))
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
        assert_eq!(int.as_i32_saturating(), 42);

        let bool_val = NeoBoolean::new(true);
        assert!(bool_val.as_bool());

        let string = NeoString::from_str("Hello, Neo!");
        assert_eq!(string.as_str(), "Hello, Neo!");
    }
}
