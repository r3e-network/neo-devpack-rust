//! Custom section generation helpers for Neo N3 manifest overlays.

use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Counter for generating unique manifest overlay identifiers.
static MANIFEST_OVERLAY_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Generate tokens for embedding a manifest overlay as a Wasm custom section.
///
/// This function creates a static byte array in the `.custom_section.neo.manifest`
/// section, which will be extracted during contract deployment.
///
/// # Arguments
///
/// * `value` - The JSON string to embed in the custom section
///
/// # Returns
///
/// TokenStream2 containing the custom section definition
pub fn manifest_overlay_tokens(value: &str) -> TokenStream2 {
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

/// Map a Syn type to a Neo N3 manifest type string.
///
/// This function converts Rust type paths to their corresponding
/// Neo N3 manifest type representations.
///
/// # Arguments
///
/// * `ty` - The Syn Type to convert
///
/// # Returns
///
/// A static string representing the manifest type
pub fn manifest_type_from_syn(ty: &syn::Type) -> &'static str {
    match ty {
        syn::Type::Path(type_path) => {
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
