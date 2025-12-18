//! Neo N3 Macros
//!
//! This crate provides procedural macros for Neo N3 smart contract development.

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, ItemFn, LitStr};

// Module declarations - these are helper modules, not proc macros
mod codegen;
mod expand;
mod parse;

/// Neo N3 Contract macro
///
/// This macro generates the necessary boilerplate for a Neo N3 smart contract.
///
/// # Example
///
/// ```rust
/// use neo_devpack::*;
///
/// #[neo_contract]
/// pub struct MyContract {
///     name: NeoString,
///     value: NeoInteger,
/// }
///
/// impl MyContract {
///     #[neo_method]
///     pub fn get_name(&self) -> NeoResult<NeoString> {
///         Ok(self.name.clone())
///     }
///
///     #[neo_method]
///     pub fn set_value(&mut self, value: NeoInteger) -> NeoResult<()> {
///         self.value = value;
///         Ok(())
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn neo_contract(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand::neo_contract(input).into()
}

/// Neo N3 Method macro
///
/// This macro marks a function as a Neo N3 contract method.
///
/// # Example
///
/// ```rust
/// #[neo_method]
/// pub fn my_method(&self, arg: NeoInteger) -> NeoResult<NeoString> {
///     // Method implementation
///     Ok(NeoString::from_str("Hello, Neo!"))
/// }
/// ```
#[proc_macro_attribute]
pub fn neo_method(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    expand::neo_method(input).into()
}

/// Neo N3 Storage macro
///
/// This macro generates storage operations for a Neo N3 contract.
///
/// # Example
///
/// ```rust
/// #[neo_storage]
/// pub struct MyStorage {
///     pub value: NeoInteger,
///     pub name: NeoString,
/// }
/// ```
#[proc_macro_attribute]
pub fn neo_storage(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand::neo_storage(input).into()
}

/// Neo N3 Entry Point macro
///
/// This macro marks a function as a Neo N3 contract entry point.
///
/// # Example
///
/// ```rust
/// #[neo_entry]
/// pub fn deploy() -> NeoResult<()> {
///     // Deployment logic
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn neo_entry(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    expand::neo_entry(input_fn).into()
}

/// Neo N3 Test macro
///
/// This macro generates test functions for a Neo N3 contract.
///
/// # Example
///
/// ```rust
/// #[neo_test]
/// pub fn test_my_contract() {
///     let contract = MyContract::new();
///     assert_eq!(contract.get_name().unwrap(), NeoString::from_str("MyContract"));
/// }
/// ```
#[proc_macro_attribute]
pub fn neo_test(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    expand::neo_test(input).into()
}

/// Neo N3 Benchmark macro
///
/// This macro generates benchmark functions for a Neo N3 contract.
///
/// # Example
///
/// ```rust
/// #[neo_bench]
/// pub fn bench_my_contract(b: &mut Bencher) {
///     b.iter(|| {
///         let contract = MyContract::new();
///         contract.get_name().unwrap()
///     });
/// }
/// ```
#[proc_macro_attribute]
pub fn neo_bench(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    expand::neo_bench(input).into()
}

//
// ============================================================================
// Proc macros moved from submodules (Rust requires them to be in lib.rs root)
// ============================================================================
//

// ---- From events.rs ----

/// Neo N3 Event macro
///
/// This macro generates the necessary boilerplate for a Neo N3 contract event.
///
/// # Example
///
/// ```rust
/// #[neo_event]
/// pub struct TransferEvent {
///     pub from: NeoByteString,
///     pub to: NeoByteString,
///     pub amount: NeoInteger,
/// }
/// ```
///
/// The macro generates:
/// - An `emit()` method that notifies the runtime with the event data
/// - Manifest metadata describing the event parameters
#[proc_macro_attribute]
pub fn neo_event(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand::neo_event(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

// ---- From manifest.rs ----

/// Neo N3 Manifest Overlay macro
///
/// Embed a JSON manifest fragment as a Wasm custom section.
///
/// # Example
///
/// ```rust
/// neo_manifest_overlay!(r#"{"name": "MyContract", "version": "1.0.0"}"#);
/// ```
#[proc_macro]
pub fn neo_manifest_overlay(input: TokenStream) -> TokenStream {
    let literal = parse_macro_input!(input as LitStr);
    expand::neo_manifest_overlay(&literal).into()
}

// ---- From permissions.rs ----

/// Declare manifest permissions and embed them as a custom section.
///
/// # Example
///
/// ```rust
/// neo_permission!("0x1234567890abcdef", ["method1", "method2"]);
/// ```
#[proc_macro]
pub fn neo_permission(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as parse::PermissionArgs);
    let contract = args.contract.value();
    let methods: Vec<String> = args.methods.into_iter().map(|lit| lit.value()).collect();
    expand::neo_permission(contract, methods).into()
}

/// Declare trusted contracts for the contract manifest.
///
/// # Example
///
/// ```rust
/// neo_trusts!(["0x1234567890abcdef", "0xfedcba0987654321"]);
/// ```
#[proc_macro]
pub fn neo_trusts(input: TokenStream) -> TokenStream {
    let list = parse_macro_input!(input as parse::StringList);
    let trusts: Vec<String> = list.items.into_iter().map(|lit| lit.value()).collect();
    expand::neo_trusts(trusts).into()
}

// ---- From safe_methods.rs ----

/// Declare safe methods for the contract manifest.
///
/// # Example
///
/// ```rust
/// neo_safe_methods!(["balanceOf", "symbol", "decimals"]);
/// ```
#[proc_macro]
pub fn neo_safe_methods(input: TokenStream) -> TokenStream {
    let list = parse_macro_input!(input as parse::StringList);
    let methods: Vec<String> = list.items.into_iter().map(|lit| lit.value()).collect();
    expand::neo_safe_methods(methods).into()
}

/// Mark a single exported function as safe in the manifest.
///
/// # Example
///
/// ```rust
/// #[neo_safe]
/// pub fn balance_of(owner: NeoByteString) -> NeoInteger {
///     // Implementation
/// }
/// ```
#[proc_macro_attribute]
pub fn neo_safe(_args: TokenStream, input: TokenStream) -> TokenStream {
    let func = parse_macro_input!(input as ItemFn);
    expand::neo_safe(func).into()
}

// ---- From standards.rs ----

/// Declare supported standards for the contract manifest.
///
/// # Example
///
/// ```rust
/// neo_supported_standards!(["NEP-17", "NEP-11"]);
/// ```
#[proc_macro]
pub fn neo_supported_standards(input: TokenStream) -> TokenStream {
    let list = parse_macro_input!(input as parse::StringList);
    let standards: Vec<String> = list.items.into_iter().map(|lit| lit.value()).collect();
    expand::neo_supported_standards(standards).into()
}

/// Neo N3 Documentation macro
///
/// This macro generates documentation for a Neo N3 contract.
///
/// # Example
///
/// ```rust
/// #[neo_doc]
/// pub struct MyContract {
///     /// The name of the contract
///     pub name: NeoString,
///     /// The value of the contract
///     pub value: NeoInteger,
/// }
/// ```
#[proc_macro_attribute]
pub fn neo_doc(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand::neo_doc(input).into()
}

/// Neo N3 Configuration macro
///
/// This macro generates configuration for a Neo N3 contract.
///
/// # Example
///
/// ```rust
/// #[neo_config]
/// pub struct MyConfig {
///     pub max_value: NeoInteger,
///     pub min_value: NeoInteger,
/// }
/// ```
#[proc_macro_attribute]
pub fn neo_config(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand::neo_config(input).into()
}

/// Neo N3 Validation macro
///
/// This macro generates validation for a Neo N3 contract.
///
/// # Example
///
/// ```rust
/// #[neo_validate]
/// pub fn validate_value(value: NeoInteger) -> NeoResult<()> {
///     if value < NeoInteger::zero() {
///         return Err(NeoError::InvalidArgument);
///     }
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn neo_validate(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    expand::neo_validate(input).into()
}

/// Neo N3 Serialization macro
///
/// This macro generates serialization for a Neo N3 contract.
///
/// # Example
///
/// ```rust
/// #[neo_serialize]
/// pub struct MyData {
///     pub value: NeoInteger,
///     pub name: NeoString,
/// }
/// ```
#[proc_macro_attribute]
pub fn neo_serialize(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand::neo_serialize(input).into()
}

/// Neo N3 Error macro
///
/// This macro generates error handling for a Neo N3 contract.
///
/// # Example
///
/// ```rust
/// #[neo_error]
/// pub enum MyError {
///     InvalidValue,
///     InvalidName,
/// }
/// ```
#[proc_macro_attribute]
pub fn neo_error(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand::neo_error(input).into()
}
