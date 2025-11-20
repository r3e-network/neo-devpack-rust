use std::fs::File;
use std::io::Write;
use std::path::Path;

use anyhow::{ensure, Result};
use sha2::{Digest, Sha256};

const NEF_MAGIC: u32 = 0x3346_454E; // "NEF3"
const COMPILER: &str = concat!("neo-llvm wasm-neovm ", env!("CARGO_PKG_VERSION"));
const MAX_SOURCE_LENGTH: usize = 256;
const MAX_METHOD_NAME_LENGTH: usize = 32;

/// Write a NEF artefact containing the provided script payload.
pub fn write_nef<P: AsRef<Path>>(script: &[u8], output_path: P) -> Result<()> {
    write_nef_with_metadata(script, None, &[], output_path)
}

fn write_var_uint(buffer: &mut Vec<u8>, value: u64) {
    if value < 0xFD {
        buffer.push(value as u8);
    } else if value <= 0xFFFF {
        buffer.push(0xFD);
        buffer.extend_from_slice(&(value as u16).to_le_bytes());
    } else if value <= 0xFFFF_FFFF {
        buffer.push(0xFE);
        buffer.extend_from_slice(&(value as u32).to_le_bytes());
    } else {
        buffer.push(0xFF);
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

fn compute_checksum(bytes: &[u8]) -> [u8; 4] {
    let hash = Sha256::digest(bytes);
    let hash = Sha256::digest(hash);
    let mut checksum = [0u8; 4];
    checksum.copy_from_slice(&hash[..4]);
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
        COMPILER.as_bytes().len() <= 64,
        "compiler identifier longer than 64 bytes"
    );

    let mut buffer = Vec::new();
    buffer.extend_from_slice(&NEF_MAGIC.to_le_bytes());

    let compiler_bytes = COMPILER.as_bytes();
    let mut compiler_field = [0u8; 64];
    compiler_field[..compiler_bytes.len()].copy_from_slice(compiler_bytes);
    buffer.extend_from_slice(&compiler_field);

    let source = source_url.unwrap_or("");
    write_var_string(&mut buffer, source)?;
    buffer.push(0); // reserved byte

    write_method_tokens(&mut buffer, method_tokens)?;

    write_var_bytes(&mut buffer, script);

    let checksum = compute_checksum(&buffer);
    buffer.extend_from_slice(&checksum);

    let mut file = File::create(output_path)?;
    file.write_all(&buffer)?;
    Ok(())
}

/// Method token for NEF files
#[derive(Debug, Clone)]
pub struct MethodToken {
    pub contract_hash: [u8; 20],
    pub method: String,
    pub parameters_count: u16,
    pub has_return_value: bool,
    pub call_flags: u8,
}

/// Length of a HASH160 value in bytes
pub const HASH160_LENGTH: usize = 20;

fn write_method_tokens(buffer: &mut Vec<u8>, method_tokens: &[MethodToken]) -> Result<()> {
    write_var_uint(buffer, method_tokens.len() as u64);
    for token in method_tokens {
        ensure!(
            token.method.as_bytes().len() <= MAX_METHOD_NAME_LENGTH,
            "method token name '{}' exceeds {} bytes",
            token.method,
            MAX_METHOD_NAME_LENGTH
        );
        buffer.extend_from_slice(&token.contract_hash);
        write_var_string(buffer, &token.method)?;
        buffer.extend_from_slice(&token.parameters_count.to_le_bytes());
        buffer.push(if token.has_return_value { 1 } else { 0 });
        buffer.push(token.call_flags);
    }
    buffer.extend_from_slice(&[0u8; 2]);
    Ok(())
}
