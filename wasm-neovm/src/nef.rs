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

// NEF serialization constants
const VAR_UINT_16BIT_PREFIX: u8 = 0xFD;
const VAR_UINT_32BIT_PREFIX: u8 = 0xFE;
const VAR_UINT_64BIT_PREFIX: u8 = 0xFF;
const VAR_UINT_THRESHOLD_16BIT: u64 = 0xFD;
const VAR_UINT_THRESHOLD_32BIT: u64 = 0xFFFF;
const VAR_UINT_THRESHOLD_64BIT: u64 = 0xFFFF_FFFF;
const COMPILER_FIELD_SIZE: usize = 64;
pub(crate) const HASH160_LENGTH: usize = 20;
const CHECKSUM_LENGTH: usize = 4;
const METHOD_TOKEN_RESERVED_BYTES: usize = 2;
// Note: Reserved byte value is 0, inlined in code

/// Write a NEF artefact containing the provided script payload.
pub fn write_nef<P: AsRef<Path>>(script: &[u8], output_path: P) -> Result<()> {
    write_nef_with_metadata(script, None, &[], output_path)
}

fn write_var_uint(buffer: &mut Vec<u8>, value: u64) {
    if value < VAR_UINT_THRESHOLD_16BIT {
        buffer.push(value as u8);
    } else if value <= VAR_UINT_THRESHOLD_32BIT {
        buffer.push(VAR_UINT_16BIT_PREFIX);
        buffer.extend_from_slice(&(value as u16).to_le_bytes());
    } else if value <= VAR_UINT_THRESHOLD_64BIT {
        buffer.push(VAR_UINT_32BIT_PREFIX);
        buffer.extend_from_slice(&(value as u32).to_le_bytes());
    } else {
        buffer.push(VAR_UINT_64BIT_PREFIX);
        buffer.extend_from_slice(&value.to_le_bytes());
    }
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

    let mut file = File::create(output_path)?;
    file.write_all(&buffer)?;
    Ok(())
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

/// Maximum valid value for call_flags (4 bits: ReadStates=1, WriteStates=2, AllowCall=4, AllowModifyAccount=8)
const MAX_CALL_FLAGS: u8 = 0x0F;

fn write_method_tokens(buffer: &mut Vec<u8>, method_tokens: &[MethodToken]) -> Result<()> {
    write_var_uint(buffer, method_tokens.len() as u64);
    for token in method_tokens {
        ensure!(
            token.method.len() <= MAX_METHOD_NAME_LENGTH,
            "method token name '{}' exceeds {} bytes",
            token.method,
            MAX_METHOD_NAME_LENGTH
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
