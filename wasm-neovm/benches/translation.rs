// Copyright (c) 2025-2026 R3E Network
// Licensed under the MIT License

//! Comprehensive benchmarks for wasm-neovm translation
//!
//! Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use wasm_neovm::{translate_module, translate_with_config, SourceChain, TranslationConfig};

// ============================================================================
// Test WASM Modules
// ============================================================================

/// Minimal WASM module (just magic + version)
const MINIMAL_WASM: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, // magic: \0asm
    0x01, 0x00, 0x00, 0x00, // version: 1
];

/// Simple WASM module with basic arithmetic
const SIMPLE_WASM: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, // magic + version
    0x01, 0x07, 0x01, // type section
    0x60, 0x02, 0x7f, 0x7f, 0x01, 0x7f, // (i32, i32) -> i32
    0x03, 0x02, 0x01, 0x00, // function section
    0x07, 0x07, 0x01, 0x03, 0x61, 0x64, 0x64, 0x00, 0x00, // export "add"
    0x0a, 0x09, 0x01, // code section
    0x07, 0x00, 0x20, 0x00, 0x20, 0x01, 0x6a, 0x0b, // body
];

/// WASM module with control flow
const CONTROL_FLOW_WASM: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, // magic + version
    0x01, 0x06, 0x01, // type section
    0x60, 0x01, 0x7f, 0x01, 0x7f, // (i32) -> i32
    0x03, 0x02, 0x01, 0x00, // function section
    0x07, 0x08, 0x01, 0x04, 0x74, 0x65, 0x73, 0x74, 0x00, 0x00, // export
    0x0a, 0x11, 0x01, // code section
    // if-else with comparison
    0x0f, 0x00, 0x20, 0x00, 0x41, 0x0a, 0x4a, 0x04, 0x7f, 0x41, 0x01, 0x05, 0x41, 0x00, 0x0b, 0x0b,
];

/// WASM module with memory operations
const MEMORY_WASM: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, // magic + version
    0x01, 0x05, 0x01, // type section
    0x60, 0x01, 0x7f, 0x01, 0x7f, // (i32) -> i32
    0x03, 0x02, 0x01, 0x00, // function section
    0x05, 0x03, 0x01, 0x00, 0x01, // memory section (1 page)
    0x07, 0x08, 0x01, 0x04, 0x6c, 0x6f, 0x61, 0x64, 0x00, 0x00, // export
    0x0a, 0x0a, 0x01, // code section
    0x08, 0x00, 0x20, 0x00, 0x28, 0x02, 0x00, 0x0b, // load from memory
];

/// Generate a WASM module with n functions
fn generate_wasm_with_functions(n: usize) -> Vec<u8> {
    use wasm_encoder::*;

    let mut module = Module::new();

    // Type section
    let mut types = TypeSection::new();
    let func_type = FuncType::new(vec![ValType::I32], vec![ValType::I32]);
    types.ty().func_type(&func_type);
    module.section(&types);

    // Function section
    let mut funcs = FunctionSection::new();
    for _ in 0..n {
        funcs.function(0);
    }
    module.section(&funcs);

    // Export section
    let mut exports = ExportSection::new();
    for i in 0..n {
        exports.export(&format!("func_{}", i), ExportKind::Func, i as u32);
    }
    module.section(&exports);

    // Code section
    let mut codes = CodeSection::new();
    for _ in 0..n {
        let mut func = Function::new(vec![]);
        func.instruction(&Instruction::LocalGet(0));
        func.instruction(&Instruction::I32Const(1));
        func.instruction(&Instruction::I32Add);
        func.instruction(&Instruction::End);
        codes.function(&func);
    }
    module.section(&codes);

    module.finish()
}

/// Generate a WASM module with n locals
fn generate_wasm_with_locals(n: usize) -> Vec<u8> {
    use wasm_encoder::*;

    let mut module = Module::new();

    // Type section
    let mut types = TypeSection::new();
    let func_type = FuncType::new(vec![], vec![ValType::I32]);
    types.ty().func_type(&func_type);
    module.section(&types);

    // Function section
    let mut funcs = FunctionSection::new();
    funcs.function(0);
    module.section(&funcs);

    // Export section
    let mut exports = ExportSection::new();
    exports.export("test", ExportKind::Func, 0);
    module.section(&exports);

    // Code section
    let mut codes = CodeSection::new();
    let mut func = Function::new((0..n).map(|_| (1, ValType::I32)).collect::<Vec<_>>());
    func.instruction(&Instruction::I32Const(n as i32));
    func.instruction(&Instruction::End);
    codes.function(&func);
    module.section(&codes);

    module.finish()
}

// ============================================================================
// Benchmark Functions
// ============================================================================

fn bench_minimal_translation(c: &mut Criterion) {
    c.bench_function("translate/minimal", |b| {
        b.iter(|| {
            let _ = translate_module(black_box(MINIMAL_WASM), black_box("minimal"));
        })
    });
}

fn bench_simple_translation(c: &mut Criterion) {
    c.bench_function("translate/simple", |b| {
        b.iter(|| {
            let _ = translate_module(black_box(SIMPLE_WASM), black_box("simple"));
        })
    });
}

fn bench_control_flow_translation(c: &mut Criterion) {
    c.bench_function("translate/control_flow", |b| {
        b.iter(|| {
            let _ = translate_module(black_box(CONTROL_FLOW_WASM), black_box("control_flow"));
        })
    });
}

fn bench_memory_translation(c: &mut Criterion) {
    c.bench_function("translate/memory", |b| {
        b.iter(|| {
            let _ = translate_module(black_box(MEMORY_WASM), black_box("memory"));
        })
    });
}

fn bench_scaling_functions(c: &mut Criterion) {
    let mut group = c.benchmark_group("translate/scaling/functions");

    for count in [1, 10, 50, 100] {
        let wasm = generate_wasm_with_functions(count);
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), &wasm, |b, wasm| {
            b.iter(|| {
                let _ = translate_module(black_box(wasm), black_box("scaling"));
            })
        });
    }

    group.finish();
}

fn bench_scaling_locals(c: &mut Criterion) {
    let mut group = c.benchmark_group("translate/scaling/locals");

    for count in [1, 10, 50, 100] {
        let wasm = generate_wasm_with_locals(count);
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), &wasm, |b, wasm| {
            b.iter(|| {
                let _ = translate_module(black_box(wasm), black_box("scaling"));
            })
        });
    }

    group.finish();
}

fn bench_source_chains(c: &mut Criterion) {
    let mut group = c.benchmark_group("translate/source_chain");

    for chain in [SourceChain::Neo, SourceChain::Solana, SourceChain::Move] {
        group.bench_with_input(
            BenchmarkId::new("chain", format!("{:?}", chain)),
            &chain,
            |b, chain| {
                let config = TranslationConfig::new("bench").with_source_chain(*chain);
                b.iter(|| {
                    let _ = translate_with_config(black_box(SIMPLE_WASM), config.clone());
                })
            },
        );
    }

    group.finish();
}

fn bench_repeated_translation(c: &mut Criterion) {
    let mut group = c.benchmark_group("translate/repeated");

    for count in [1, 10, 100] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            b.iter(|| {
                for _ in 0..count {
                    let _ = translate_module(black_box(SIMPLE_WASM), black_box("repeat"));
                }
            })
        });
    }

    group.finish();
}

fn bench_wasm_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("translate/size");

    let sizes = [
        ("minimal", MINIMAL_WASM.to_vec()),
        ("simple", SIMPLE_WASM.to_vec()),
        ("control_flow", CONTROL_FLOW_WASM.to_vec()),
        ("memory", MEMORY_WASM.to_vec()),
        ("10_funcs", generate_wasm_with_functions(10)),
        ("50_funcs", generate_wasm_with_functions(50)),
    ];

    for (name, wasm) in sizes {
        group.throughput(Throughput::Bytes(wasm.len() as u64));
        group.bench_with_input(BenchmarkId::new("size", name), &wasm, |b, wasm| {
            b.iter(|| {
                let _ = translate_module(black_box(wasm), black_box("size_test"));
            })
        });
    }

    group.finish();
}

// ============================================================================
// Criterion Groups
// ============================================================================

criterion_group!(
    basic,
    bench_minimal_translation,
    bench_simple_translation,
    bench_control_flow_translation,
    bench_memory_translation
);

criterion_group!(
    scaling,
    bench_scaling_functions,
    bench_scaling_locals,
    bench_repeated_translation
);

criterion_group!(variants, bench_source_chains, bench_wasm_sizes);

criterion_main!(basic, scaling, variants);
