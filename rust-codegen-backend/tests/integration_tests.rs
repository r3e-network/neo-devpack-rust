//! Smoke tests exercising the public API surface of the shim backend.

use neovm_codegen_backend::{CompilationOptions, NeoVMCodegenBackend, OptimizationLevel};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

fn tmp_file(stem: &str) -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros();
    format!("{}/{}_{}.nef", std::env::temp_dir().display(), stem, ts)
}

#[test]
fn instantiates_with_custom_options() {
    let options = CompilationOptions {
        optimization_level: OptimizationLevel::Speed,
        debug_info: true,
        emit_manifest: true,
        target_triple: "neovm-unknown-neo3".to_string(),
    };

    let backend = NeoVMCodegenBackend::with_options(options.clone());
    assert_eq!(backend.get_syscall_hash("System.Runtime.GetTime"), None);

    assert_eq!(options.optimization_level, OptimizationLevel::Speed);
    assert!(options.debug_info);
    assert!(options.emit_manifest);
}

#[test]
fn generates_nef_from_bytecode() {
    let backend = NeoVMCodegenBackend::new();
    let out = tmp_file("integration_nef");
    backend
        .generate_nef(
            &[0x11, 0x12, 0x9E, 0x40],
            "{\"name\":\"Integration\"}",
            &out,
        )
        .unwrap();

    let data = fs::read(&out).unwrap();
    assert!(data.starts_with(b"NEF3"));
    assert_eq!(data[4], 0x01);
    fs::remove_file(out).unwrap();
}

#[test]
fn compile_pipeline_emits_file() {
    let backend = NeoVMCodegenBackend::new();
    let out = tmp_file("compile_end_to_end");
    backend.compile_to_neovm("hello_world.rs", &out).unwrap();
    assert!(Path::new(&out).exists());
    fs::remove_file(out).unwrap();
}
