// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use anyhow::{bail, Result};

const DOC_HINT: &str = "Refer to docs/wasm-neovm-status.md for current coverage.";

/// Return an error for unsupported floating-point operations.
#[inline]
pub fn unsupported_float<T>(context: &str) -> Result<T> {
    bail!(
        "floating point operation '{}' is not supported ({}).",
        context,
        DOC_HINT
    )
}

/// Return an error for unsupported SIMD operations.
#[inline]
pub fn unsupported_simd<T>(context: &str) -> Result<T> {
    bail!(
        "SIMD operation '{}' is not supported ({}).",
        context,
        DOC_HINT
    )
}

/// Return an error for unsupported reference type operations.
#[inline]
pub fn unsupported_reference_type<T>(context: &str) -> Result<T> {
    bail!(
        "reference type '{}' is not supported ({}).",
        context,
        DOC_HINT
    )
}
