// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use std::collections::HashSet;

use neo_syscalls::SYSCALLS as DEVPACK_SYSCALLS;
use wasm_neovm::{neo_syscalls as translator_aliases, syscalls as translator_syscalls};

fn camel_to_snake(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut prev_was_lower_or_digit = false;

    for ch in input.chars() {
        if ch.is_ascii_uppercase() {
            if prev_was_lower_or_digit {
                output.push('_');
            }
            output.push(ch.to_ascii_lowercase());
            prev_was_lower_or_digit = false;
        } else {
            output.push(ch.to_ascii_lowercase());
            prev_was_lower_or_digit = ch.is_ascii_lowercase() || ch.is_ascii_digit();
        }
    }

    output
}

fn canonical_alias(descriptor: &str) -> Option<String> {
    let mut parts = descriptor.split('.');
    let root = parts.next()?;
    let category = parts.next()?;
    let method = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    if !matches!(root, "System" | "Neo") {
        return None;
    }

    Some(format!(
        "{}_{}",
        category.to_ascii_lowercase(),
        camel_to_snake(method)
    ))
}

#[test]
fn devpack_syscalls_have_descriptor_and_hash_parity_with_translator() {
    let mut names = HashSet::new();

    for syscall in DEVPACK_SYSCALLS {
        assert!(
            names.insert(syscall.name),
            "duplicate devpack syscall descriptor '{}'",
            syscall.name
        );

        let translator_entry =
            translator_syscalls::lookup_extended(syscall.name).unwrap_or_else(|| {
                panic!(
                    "translator missing syscall descriptor '{}' from neo-syscalls",
                    syscall.name
                )
            });
        assert_eq!(
            translator_entry.hash, syscall.hash,
            "syscall hash mismatch for '{}'",
            syscall.name
        );

        let hash_lookup = translator_syscalls::lookup_by_hash(syscall.hash)
            .unwrap_or_else(|| panic!("translator missing hash 0x{:08x}", syscall.hash));
        assert_eq!(
            hash_lookup.name, syscall.name,
            "hash 0x{:08x} resolves to '{}' instead of '{}'",
            syscall.hash, hash_lookup.name, syscall.name
        );
    }
}

#[test]
fn devpack_syscalls_have_canonical_aliases_in_translator() {
    for syscall in DEVPACK_SYSCALLS {
        assert_eq!(
            translator_aliases::lookup_neo_syscall(syscall.name),
            Some(syscall.name),
            "translator lookup should resolve canonical descriptor '{}'",
            syscall.name
        );

        if let Some(alias) = canonical_alias(syscall.name) {
            assert_eq!(
                translator_aliases::lookup_neo_syscall(&alias),
                Some(syscall.name),
                "translator missing canonical alias '{}' for '{}'",
                alias,
                syscall.name
            );
        }
    }
}

#[test]
fn descriptor_and_alias_variants_resolve_consistently() {
    for syscall in DEVPACK_SYSCALLS {
        let lowercase = syscall.name.to_ascii_lowercase();
        assert_eq!(
            translator_aliases::lookup_neo_syscall(&lowercase),
            Some(syscall.name),
            "lowercase descriptor variant should resolve for '{}'",
            syscall.name
        );

        let slash_variant = syscall.name.replace('.', "/");
        assert_eq!(
            translator_aliases::lookup_neo_syscall(&slash_variant),
            Some(syscall.name),
            "slash descriptor variant should resolve for '{}'",
            syscall.name
        );

        if let Some(alias) = canonical_alias(syscall.name) {
            let dash_alias = alias.replace('_', "-");
            assert_eq!(
                translator_aliases::lookup_neo_syscall(&dash_alias),
                Some(syscall.name),
                "dash alias variant should resolve for '{}'",
                syscall.name
            );
        }
    }
}
