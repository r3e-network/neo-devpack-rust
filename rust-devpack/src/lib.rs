//! Neo N3 Rust Development Pack
//!
//! Complete Rust SDK for Neo N3 smart contract development

pub mod codec;
pub mod storage;
pub mod utils;

// Re-export specific items to avoid conflicts
pub use neo_macros::*;
pub use neo_runtime::{
    NeoContractRuntime, NeoCrypto, NeoJSON, NeoRuntime, NeoRuntimeContext, NeoStorage,
};
pub use neo_syscalls::*;
pub use neo_types::{
    NeoArray, NeoBoolean, NeoByteString, NeoContract, NeoContract as NeoContractTrait,
    NeoContractABI, NeoContractEntry, NeoContractEvent, NeoContractManifest, NeoContractMethod,
    NeoContractMethodTrait, NeoContractParameter, NeoContractPermission, NeoError, NeoInteger,
    NeoIterator, NeoMap, NeoResult, NeoStorageContext, NeoString, NeoStruct, NeoValue,
};
pub use serde;

/// Neo N3 Prelude - commonly used items
pub mod prelude {
    pub use crate::{
        neo_contract, neo_entry, neo_event, neo_manifest_overlay, neo_method, neo_permission,
        neo_safe, neo_safe_methods, neo_supported_standards, neo_trusts, serde, NeoArray,
        NeoBoolean, NeoByteString, NeoContract, NeoContractABI, NeoContractEntry, NeoContractEvent,
        NeoContractManifest, NeoContractMethod, NeoContractMethodTrait, NeoContractParameter,
        NeoContractPermission, NeoContractRuntime, NeoContractTrait, NeoCrypto, NeoError,
        NeoInteger, NeoIterator, NeoJSON, NeoMap, NeoResult, NeoRuntime, NeoRuntimeContext,
        NeoStorage, NeoStorageContext, NeoString, NeoStruct, NeoValue,
    };
}

/// Neo N3 Contract Examples
///
/// Basic Hello-World pattern showing how to expose a method that simply
/// returns contract state:
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
        assert_eq!(int.as_i32(), 42);

        let bool_val = NeoBoolean::new(true);
        assert_eq!(bool_val.as_bool(), true);

        let string = NeoString::from_str("Hello, Neo!");
        assert_eq!(string.as_str(), "Hello, Neo!");
    }
}
