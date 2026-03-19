//! Build script for wasm-neovm
//!
//! This build script generates opcode and syscall tables from the Neo source code.
//! If the Neo source is not available, it falls back to bundled snapshots.

use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use regex::Regex;
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

const REQUIRE_NEO_CHECKOUT_ENV: &str = "WASM_NEOVM_REQUIRE_NEO_CHECKOUT";
const LEGACY_NEO_OPCODE_REL_PATH: &str = "neo/src/Neo.VM/OpCode.cs";
const SPLIT_NEO_VM_OPCODE_REL_PATH: &str = "neo-vm/src/Neo.VM/OpCode.cs";
const NEO_SYSCALL_REL_PATH: &str = "neo/src/Neo/SmartContract";

/// Main entry point for the build script.
///
/// This function:
/// 1. Locates the Neo source directory (if available)
/// 2. Generates opcode and syscall tables
/// 3. Sets up file watch triggers for incremental builds
fn main() -> Result<()> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let require_neo_checkout = env_flag_enabled(REQUIRE_NEO_CHECKOUT_ENV);
    let repo_root = manifest_dir
        .parent()
        .context("unable to locate repository root")?
        .to_path_buf();
    let fallback_dir = manifest_dir.join("src/generated");
    let opcode_path = resolve_opcode_path(&repo_root);
    let syscall_root = repo_root.join(NEO_SYSCALL_REL_PATH);
    let has_partial_reference_checkout =
        repo_root.join("neo").exists() || repo_root.join("neo-vm").exists();

    println!("cargo:rerun-if-env-changed={REQUIRE_NEO_CHECKOUT_ENV}");

    if let Some(opcode_path) = opcode_path.as_deref().filter(|_| syscall_root.exists()) {
        generate_opcodes(opcode_path)?;
        generate_syscalls(&syscall_root)?;

        println!("cargo:rerun-if-changed={}", opcode_path.display());
        for entry in WalkDir::new(&syscall_root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|ext| ext == "cs").unwrap_or(false))
        {
            println!("cargo:rerun-if-changed={}", entry.path().display());
        }
    } else {
        let missing_sources =
            missing_neo_sources(&repo_root, opcode_path.as_deref(), &syscall_root);
        if require_neo_checkout {
            bail!(
                "{REQUIRE_NEO_CHECKOUT_ENV}=1 but Neo source checkout is incomplete; missing: {}",
                missing_sources.join(", ")
            );
        }

        emit_fallback(&fallback_dir, "opcodes.rs")?;
        emit_fallback(&fallback_dir, "syscalls.rs")?;
        if has_partial_reference_checkout {
            let _ = missing_sources;
        } else {
            let _ = (
                &repo_root,
                NEO_SYSCALL_REL_PATH,
                SPLIT_NEO_VM_OPCODE_REL_PATH,
            );
        }
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

fn env_flag_enabled(name: &str) -> bool {
    env::var(name)
        .ok()
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn resolve_opcode_path(repo_root: &Path) -> Option<PathBuf> {
    let candidates = [
        repo_root.join(LEGACY_NEO_OPCODE_REL_PATH),
        repo_root.join(SPLIT_NEO_VM_OPCODE_REL_PATH),
    ];
    candidates.into_iter().find(|path| path.exists())
}

fn missing_neo_sources(
    repo_root: &Path,
    opcode_path: Option<&Path>,
    syscall_root: &Path,
) -> Vec<String> {
    let mut missing = Vec::new();
    if opcode_path.is_none() {
        missing.push(format!(
            "{} or {}",
            repo_root.join(LEGACY_NEO_OPCODE_REL_PATH).display(),
            repo_root.join(SPLIT_NEO_VM_OPCODE_REL_PATH).display()
        ));
    }
    if !syscall_root.exists() {
        missing.push(syscall_root.display().to_string());
    }
    missing
}

fn parse_u8_decimal(value: &str, field_name: &str) -> Result<u8> {
    let value = value.trim();
    let parsed = value
        .parse::<u16>()
        .with_context(|| format!("invalid {field_name} value '{value}'"))?;
    u8::try_from(parsed).with_context(|| format!("{field_name} value '{value}' exceeds u8 range"))
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
fn generate_opcodes(opcode_path: &Path) -> Result<()> {
    let contents = fs::read_to_string(opcode_path)
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
            let body = caps
                .name("body")
                .context("missing 'body' capture group in OperandSize attribute")?
                .as_str();
            for part in body.split(',') {
                let part = part.trim();
                if let Some(value) = part
                    .strip_prefix("SizePrefix")
                    .and_then(|suffix| suffix.split('=').nth(1))
                {
                    current_prefix = parse_u8_decimal(value, "opcode operand size prefix")?;
                } else if let Some(value) = part
                    .strip_prefix("Size")
                    .and_then(|suffix| suffix.split('=').nth(1))
                {
                    current_size = parse_u8_decimal(value, "opcode operand size")?;
                }
            }
            continue;
        }

        if let Some(caps) = entry_re.captures(line) {
            let name = caps
                .name("name")
                .context("missing 'name' capture group in opcode entry")?
                .as_str()
                .to_string();
            let value_str = caps
                .name("value")
                .context("missing 'value' capture group in opcode entry")?
                .as_str();
            let value = if let Some(hex) = value_str
                .strip_prefix("0x")
                .or(value_str.strip_prefix("0X"))
            {
                u8::from_str_radix(hex, 16)
                    .with_context(|| format!("invalid opcode hex value for {name}"))?
            } else {
                let parsed = value_str
                    .parse::<u16>()
                    .with_context(|| format!("invalid opcode value for {name}"))?;
                u8::try_from(parsed)
                    .with_context(|| format!("opcode value for {name} exceeds u8 range"))?
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
fn generate_syscalls(smart_contract_dir: &Path) -> Result<()> {
    let mut names = BTreeSet::new();
    let register_re = Regex::new(r#"Register\(\"([^\"]+)\""#)?;

    for entry in WalkDir::new(smart_contract_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|ext| ext == "cs").unwrap_or(false))
    {
        let contents = fs::read_to_string(entry.path())?;
        for cap in register_re.captures_iter(&contents) {
            let name = cap
                .get(1)
                .context("missing capture group in Register call")?
                .as_str()
                .to_string();
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
