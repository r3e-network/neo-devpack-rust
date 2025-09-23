//! Thin NeoVM Rust codegen facade used by integration tests.

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

const NEF_MAGIC: &[u8; 4] = b"NEF3";
const NEF_VERSION: u8 = 0x01;

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = if crc & 1 == 1 { 0xEDB8_8320 } else { 0 };
            crc = (crc >> 1) ^ mask;
        }
    }
    crc ^ 0xFFFF_FFFF
}

/// NeoVM codegen backend for Rust.
pub struct NeoVMCodegenBackend {
    #[allow(dead_code)]
    target_triple: String,
    syscall_registry: HashMap<String, u32>,
    #[allow(dead_code)]
    optimization_level: OptimizationLevel,
    #[allow(dead_code)]
    debug_info: bool,
}

/// Optimization levels supported by the shim backend.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptimizationLevel {
    None,
    Size,
    Speed,
}

/// High-level compilation options (mirrors the design docs shapes).
#[derive(Debug, Clone)]
pub struct CompilationOptions {
    pub optimization_level: OptimizationLevel,
    pub debug_info: bool,
    pub emit_manifest: bool,
    pub target_triple: String,
}

impl NeoVMCodegenBackend {
    pub fn new() -> Self {
        Self {
            target_triple: "neovm-unknown-neo3".to_string(),
            syscall_registry: HashMap::new(),
            optimization_level: OptimizationLevel::Size,
            debug_info: false,
        }
    }

    pub fn with_options(options: CompilationOptions) -> Self {
        Self {
            target_triple: options.target_triple,
            syscall_registry: HashMap::new(),
            optimization_level: options.optimization_level,
            debug_info: options.debug_info,
        }
    }

    /// Load syscall registry entries required by tests.
    pub fn initialize(&mut self) -> Result<(), String> {
        self.load_syscall_registry()?;
        Ok(())
    }

    /// Pull syscall hashes from the embedded design set.
    fn load_syscall_registry(&mut self) -> Result<(), String> {
        self.syscall_registry
            .insert("System.Runtime.GetTime".to_string(), 0x68B4_C4C1);
        self.syscall_registry
            .insert("System.Runtime.CheckWitness".to_string(), 0x0B5B_4B1A);
        self.syscall_registry
            .insert("System.Runtime.Notify".to_string(), 0x0F4B_4B1A);
        Ok(())
    }

    pub fn get_syscall_hash(&self, name: &str) -> Option<u32> {
        self.syscall_registry.get(name).copied()
    }

    /// Simulated end-to-end pipeline: read input, emit NEF to `output_path`.
    pub fn compile_to_neovm(&self, input_path: &str, output_path: &str) -> Result<(), String> {
        println!(
            "Compiling {} to NeoVM bytecode at {}",
            input_path, output_path
        );

        let bytecode = self.generate_bytecode(input_path)?;
        let manifest = self.generate_manifest(input_path)?;
        self.generate_nef(&bytecode, &manifest, output_path)
    }

    /// Encode the provided script + manifest pair into a NEF artefact.
    pub fn generate_nef(
        &self,
        bytecode: &[u8],
        manifest: &str,
        output_path: &str,
    ) -> Result<(), String> {
        if bytecode.is_empty() {
            return Err("Bytecode payload may not be empty".to_string());
        }

        let mut nef = Vec::new();
        nef.extend_from_slice(NEF_MAGIC);
        nef.push(NEF_VERSION);
        nef.extend_from_slice(&0u32.to_le_bytes()); // reserved field

        nef.extend_from_slice(&(bytecode.len() as u32).to_le_bytes());
        nef.extend_from_slice(bytecode);

        let manifest_bytes = manifest.as_bytes();
        nef.extend_from_slice(&(manifest_bytes.len() as u32).to_le_bytes());
        nef.extend_from_slice(manifest_bytes);

        let checksum = crc32(&nef);
        nef.extend_from_slice(&checksum.to_le_bytes());

        let mut file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {e}"))?;
        file.write_all(&nef)
            .map_err(|e| format!("Failed to write NEF file: {e}"))?;

        let metadata =
            fs::metadata(output_path).map_err(|e| format!("Failed to read file metadata: {e}"))?;
        if metadata.len() <= (NEF_MAGIC.len() as u64 + 1 + 4 + 4 + 4) {
            return Err("Generated NEF artefact is smaller than the NEF header".to_string());
        }

        Ok(())
    }

    /// Produce a tiny, spec-aligned NeoVM snippet.
    fn generate_bytecode(&self, _input_path: &str) -> Result<Vec<u8>, String> {
        // PUSH1 (const 1), PUSH2 (const 2), ADD, RET.
        Ok(vec![0x11, 0x12, 0x9E, 0x40])
    }

    /// Emit a minimal manifest shell for the generated contract.
    fn generate_manifest(&self, input_path: &str) -> Result<String, String> {
        let name = Path::new(input_path)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("GeneratedContract");

        Ok(format!(
            r#"{{
  "name": "{name}",
  "version": "1.0.0",
  "author": "neo-llvm",
  "description": "Contract generated by NeoVM LLVM backend",
  "abi": {{
    "hash": "0x00000000",
    "methods": [],
    "events": []
  }}
}}"#
        ))
    }
}

impl Default for NeoVMCodegenBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tmp_file(name: &str) -> String {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("{}/{}_{}.nef", std::env::temp_dir().display(), name, ts)
    }

    #[test]
    fn backend_defaults() {
        let backend = NeoVMCodegenBackend::new();
        assert_eq!(backend.target_triple, "neovm-unknown-neo3");
        assert_eq!(backend.optimization_level, OptimizationLevel::Size);
    }

    #[test]
    fn syscall_registry_contains_core_entries() {
        let mut backend = NeoVMCodegenBackend::new();
        backend.initialize().unwrap();
        assert_eq!(
            backend.get_syscall_hash("System.Runtime.GetTime"),
            Some(0x68B4_C4C1)
        );
        assert!(backend.get_syscall_hash("Unknown.Syscall").is_none());
    }

    #[test]
    fn generate_nef_writes_valid_header() {
        let backend = NeoVMCodegenBackend::new();
        let output = tmp_file("basic_nef");
        backend
            .generate_nef(&[0x11, 0x12, 0x9E, 0x40], "{\"name\":\"Test\"}", &output)
            .unwrap();

        let bytes = fs::read(&output).unwrap();
        assert!(bytes.starts_with(NEF_MAGIC));
        assert_eq!(bytes[4], NEF_VERSION);

        fs::remove_file(output).unwrap();
    }

    #[test]
    fn compile_pipeline_creates_nef_artifact() {
        let backend = NeoVMCodegenBackend::new();
        let output = tmp_file("compile_pipeline");
        backend
            .compile_to_neovm("examples/hello.rs", &output)
            .unwrap();
        assert!(Path::new(&output).exists());
        fs::remove_file(output).unwrap();
    }
}
