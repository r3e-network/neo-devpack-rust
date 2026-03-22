// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::ItemFn;

use crate::codegen;

pub(crate) fn neo_method(input: ItemFn) -> TokenStream2 {
    let vis = &input.vis;
    let sig = &input.sig;
    let block = &input.block;

    quote! {
        #vis #sig #block
    }
}

pub(crate) fn neo_entry(input: ItemFn) -> TokenStream2 {
    let entry_name = input.sig.ident.to_string();
    let kind = entry_name.as_str();

    let metadata = serde_json::json!({
        "entry": {
            "name": entry_name,
            "kind": kind,
        }
    });
    let overlay = codegen::manifest_overlay_tokens(&metadata.to_string());

    quote! {
        #input
        #overlay
    }
}

pub(crate) fn neo_test(input: ItemFn) -> TokenStream2 {
    let name = &input.sig.ident;
    let vis = &input.vis;
    let sig = &input.sig;
    let block = &input.block;
    let test_mod = format_ident!("__neo_test_{}", name);
    let test_fn = format_ident!("{}_case", name);

    quote! {
        #vis #sig #block

        #[cfg(test)]
        mod #test_mod {
            use super::*;

            #[test]
            fn #test_fn() {
                super::#name()
            }
        }
    }
}

pub(crate) fn neo_bench(input: ItemFn) -> TokenStream2 {
    let name = &input.sig.ident;
    let vis = &input.vis;
    let sig = &input.sig;
    let block = &input.block;
    let bench_mod = format_ident!("__neo_bench_{}", name);

    quote! {
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
    }
}
