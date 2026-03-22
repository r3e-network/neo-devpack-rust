// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

/// Map common env imports from Solana programs.
pub(super) fn map_env_import(name: &str) -> Option<&'static str> {
    match name {
        // Memory operations - handled by wasm-neovm runtime helpers
        "memcpy" | "__memcpy" => None,
        "memmove" | "__memmove" => None,
        "memset" | "__memset" => None,
        "memcmp" | "__memcmp" => None,

        // Panic/abort
        "abort" | "__rust_panic" | "rust_begin_unwind" => None, // Maps to ABORT opcode

        // Math functions (from libm)
        "__floatundidf" | "__floatundisf" => None, // Float conversion
        "__fixdfdi" | "__fixsfdi" => None,

        _ => None,
    }
}
