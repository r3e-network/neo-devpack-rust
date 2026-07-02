// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use serde_json::json;
use syn::{DeriveInput, Fields, ItemFn, LitStr};

use crate::codegen;

pub(crate) fn neo_event(input: DeriveInput) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let fields = match &input.data {
        syn::Data::Struct(struct_data) => match &struct_data.fields {
            Fields::Named(named) => named.named.iter().collect::<Vec<_>>(),
            Fields::Unnamed(_) | Fields::Unit => {
                return Err(syn::Error::new_spanned(
                    &input,
                    "#[neo_event] requires a struct with named fields",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                &input,
                "#[neo_event] can only be applied to structs",
            ));
        }
    };

    let push_fields = fields
        .iter()
        .map(|field| {
            let ident = field
                .ident
                .as_ref()
                .ok_or_else(|| syn::Error::new_spanned(field, "expected named field"))?;
            Ok(quote! {
                state.push(::neo_devpack::NeoValue::from(::core::clone::Clone::clone(&self.#ident)));
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;

    let event_name_string = name.to_string();
    let parameters: Vec<serde_json::Value> = fields
        .iter()
        .map(|field| {
            let field_name = field
                .ident
                .as_ref()
                .map(|i| i.to_string())
                .unwrap_or_else(|| "unnamed".to_string());
            let manifest_type = codegen::manifest_type_from_syn(&field.ty);
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
    let overlay = codegen::manifest_overlay_tokens(&metadata.to_string());

    Ok(quote! {
        #input

        impl #name {
            /// Emit this event with its field values as the notification
            /// state. One code path for host and wasm32: `NeoRuntime::notify`
            /// records into the host recorder on the host, and on wasm32 it
            /// serialises the state and crosses the `runtime_notify_with_state`
            /// import, which the wasm-neovm translator lowers to a real
            /// `System.Runtime.Notify` with the decoded state array — so
            /// on-chain notifications carry the same payload host tests see.
            /// (Previously the wasm32 path fell back to the name-only
            /// `notify_event`, silently dropping every field.)
            pub fn emit(&self) -> ::neo_devpack::NeoResult<()> {
                let event_name = stringify!(#name);
                let mut state = ::neo_devpack::NeoArray::new();
                #(#push_fields)*
                let label = ::neo_devpack::NeoString::from_str(event_name);
                ::neo_devpack::NeoRuntime::notify(&label, &state)
            }
        }

        #overlay
    })
}

pub(crate) fn neo_manifest_overlay(literal: &LitStr) -> TokenStream2 {
    let value = literal.value();

    if let Err(err) = serde_json::from_str::<serde_json::Value>(&value) {
        return syn::Error::new(literal.span(), format!("invalid JSON: {err}")).to_compile_error();
    }

    codegen::manifest_overlay_tokens(&value)
}

pub(crate) fn neo_permission(contract: String, methods: Vec<String>) -> TokenStream2 {
    let overlay = json!({
        "permissions": [
            {
                "contract": contract,
                "methods": methods,
            }
        ]
    });

    codegen::manifest_overlay_tokens(&overlay.to_string())
}

pub(crate) fn neo_trusts(trusts: Vec<String>) -> TokenStream2 {
    let overlay = json!({
        "trusts": trusts,
    });

    codegen::manifest_overlay_tokens(&overlay.to_string())
}

pub(crate) fn neo_safe_methods(methods: Vec<String>) -> TokenStream2 {
    let methods: Vec<serde_json::Value> = methods
        .into_iter()
        .map(|name| json!({"name": name, "safe": true}))
        .collect();

    let overlay = json!({
        "abi": { "methods": methods }
    });

    codegen::manifest_overlay_tokens(&overlay.to_string())
}

pub(crate) fn neo_safe(input: ItemFn) -> TokenStream2 {
    // Deprecated: the manifest-overlay `const` this used to emit is illegal
    // inside `impl` blocks — the only place contract methods live — so the
    // attribute can never work where users put it. Fail loudly instead of
    // silently producing broken output.
    let error = syn::Error::new_spanned(
        &input.sig.ident,
        "#[neo_safe] is deprecated and non-functional: it expands to a manifest overlay \
         `const` that is illegal inside `impl` blocks. Use #[neo_method(safe)] on the \
         method or the neo_safe_methods!([...]) overlay instead.",
    )
    .to_compile_error();

    // Re-emit the annotated function so the deprecation error does not
    // cascade into unrelated "cannot find function" errors.
    quote! {
        #input
        #error
    }
}

pub(crate) fn neo_supported_standards(standards: Vec<String>) -> TokenStream2 {
    let overlay = json!({
        "supportedstandards": standards,
    });

    codegen::manifest_overlay_tokens(&overlay.to_string())
}
