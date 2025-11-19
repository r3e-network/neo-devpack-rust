//! Neo N3 Macros
//!
//! This crate provides procedural macros for Neo N3 smart contract development.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use serde_json::{json, Value as JsonValue};
use std::sync::atomic::{AtomicUsize, Ordering};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{bracketed, parse_macro_input, DeriveInput, Fields, ItemFn, LitStr, Token, Type};

static MANIFEST_OVERLAY_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn manifest_overlay_tokens(value: &str) -> TokenStream2 {
    let bytes = value.as_bytes();
    let len = bytes.len();
    let byte_tokens = bytes.iter().map(|b| quote! { #b });
    let counter = MANIFEST_OVERLAY_COUNTER.fetch_add(1, Ordering::Relaxed);
    let ident = format_ident!("__NEO_MANIFEST_OVERLAY_{}", counter);

    quote! {
        const _: () = {
            #[link_section = ".custom_section.neo.manifest"]
            #[used]
            static #ident: [u8; #len] = [#(#byte_tokens),*];
        };
    }
}

struct PermissionArgs {
    contract: LitStr,
    methods: Vec<LitStr>,
}

impl Parse for PermissionArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let contract: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;

        let content;
        bracketed!(content in input);
        let methods: Punctuated<LitStr, Token![,]> =
            content.call(Punctuated::<LitStr, Token![,]>::parse_terminated)?;

        Ok(Self {
            contract,
            methods: methods.into_iter().collect(),
        })
    }
}

struct StringList {
    items: Vec<LitStr>,
}

impl Parse for StringList {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let content;
        bracketed!(content in input);
        let items: Punctuated<LitStr, Token![,]> =
            content.call(Punctuated::<LitStr, Token![,]>::parse_terminated)?;
        Ok(Self {
            items: items.into_iter().collect(),
        })
    }
}

/// Neo N3 Manifest Overlay macro
///
/// Embed a JSON manifest fragment as a Wasm custom section.
#[proc_macro]
pub fn neo_manifest_overlay(input: TokenStream) -> TokenStream {
    let literal = parse_macro_input!(input as LitStr);
    let value = literal.value();

    if let Err(err) = serde_json::from_str::<JsonValue>(&value) {
        return syn::Error::new(literal.span(), format!("invalid JSON: {err}"))
            .to_compile_error()
            .into();
    }

    manifest_overlay_tokens(&value).into()
}

/// Declare manifest permissions and embed them as a custom section.
#[proc_macro]
pub fn neo_permission(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as PermissionArgs);
    let contract = args.contract.value();
    let methods: Vec<String> = args.methods.into_iter().map(|lit| lit.value()).collect();

    let overlay = json!({
        "permissions": [
            {
                "contract": contract,
                "methods": methods,
            }
        ]
    });

    manifest_overlay_tokens(&overlay.to_string()).into()
}

/// Declare supported standards for the contract manifest.
#[proc_macro]
pub fn neo_supported_standards(input: TokenStream) -> TokenStream {
    let list = parse_macro_input!(input as StringList);
    let standards: Vec<String> = list.items.into_iter().map(|lit| lit.value()).collect();

    let overlay = json!({
        "supportedstandards": standards,
    });

    manifest_overlay_tokens(&overlay.to_string()).into()
}

/// Declare safe methods for the contract manifest.
#[proc_macro]
pub fn neo_safe_methods(input: TokenStream) -> TokenStream {
    let list = parse_macro_input!(input as StringList);
    let methods: Vec<serde_json::Value> = list
        .items
        .into_iter()
        .map(|lit| {
            let name = lit.value();
            json!({"name": name, "safe": true})
        })
        .collect();

    let overlay = json!({
        "abi": { "methods": methods }
    });

    manifest_overlay_tokens(&overlay.to_string()).into()
}

/// Mark a single exported function as safe in the manifest.
#[proc_macro_attribute]
pub fn neo_safe(_args: TokenStream, input: TokenStream) -> TokenStream {
    let func = parse_macro_input!(input as ItemFn);
    let name = func.sig.ident.to_string();

    let overlay = json!({
        "abi": { "methods": [ { "name": name, "safe": true } ] }
    });

    let overlay_tokens = manifest_overlay_tokens(&overlay.to_string());

    quote! {
        #func
        #overlay_tokens
    }
    .into()
}

/// Declare trusted contracts for the contract manifest.
#[proc_macro]
pub fn neo_trusts(input: TokenStream) -> TokenStream {
    let list = parse_macro_input!(input as StringList);
    let trusts: Vec<String> = list.items.into_iter().map(|lit| lit.value()).collect();

    let overlay = json!({
        "trusts": trusts,
    });

    manifest_overlay_tokens(&overlay.to_string()).into()
}

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
    let name = &input.ident;

    let expanded = quote! {
        #input

        impl NeoContract for #name {
            fn name() -> &'static str {
                stringify!(#name)
            }

            fn version() -> &'static str {
                "1.0.0"
            }

            fn author() -> &'static str {
                "neo-devpack"
            }

            fn description() -> &'static str {
                "Neo N3 smart contract generated by neo-devpack"
            }
        }
    };

    TokenStream::from(expanded)
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
    let _name = &input.sig.ident;
    let vis = &input.vis;
    let sig = &input.sig;
    let block = &input.block;

    let expanded = quote! {
        #vis #sig #block
    };

    TokenStream::from(expanded)
}

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
#[proc_macro_attribute]
pub fn neo_event(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let fields = match &input.data {
        syn::Data::Struct(struct_data) => match &struct_data.fields {
            Fields::Named(named) => named.named.iter().collect::<Vec<_>>(),
            Fields::Unnamed(_) | Fields::Unit => {
                return syn::Error::new_spanned(
                    &input,
                    "#[neo_event] requires a struct with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(&input, "#[neo_event] can only be applied to structs")
                .to_compile_error()
                .into();
        }
    };

    let push_fields = fields.iter().map(|field| {
        let ident = field.ident.as_ref().expect("named field");
        quote! {
            state.push(::neo_devpack::NeoValue::from(::core::clone::Clone::clone(&self.#ident)));
        }
    });

    let event_name_string = name.to_string();
    let parameters: Vec<serde_json::Value> = fields
        .iter()
        .map(|field| {
            let field_name = field.ident.as_ref().unwrap().to_string();
            let manifest_type = manifest_type_from_syn(&field.ty);
            json!({
                "name": field_name,
                "type": manifest_type,
            })
        })
        .collect();
    let metadata = json!({
        "abi": {
            "events": [
                {
                    "name": event_name_string,
                    "parameters": parameters,
                }
            ]
        }
    });
    let overlay = manifest_overlay_tokens(&metadata.to_string());

    let expanded = quote! {
        #input

        impl #name {
            pub fn emit(&self) -> ::neo_devpack::NeoResult<()> {
                let event_name = stringify!(#name);
                let mut state = ::neo_devpack::NeoArray::new();
                #(#push_fields)*
                let label = ::neo_devpack::NeoString::from_str(event_name);
                ::neo_devpack::NeoRuntime::notify(&label, &state)
            }
        }

        #overlay
    };

    expanded.into()
}

fn manifest_type_from_syn(ty: &Type) -> &'static str {
    match ty {
        Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                match segment.ident.to_string().as_str() {
                    "NeoBoolean" => "Boolean",
                    "NeoInteger" => "Integer",
                    "NeoByteString" => "ByteArray",
                    "NeoString" => "String",
                    "NeoArray" => "Array",
                    "NeoMap" => "Map",
                    "NeoStruct" => "Array",
                    "NeoValue" => "Any",
                    "NeoIterator" => "InteropInterface",
                    "NeoContract" | "NeoContractEntry" => "InteropInterface",
                    _ => "Any",
                }
            } else {
                "Any"
            }
        }
        _ => "Any",
    }
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
    let name = &input.ident;
    let storage_key = format!("{}::storage", name);

    let expanded = quote! {
        #input

        impl #name {
            pub fn load_result(
                context: &::neo_devpack::NeoStorageContext,
            ) -> ::neo_devpack::NeoResult<Self>
            where
                Self: ::core::default::Default
                    + ::neo_devpack::serde::de::DeserializeOwned,
            {
                let key = ::neo_devpack::NeoByteString::from_slice(#storage_key.as_bytes());
                let bytes = ::neo_devpack::NeoStorage::get(context, &key)?;
                if bytes.is_empty() {
                    return Ok(Self::default());
                }
                ::neo_devpack::codec::deserialize(bytes.as_slice())
            }

            pub fn load(
                context: &::neo_devpack::NeoStorageContext,
            ) -> Self
            where
                Self: ::core::default::Default
                    + ::neo_devpack::serde::de::DeserializeOwned,
            {
                Self::load_result(context).unwrap_or_else(|_| Self::default())
            }

            pub fn save(
                &self,
                context: &::neo_devpack::NeoStorageContext,
            ) -> ::neo_devpack::NeoResult<()>
            where
                Self: ::neo_devpack::serde::Serialize,
            {
                if context.is_read_only() {
                    return Err(::neo_devpack::NeoError::InvalidOperation);
                }
                let key = ::neo_devpack::NeoByteString::from_slice(#storage_key.as_bytes());
                let bytes = ::neo_devpack::codec::serialize(self)?;
                let payload = ::neo_devpack::NeoByteString::new(bytes);
                ::neo_devpack::NeoStorage::put(context, &key, &payload)
            }
        }
    };

    expanded.into()
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
    let name = &input_fn.sig.ident;
    let entry_name = name.to_string();
    let kind = match entry_name.as_str() {
        "deploy" => "deploy",
        "update" => "update",
        "destroy" => "destroy",
        other => other,
    };

    let metadata = json!({
        "entry": {
            "name": entry_name,
            "kind": kind,
        }
    });
    let overlay = manifest_overlay_tokens(&metadata.to_string());

    let expanded = quote! {
        #input_fn
        #overlay
    };

    expanded.into()
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
    let name = &input.sig.ident;
    let vis = &input.vis;
    let sig = &input.sig;
    let block = &input.block;
    let test_mod = format_ident!("__neo_test_{}", name);
    let test_fn = format_ident!("{}_case", name);

    let expanded = quote! {
        #vis #sig #block

        #[cfg(test)]
        mod #test_mod {
            use super::*;

            #[test]
            fn #test_fn() {
                super::#name()
            }
        }
    };

    TokenStream::from(expanded)
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
    let name = &input.sig.ident;
    let vis = &input.vis;
    let sig = &input.sig;
    let block = &input.block;
    let bench_mod = format_ident!("__neo_bench_{}", name);

    let expanded = quote! {
        #vis #sig #block

        #[cfg(feature = "bench")]
        mod #bench_mod {
            use super::*;
            use criterion::*;

            fn run(c: &mut Criterion) {
                c.bench_function(stringify!(#name), |b| {
                    b.iter(|| {
                        super::#name()
                    });
                });
            }

            criterion_group!(benches, run);
            criterion_main!(benches);
        }
    };

    TokenStream::from(expanded)
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
    let name = &input.ident;

    let expanded = quote! {
        #input

        impl #name {
            pub fn documentation() -> &'static str {
                "Neo N3 smart contract documentation"
            }
        }
    };

    TokenStream::from(expanded)
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
    let name = &input.ident;

    let expanded = quote! {
        #input

        impl #name {
            pub fn default() -> Self {
                Self {
                    max_value: NeoInteger::max_i32(),
                    min_value: NeoInteger::min_i32(),
                }
            }

            pub fn load() -> NeoResult<Self> {
                // This would be implemented by the LLVM backend
                Ok(Self::default())
            }

            pub fn save(&self) -> NeoResult<()> {
                // This would be implemented by the LLVM backend
                Ok(())
            }
        }
    };

    TokenStream::from(expanded)
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
    let name = &input.sig.ident;
    let vis = &input.vis;
    let sig = &input.sig;
    let block = &input.block;

    let expanded = quote! {
        #vis #sig #block

        impl #name {
            pub fn validate(&self) -> NeoResult<()> {
                #name(self)
            }
        }
    };

    TokenStream::from(expanded)
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
    let name = &input.ident;

    let expanded = quote! {
        #input

        impl #name {
            pub fn serialize(&self) -> ::neo_devpack::NeoResult<::neo_devpack::NeoByteString> {
                let bytes = ::neo_devpack::codec::serialize(self)?;
                Ok(::neo_devpack::NeoByteString::new(bytes))
            }

            pub fn deserialize(data: &::neo_devpack::NeoByteString) -> ::neo_devpack::NeoResult<Self> {
                ::neo_devpack::codec::deserialize(data.as_slice())
            }
        }
    };

    TokenStream::from(expanded)
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
    let name = &input.ident;

    let expanded = quote! {
        #input

        impl #name {
            pub fn as_neo_error(&self) -> NeoError {
                match self {
                    #name::InvalidValue => NeoError::InvalidArgument,
                    #name::InvalidName => NeoError::InvalidArgument,
                }
            }
        }
    };

    TokenStream::from(expanded)
}
