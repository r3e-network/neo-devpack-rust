// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! NEF (Neo Executable Format) file generation
//!
//! This module handles the creation of NEF files, which are the standard format
//! for Neo N3 smart contract executables. NEF files contain:
//! - Magic number identifying the format version
//! - Compiler information
//! - Optional source URL
//! - Method tokens for external contract calls
//! - The compiled NeoVM script
//! - SHA256 double-hash checksum

use std::fs::File;
use std::io::Write;
use std::path::Path;

use anyhow::{ensure, Result};
use sha2::{Digest, Sha256};

/// NEF3 magic number (little-endian: "NEF3")
const NEF_MAGIC: u32 = 0x3346_454E;
const COMPILER: &str = concat!("neo-devpack-rust wasm-neovm ", env!("CARGO_PKG_VERSION"));
const MAX_SOURCE_LENGTH: usize = 256;
const MAX_METHOD_NAME_LENGTH: usize = 32;

const COMPILER_FIELD_SIZE: usize = 64;
pub(crate) const HASH160_LENGTH: usize = 20;
const CHECKSUM_LENGTH: usize = 4;
const METHOD_TOKEN_RESERVED_BYTES: usize = 2;
/// Maximum number of method tokens a NEF file may contain (Neo N3
/// `NefFile` deserializes via `ReadSerializableArray<MethodToken>(128)`).
pub const MAX_METHOD_TOKENS: usize = 128;
// Note: Reserved byte value is 0, inlined in code

/// Write a NEF artefact containing the provided script payload.
pub fn write_nef<P: AsRef<Path>>(script: &[u8], output_path: P) -> Result<()> {
    let bytes = encode_nef(script)?;
    let mut file = File::create(output_path)?;
    file.write_all(&bytes)?;
    Ok(())
}

/// Encode a NEF artefact containing the provided script payload.
pub fn encode_nef(script: &[u8]) -> Result<Vec<u8>> {
    encode_nef_with_metadata(script, None, &[])
}

/// Append a Neo `VarInt` length/count prefix. Delegates to the canonical
/// `core::encoding::encode_varint` so the NEF writer and the rest of the
/// crate share one definition of the wire format. (NEF is serialized once
/// per compilation, so the small per-call allocation is irrelevant.)
fn write_var_uint(buffer: &mut Vec<u8>, value: u64) {
    buffer.extend_from_slice(&crate::core::encoding::encode_varint(value));
}

fn write_var_bytes(buffer: &mut Vec<u8>, bytes: &[u8]) {
    write_var_uint(buffer, bytes.len() as u64);
    buffer.extend_from_slice(bytes);
}

fn write_var_string(buffer: &mut Vec<u8>, value: &str) -> Result<()> {
    let bytes = value.as_bytes();
    ensure!(
        bytes.len() <= MAX_SOURCE_LENGTH,
        "source string exceeds {MAX_SOURCE_LENGTH} bytes"
    );
    write_var_bytes(buffer, bytes);
    Ok(())
}

fn compute_checksum(bytes: &[u8]) -> [u8; CHECKSUM_LENGTH] {
    let hash = Sha256::digest(bytes);
    let hash = Sha256::digest(hash);
    let mut checksum = [0u8; CHECKSUM_LENGTH];
    checksum.copy_from_slice(&hash[..CHECKSUM_LENGTH]);
    checksum
}

/// Write a NEF file with metadata support
pub fn write_nef_with_metadata<P: AsRef<Path>>(
    script: &[u8],
    source_url: Option<&str>,
    method_tokens: &[MethodToken],
    output_path: P,
) -> Result<()> {
    let bytes = encode_nef_with_metadata(script, source_url, method_tokens)?;
    let mut file = File::create(output_path)?;
    file.write_all(&bytes)?;
    Ok(())
}

/// Encode a NEF payload with metadata support.
pub fn encode_nef_with_metadata(
    script: &[u8],
    source_url: Option<&str>,
    method_tokens: &[MethodToken],
) -> Result<Vec<u8>> {
    ensure!(!script.is_empty(), "script payload is empty");
    ensure!(
        COMPILER.len() <= 64,
        "compiler identifier longer than 64 bytes"
    );

    let mut buffer = Vec::new();
    buffer.extend_from_slice(&NEF_MAGIC.to_le_bytes());

    let compiler_bytes = COMPILER.as_bytes();
    let mut compiler_field = [0u8; COMPILER_FIELD_SIZE];
    compiler_field[..compiler_bytes.len()].copy_from_slice(compiler_bytes);
    buffer.extend_from_slice(&compiler_field);

    let source = source_url.unwrap_or("");
    write_var_string(&mut buffer, source)?;
    buffer.push(0); // reserved byte

    write_method_tokens(&mut buffer, method_tokens)?;

    write_var_bytes(&mut buffer, script);

    let checksum = compute_checksum(&buffer);
    buffer.extend_from_slice(&checksum[..CHECKSUM_LENGTH]);

    Ok(buffer)
}

/// Method token for NEF files
#[derive(Debug, Clone)]
pub struct MethodToken {
    /// 20-byte script hash of the target contract.
    pub contract_hash: [u8; 20],
    /// Name of the method to call.
    pub method: String,
    /// Number of parameters the method expects.
    pub parameters_count: u16,
    /// Whether the method returns a value.
    pub has_return_value: bool,
    /// Call flags controlling allowed operations.
    pub call_flags: u8,
}

// HASH160_LENGTH is defined above with other constants

/// Maximum valid value for call_flags (C# `CallFlags.All`, 4 bits:
/// ReadStates=1, WriteStates=2, AllowCall=4, AllowNotify=8)
const MAX_CALL_FLAGS: u8 = 0x0F;

fn write_method_tokens(buffer: &mut Vec<u8>, method_tokens: &[MethodToken]) -> Result<()> {
    ensure!(
        method_tokens.len() <= MAX_METHOD_TOKENS,
        "NEF method token count {} exceeds maximum of {} (Neo N3 NefFile limit)",
        method_tokens.len(),
        MAX_METHOD_TOKENS
    );
    write_var_uint(buffer, method_tokens.len() as u64);
    for token in method_tokens {
        ensure!(
            token.method.len() <= MAX_METHOD_NAME_LENGTH,
            "method token name '{}' exceeds {} bytes",
            token.method,
            MAX_METHOD_NAME_LENGTH
        );
        // Neo N3 `MethodToken` rejects method names beginning with '_' on
        // deserialize (`if (Method.StartsWith('_')) throw new FormatException`),
        // so a token that passes our writer would still be refused by every
        // consensus node at deploy time. Fail fast here instead.
        ensure!(
            !token.method.starts_with('_'),
            "method token name '{}' must not start with '_' (rejected by Neo N3 at deploy)",
            token.method
        );

        // Validate contract_hash is exactly 20 bytes (HASH160)
        ensure!(
            token.contract_hash.len() == HASH160_LENGTH,
            "method token '{}' has invalid contract_hash length: expected {}, got {}",
            token.method,
            HASH160_LENGTH,
            token.contract_hash.len()
        );

        // Validate call_flags is within valid range
        ensure!(
            token.call_flags <= MAX_CALL_FLAGS,
            "method token '{}' has invalid call_flags: {} (max {})",
            token.method,
            token.call_flags,
            MAX_CALL_FLAGS
        );

        // Note: parameters_count is u16, so it cannot exceed u16::MAX by definition
        // No validation needed here

        buffer.extend_from_slice(&token.contract_hash);
        write_var_string(buffer, &token.method)?;
        buffer.extend_from_slice(&token.parameters_count.to_le_bytes());
        buffer.push(if token.has_return_value { 1 } else { 0 });
        buffer.push(token.call_flags);
    }
    buffer.extend_from_slice(&[0u8; METHOD_TOKEN_RESERVED_BYTES]);
    Ok(())
}
