//! Build script for wasm-neovm
//!
//! This build script generates opcode and syscall tables from the Neo source code.
//! If the Neo source is not available, it falls back to bundled snapshots.

use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use regex::Regex;
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

/// Main entry point for the build script.
///
/// This function:
/// 1. Locates the Neo source directory (if available)
/// 2. Generates opcode and syscall tables
/// 3. Sets up file watch triggers for incremental builds
fn main() -> Result<()> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let repo_root = manifest_dir
        .parent()
        .context("unable to locate repository root")?
        .to_path_buf();
    let neo_dir = repo_root.join("neo");
    let fallback_dir = manifest_dir.join("src/generated");
    let opcode_path = neo_dir.join("src/Neo.VM/OpCode.cs");
    let syscall_root = neo_dir.join("src/Neo/SmartContract");

    if opcode_path.exists() && syscall_root.exists() {
        generate_opcodes(&neo_dir)?;
        generate_syscalls(&neo_dir)?;

        println!("cargo:rerun-if-changed={}", opcode_path.display());
        for entry in WalkDir::new(&syscall_root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "cs").unwrap_or(false))
        {
            println!("cargo:rerun-if-changed={}", entry.path().display());
        }
    } else {
        emit_fallback(&fallback_dir, "opcodes.rs")?;
        emit_fallback(&fallback_dir, "syscalls.rs")?;
        println!("cargo:warning=neo checkout not found; using bundled opcode/syscall snapshot");
        println!(
            "cargo:rerun-if-changed={}",
            fallback_dir.join("opcodes.rs").display()
        );
        println!(
            "cargo:rerun-if-changed={}",
            fallback_dir.join("syscalls.rs").display()
        );
    }
    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}

/// Emits a fallback file from the bundled snapshots.
///
/// This is used when the Neo source code checkout is not available.
fn emit_fallback(fallback_dir: &Path, filename: &str) -> Result<()> {
    let fallback_path = fallback_dir.join(filename);
    let contents = fs::read_to_string(&fallback_path)
        .with_context(|| format!("failed to read {}", fallback_path.display()))?;
    let out_path = PathBuf::from(env::var("OUT_DIR")?).join(filename);
    fs::write(&out_path, contents)
        .with_context(|| format!("failed to write {}", out_path.display()))?;
    Ok(())
}

/// Generates the opcode table from Neo source code.
///
/// Parses OpCode.cs from the Neo VM source and extracts:
/// - Opcode names
/// - Byte values
/// - Operand sizes and prefixes
///
/// The generated table is written to $OUT_DIR/opcodes.rs
fn generate_opcodes(neo_dir: &Path) -> Result<()> {
    let opcode_path = neo_dir.join("src/Neo.VM/OpCode.cs");
    let contents = fs::read_to_string(&opcode_path)
        .with_context(|| format!("failed to read {}", opcode_path.display()))?;

    let attr_re = Regex::new(r"\[OperandSize\((?P<body>[^)]*)\)\]")?;
    let entry_re = Regex::new(r"^\s*(?P<name>[A-Za-z0-9_]+)\s*=\s*(?P<value>0x[0-9A-Fa-f]+|\d+)")?;

    #[derive(Debug)]
    struct OpcodeEntry {
        name: String,
        byte: u8,
        operand_size: u8,
        operand_size_prefix: u8,
    }

    let mut entries: Vec<OpcodeEntry> = Vec::new();
    let mut current_size: u8 = 0;
    let mut current_prefix: u8 = 0;

    for line in contents.lines() {
        if let Some(caps) = attr_re.captures(line) {
            current_size = 0;
            current_prefix = 0;
            let body = caps.name("body").unwrap().as_str();
            for part in body.split(',') {
                let part = part.trim();
                if part.starts_with("SizePrefix") {
                    if let Some(value) = part.split('=').nth(1) {
                        current_prefix = value.trim().parse::<u8>().unwrap_or(0);
                    }
                } else if part.starts_with("Size") {
                    if let Some(value) = part.split('=').nth(1) {
                        current_size = value.trim().parse::<u8>().unwrap_or(0);
                    }
                }
            }
            continue;
        }

        if let Some(caps) = entry_re.captures(line) {
            let name = caps.name("name").unwrap().as_str().to_string();
            let value_str = caps.name("value").unwrap().as_str();
            let value = if let Some(hex) = value_str
                .strip_prefix("0x")
                .or(value_str.strip_prefix("0X"))
            {
                u8::from_str_radix(hex, 16)
                    .with_context(|| format!("invalid opcode hex value for {name}"))?
            } else {
                value_str
                    .parse::<u16>()
                    .with_context(|| format!("invalid opcode value for {name}"))?
                    as u8
            };

            entries.push(OpcodeEntry {
                name,
                byte: value,
                operand_size: current_size,
                operand_size_prefix: current_prefix,
            });
            current_size = 0;
            current_prefix = 0;
        }
    }

    let out_path = PathBuf::from(env::var("OUT_DIR")?).join("opcodes.rs");
    let mut file = fs::File::create(&out_path)
        .with_context(|| format!("failed to create {}", out_path.display()))?;

    writeln!(
        file,
        "#[derive(Debug, Copy, Clone)]\npub struct OpcodeInfo {{\n    pub name: &'static str,\n    pub byte: u8,\n    pub operand_size: u8,\n    pub operand_size_prefix: u8,\n}}\n"
    )?;
    writeln!(file, "pub static OPCODES: &[OpcodeInfo] = &[")?;
    for entry in &entries {
        writeln!(
            file,
            "    OpcodeInfo {{ name: \"{}\", byte: 0x{:02X}, operand_size: {}, operand_size_prefix: {} }},",
            entry.name, entry.byte, entry.operand_size, entry.operand_size_prefix
        )?;
    }
    writeln!(file, "];")?;

    Ok(())
}

/// Generates the syscall table from Neo source code.
///
/// Walks through Neo/SmartContract directory and extracts all syscall names
/// from Register() calls. Computes SHA256 hashes for each syscall.
///
/// The generated table is written to $OUT_DIR/syscalls.rs
fn generate_syscalls(neo_dir: &Path) -> Result<()> {
    let mut names = BTreeSet::new();
    let register_re = Regex::new(r#"Register\(\"([^\"]+)\""#)?;

    let smart_contract_dir = neo_dir.join("src/Neo/SmartContract");
    for entry in WalkDir::new(&smart_contract_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "cs").unwrap_or(false))
    {
        let contents = fs::read_to_string(entry.path())?;
        for cap in register_re.captures_iter(&contents) {
            let name = cap.get(1).unwrap().as_str().to_string();
            names.insert(name);
        }
    }

    let mut infos = Vec::new();
    for name in names {
        let mut hasher = Sha256::new();
        hasher.update(name.as_bytes());
        let digest = hasher.finalize();
        let hash = u32::from_le_bytes([digest[0], digest[1], digest[2], digest[3]]);
        infos.push((name, hash));
    }

    let out_path = PathBuf::from(env::var("OUT_DIR")?).join("syscalls.rs");
    let mut file = fs::File::create(&out_path)
        .with_context(|| format!("failed to create {}", out_path.display()))?;

    writeln!(
        file,
        "#[derive(Debug, Copy, Clone)]\npub struct SyscallInfo {{\n    pub name: &'static str,\n    pub hash: u32,\n}}\n"
    )?;
    writeln!(file, "pub static SYSCALLS: &[SyscallInfo] = &[")?;
    for (name, hash) in &infos {
        writeln!(
            file,
            "    SyscallInfo {{ name: \"{}\", hash: 0x{:08X} }},",
            name, hash
        )?;
    }
    writeln!(file, "];")?;

    Ok(())
}
