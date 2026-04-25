// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, Fields, ItemFn};

pub(crate) fn neo_storage(input: DeriveInput) -> TokenStream2 {
    let name = &input.ident;
    let storage_key = format!("{}::storage", name);

    quote! {
        #input

        impl #name {
            /// Load storage with error handling.
            ///
            /// # Note on Name Collisions
            /// If your struct already has a `load_result` method, you may encounter
            /// compilation errors. In that case, manually implement the storage logic
            /// or rename your existing method.
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

            /// Load storage and propagate malformed-state errors.
            ///
            /// # Note on Name Collisions
            /// If your struct already has a `load` method, you may encounter
            /// compilation errors. Use `load_result` directly or rename your method.
            pub fn load(
                context: &::neo_devpack::NeoStorageContext,
            ) -> ::neo_devpack::NeoResult<Self>
            where
                Self: ::core::default::Default
                    + ::neo_devpack::serde::de::DeserializeOwned,
            {
                Self::load_result(context)
            }

            /// Load storage, returning default only when callers intentionally choose fallback.
            ///
            /// Prefer `load`/`load_result` for stateful contracts so corrupted or
            /// migration-incompatible state cannot be mistaken for a clean deployment.
            pub fn load_or_default(
                context: &::neo_devpack::NeoStorageContext,
            ) -> Self
            where
                Self: ::core::default::Default
                    + ::neo_devpack::serde::de::DeserializeOwned,
            {
                Self::load_result(context).unwrap_or_else(|_| Self::default())
            }

            /// Save to storage.
            ///
            /// # Note on Name Collisions
            /// If your struct already has a `save` method, you may encounter
            /// compilation errors. Implement storage manually or rename your method.
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
    }
}

pub(crate) fn neo_doc(input: DeriveInput) -> TokenStream2 {
    let name = &input.ident;

    quote! {
        #input

        impl #name {
            pub fn documentation() -> &'static str {
                "Neo N3 smart contract documentation"
            }
        }
    }
}

pub(crate) fn neo_config(input: DeriveInput) -> TokenStream2 {
    let name = &input.ident;

    let fields = match &input.data {
        syn::Data::Struct(struct_data) => match &struct_data.fields {
            Fields::Named(named) => named.named.iter().collect::<Vec<_>>(),
            _ => {
                return syn::Error::new_spanned(
                    &input,
                    "#[neo_config] requires a struct with named fields",
                )
                .to_compile_error();
            }
        },
        _ => {
            return syn::Error::new_spanned(&input, "#[neo_config] can only be applied to structs")
                .to_compile_error();
        }
    };

    let field_inits = match fields
        .iter()
        .map(|field| -> Result<TokenStream2, syn::Error> {
            let ident = field.ident.as_ref().ok_or_else(|| {
                syn::Error::new_spanned(
                    field,
                    "#[neo_config] macro only supports named struct fields",
                )
            })?;
            let field_name = ident.to_string();

            let init_expr = if field_name.contains("max") {
                quote! { ::neo_devpack::NeoInteger::max_i32() }
            } else if field_name.contains("min") {
                quote! { ::neo_devpack::NeoInteger::min_i32() }
            } else {
                quote! { ::core::default::Default::default() }
            };

            Ok(quote! { #ident: #init_expr })
        })
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(initializers) => initializers,
        Err(error) => return error.to_compile_error(),
    };

    quote! {
        #input

        impl ::core::default::Default for #name {
            fn default() -> Self {
                Self {
                    #(#field_inits),*
                }
            }
        }

        impl #name {
            /// Load configuration with default values.
            pub fn load() -> ::neo_devpack::NeoResult<Self> {
                Ok(Self::default())
            }

            /// Save configuration.
            pub fn save(&self) -> ::neo_devpack::NeoResult<()> {
                Ok(())
            }
        }
    }
}

pub(crate) fn neo_validate(input: ItemFn) -> TokenStream2 {
    quote! { #input }
}

pub(crate) fn neo_serialize(input: DeriveInput) -> TokenStream2 {
    let name = &input.ident;

    quote! {
        #input

        impl #name {
            pub fn serialize(&self) -> ::neo_devpack::NeoResult<::neo_devpack::NeoByteString> {
                let bytes = ::neo_devpack::codec::serialize(self)?;
                Ok(::neo_devpack::NeoByteString::new(bytes))
            }

            pub fn deserialize(
                data: &::neo_devpack::NeoByteString,
            ) -> ::neo_devpack::NeoResult<Self> {
                ::neo_devpack::codec::deserialize(data.as_slice())
            }
        }
    }
}

pub(crate) fn neo_error(input: DeriveInput) -> TokenStream2 {
    let name = &input.ident;
    let name_str = name.to_string();

    quote! {
        #input

        impl ::core::convert::From<#name> for ::neo_devpack::NeoError {
            fn from(err: #name) -> Self {
                ::neo_devpack::NeoError::Custom(
                    ::std::format!("{}: {:?}", #name_str, err)
                )
            }
        }

        impl #name {
            pub fn as_neo_error(&self) -> ::neo_devpack::NeoError {
                ::neo_devpack::NeoError::Custom(
                    ::std::format!("{}: {:?}", #name_str, self)
                )
            }
        }
    }
}
