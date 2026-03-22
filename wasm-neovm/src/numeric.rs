// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use anyhow::{bail, Result};

const DOC_HINT: &str = "Refer to docs/wasm-neovm-status.md for current coverage.";

// ============================================================================
// Integer Encoding Functions
// ============================================================================

/// Push a big integer value onto the stack
///
/// # Deprecated
///
/// This function uses **Neo Legacy** opcodes (PUSHM1=0x4F, PUSH0=0x00, PUSH1-16=0x50+n)
/// which are INCOMPATIBLE with Neo N3. The Neo N3 opcodes are PUSHM1=0x0F, PUSH0=0x10,
/// PUSH1-16=0x11-0x20. Use `translator::helpers::push::emit_push_int` instead, which
/// emits correct Neo N3 bytecode.
#[deprecated(
    note = "Uses Neo Legacy opcodes, not Neo N3. Use translator::helpers::push::emit_push_int instead."
)]
pub fn push_biginteger(script: &mut Vec<u8>, value: i64) {
    if value == -1 {
        script.push(0x4F); // PUSHM1
    } else if value == 0 {
        script.push(0x00); // PUSH0
    } else if value > 0 && value <= 16 {
        script.push(0x50 + (value as u8)); // PUSH1-PUSH16
    } else {
        let bytes = value.to_le_bytes();
        let mut significant_bytes = &bytes[..];
        while significant_bytes.len() > 1 && significant_bytes[significant_bytes.len() - 1] == 0 {
            significant_bytes = &significant_bytes[..significant_bytes.len() - 1];
        }
        script.push(significant_bytes.len() as u8);
        script.extend_from_slice(significant_bytes);
    }
}

/// Push a byte vector onto the stack
///
/// # Deprecated
///
/// This function uses Neo Legacy `PUSHBYTES1-75` encoding (raw length byte for data <= 75 bytes)
/// which is NOT valid in Neo N3. Neo N3 requires PUSHDATA1/2/4 for all byte data.
/// Use `translator::helpers::push::emit_push_data` instead, which emits correct Neo N3 bytecode.
#[deprecated(
    note = "Uses Neo Legacy PUSHBYTES encoding, not Neo N3. Use translator::helpers::push::emit_push_data instead."
)]
pub fn push_bytevec(script: &mut Vec<u8>, data: &[u8]) {
    let len = data.len();
    if len <= 75 {
        script.push(len as u8);
        script.extend_from_slice(data);
    } else if len <= 255 {
        script.push(0x0C); // PUSHDATA1
        script.push(len as u8);
        script.extend_from_slice(data);
    } else if len <= 65535 {
        script.push(0x0D); // PUSHDATA2
        script.extend_from_slice(&(len as u16).to_le_bytes());
        script.extend_from_slice(data);
    } else {
        script.push(0x0E); // PUSHDATA4
        script.extend_from_slice(&(len as u32).to_le_bytes());
        script.extend_from_slice(data);
    }
}

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
