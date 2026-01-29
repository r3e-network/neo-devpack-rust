//! Round 60: Benchmark Tests - Meaningful Performance Measurements
//!
//! This module provides comprehensive benchmarks for the wasm-neovm translator.
//! Benchmarks measure real-world scenarios and provide actionable insights.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;

/// Benchmark group: Basic translation performance
fn bench_basic_translation(c: &mut Criterion) {
    let mut group = c.benchmark_group("basic_translation");
    group.measurement_time(Duration::from_secs(10));

    // Benchmark: Minimal module translation
    group.bench_function("minimal_module", |b| {
        let wasm = wat::parse_str("(module)").unwrap();
        b.iter(|| {
            let _ = wasm_neovm::translate_module(black_box(&wasm), black_box("Minimal"));
        });
    });

    // Benchmark: Single function translation
    group.bench_function("single_function", |b| {
        let wasm =
            wat::parse_str(r#"(module (func (export "test") (result i32) i32.const 42))"#).unwrap();
        b.iter(|| {
            let _ = wasm_neovm::translate_module(black_box(&wasm), black_box("Single"));
        });
    });

    // Benchmark: Simple arithmetic
    group.bench_function("simple_arithmetic", |b| {
        let wasm = wat::parse_str(
            r#"(module (func (export "add") (param i32 i32) (result i32)
                local.get 0 local.get 1 i32.add))"#,
        )
        .unwrap();
        b.iter(|| {
            let _ = wasm_neovm::translate_module(black_box(&wasm), black_box("Arithmetic"));
        });
    });

    group.finish();
}

/// Benchmark group: Scaling with code size
fn bench_code_size_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("code_size_scaling");
    group.measurement_time(Duration::from_secs(15));

    // Generate modules with increasing numbers of functions
    for num_funcs in [10, 50, 100, 500].iter() {
        let mut funcs = String::new();
        for i in 0..*num_funcs {
            funcs.push_str(&format!(
                r#"(func (export "func{i}") (result i32) i32.const {i})"#
            ));
        }
        let wat = format!("(module {})", funcs);
        let wasm = wat::parse_str(&wat).unwrap();

        group.throughput(Throughput::Elements(*num_funcs as u64));
        group.bench_with_input(BenchmarkId::from_parameter(num_funcs), &wasm, |b, wasm| {
            b.iter(|| {
                let _ = wasm_neovm::translate_module(black_box(wasm), black_box("Scaling"));
            });
        });
    }

    group.finish();
}

/// Benchmark group: Instruction complexity
fn bench_instruction_complexity(c: &mut Criterion) {
    let mut group = c.benchmark_group("instruction_complexity");
    group.measurement_time(Duration::from_secs(10));

    // Benchmark: Simple vs complex expressions
    let simple =
        wat::parse_str(r#"(module (func (export "simple") (result i32) i32.const 42))"#).unwrap();

    let complex = wat::parse_str(
        r#"(module (func (export "complex") (result i32)
            i32.const 1
            i32.const 2
            i32.add
            i32.const 3
            i32.mul
            i32.const 4
            i32.sub
            i32.const 5
            i32.div_s))"#,
    )
    .unwrap();

    group.bench_function("simple_expression", |b| {
        b.iter(|| {
            let _ = wasm_neovm::translate_module(black_box(&simple), black_box("Simple"));
        });
    });

    group.bench_function("complex_expression", |b| {
        b.iter(|| {
            let _ = wasm_neovm::translate_module(black_box(&complex), black_box("Complex"));
        });
    });

    // Benchmark: Control flow
    let control_flow = wat::parse_str(
        r#"(module (func (export "control") (param i32) (result i32)
            local.get 0
            if (result i32)
                local.get 0 i32.const 2 i32.mul
            else
                local.get 0 i32.const 3 i32.div_s
            end))"#,
    )
    .unwrap();

    group.bench_function("control_flow", |b| {
        b.iter(|| {
            let _ = wasm_neovm::translate_module(black_box(&control_flow), black_box("Control"));
        });
    });

    // Benchmark: Loop
    let loop_wasm = wat::parse_str(
        r#"(module (func (export "loop_test") (param i32) (result i32)
            (local $i i32) (local $sum i32)
            i32.const 0 local.set $sum
            i32.const 0 local.set $i
            loop $cont
                local.get $sum local.get $i i32.add local.set $sum
                local.get $i i32.const 1 i32.add local.set $i
                local.get $i local.get 0 i32.lt_s br_if $cont
            end
            local.get $sum))"#,
    )
    .unwrap();

    group.bench_function("loop_translation", |b| {
        b.iter(|| {
            let _ = wasm_neovm::translate_module(black_box(&loop_wasm), black_box("Loop"));
        });
    });

    group.finish();
}

/// Benchmark group: Memory operations
fn bench_memory_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_operations");
    group.measurement_time(Duration::from_secs(10));

    // Benchmark: Memory allocation
    let memory_alloc = wat::parse_str(r#"(module (memory 1) (func (export "test")))"#).unwrap();

    group.bench_function("memory_allocation", |b| {
        b.iter(|| {
            let _ = wasm_neovm::translate_module(black_box(&memory_alloc), black_box("MemAlloc"));
        });
    });

    // Benchmark: Memory load
    let memory_load = wat::parse_str(
        r#"(module
            (memory 1)
            (func (export "load") (param i32) (result i32)
                local.get 0 i32.load))"#,
    )
    .unwrap();

    group.bench_function("memory_load", |b| {
        b.iter(|| {
            let _ = wasm_neovm::translate_module(black_box(&memory_load), black_box("MemLoad"));
        });
    });

    // Benchmark: Memory store
    let memory_store = wat::parse_str(
        r#"(module
            (memory 1)
            (func (export "store") (param i32 i32)
                local.get 0 local.get 1 i32.store))"#,
    )
    .unwrap();

    group.bench_function("memory_store", |b| {
        b.iter(|| {
            let _ = wasm_neovm::translate_module(black_box(&memory_store), black_box("MemStore"));
        });
    });

    // Benchmark: Multiple memory operations
    let memory_complex = wat::parse_str(
        r#"(module
            (memory 1)
            (func (export "complex") (param i32) (result i32)
                local.get 0 i32.const 42 i32.store
                local.get 0 i32.const 4 i32.add i32.const 100 i32.store
                local.get 0 i32.load
                local.get 0 i32.const 4 i32.add i32.load
                i32.add))"#,
    )
    .unwrap();

    group.bench_function("memory_complex", |b| {
        b.iter(|| {
            let _ =
                wasm_neovm::translate_module(black_box(&memory_complex), black_box("MemComplex"));
        });
    });

    group.finish();
}

/// Benchmark group: Real-world contract patterns
fn bench_real_world_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_world_patterns");
    group.measurement_time(Duration::from_secs(15));

    // Token transfer pattern
    let token_transfer = wat::parse_str(
        r#"(module
            (memory 1)
            (global $total_supply (mut i64) (i64.const 1000000))
            (func (export "transfer") (param i32 i32 i64) (result i32)
                local.get 0 local.get 1 local.get 2
                i64.const 0 i64.gt_s
                if (result i32) i32.const 1 else i32.const 0 end)
            (func (export "balanceOf") (param i32) (result i64)
                i64.const 0)
            (func (export "totalSupply") (result i64)
                global.get $total_supply))"#,
    )
    .unwrap();

    group.bench_function("token_contract", |b| {
        b.iter(|| {
            let _ = wasm_neovm::translate_module(black_box(&token_transfer), black_box("Token"));
        });
    });

    // Counter contract pattern
    let counter = wat::parse_str(
        r#"(module
            (memory 1)
            (global $counter (mut i32) (i32.const 0))
            (func (export "get") (result i32)
                global.get $counter)
            (func (export "increment") (result i32)
                global.get $counter i32.const 1 i32.add
                global.set $counter
                global.get $counter)
            (func (export "decrement") (result i32)
                global.get $counter i32.const 1 i32.sub
                global.set $counter
                global.get $counter))"#,
    )
    .unwrap();

    group.bench_function("counter_contract", |b| {
        b.iter(|| {
            let _ = wasm_neovm::translate_module(black_box(&counter), black_box("Counter"));
        });
    });

    // Calculator contract pattern
    let calculator = wat::parse_str(
        r#"(module
            (func (export "add") (param i32 i32) (result i32)
                local.get 0 local.get 1 i32.add)
            (func (export "sub") (param i32 i32) (result i32)
                local.get 0 local.get 1 i32.sub)
            (func (export "mul") (param i32 i32) (result i32)
                local.get 0 local.get 1 i32.mul)
            (func (export "div") (param i32 i32) (result i32)
                local.get 0 local.get 1 i32.div_s))"#,
    )
    .unwrap();

    group.bench_function("calculator_contract", |b| {
        b.iter(|| {
            let _ = wasm_neovm::translate_module(black_box(&calculator), black_box("Calculator"));
        });
    });

    group.finish();
}

/// Benchmark group: Comparison operations
fn bench_comparison_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("comparison_operations");
    group.measurement_time(Duration::from_secs(10));

    // Benchmark: Integer comparisons (i32)
    let i32_comparisons = wat::parse_str(
        r#"(module (func (export "cmp") (param i32 i32) (result i32)
            local.get 0 local.get 1 i32.eq
            local.get 0 local.get 1 i32.ne
            local.get 0 local.get 1 i32.lt_s
            local.get 0 local.get 1 i32.gt_s
            local.get 0 local.get 1 i32.le_s
            local.get 0 local.get 1 i32.ge_s
            drop drop drop drop drop))"#,
    )
    .unwrap();

    group.bench_function("i32_comparisons", |b| {
        b.iter(|| {
            let _ = wasm_neovm::translate_module(black_box(&i32_comparisons), black_box("I32Cmp"));
        });
    });

    // Benchmark: Integer comparisons (i64)
    let i64_comparisons = wat::parse_str(
        r#"(module (func (export "cmp") (param i64 i64) (result i32)
            local.get 0 local.get 1 i64.eq
            local.get 0 local.get 1 i64.ne
            local.get 0 local.get 1 i64.lt_s
            local.get 0 local.get 1 i64.gt_s
            local.get 0 local.get 1 i64.le_s
            local.get 0 local.get 1 i64.ge_s
            drop drop drop drop drop))"#,
    )
    .unwrap();

    group.bench_function("i64_comparisons", |b| {
        b.iter(|| {
            let _ = wasm_neovm::translate_module(black_box(&i64_comparisons), black_box("I64Cmp"));
        });
    });

    group.finish();
}

/// Benchmark group: Bitwise operations
fn bench_bitwise_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("bitwise_operations");
    group.measurement_time(Duration::from_secs(10));

    // Benchmark: Bitwise operations
    let bitwise = wat::parse_str(
        r#"(module (func (export "bitops") (param i32 i32) (result i32)
            local.get 0 local.get 1 i32.and
            local.get 0 local.get 1 i32.or
            local.get 0 local.get 1 i32.xor
            local.get 0 i32.clz
            local.get 0 i32.ctz
            local.get 0 i32.popcnt
            drop drop drop drop drop
            i32.const 0))"#,
    )
    .unwrap();

    group.bench_function("i32_bitwise", |b| {
        b.iter(|| {
            let _ = wasm_neovm::translate_module(black_box(&bitwise), black_box("Bitwise"));
        });
    });

    // Benchmark: Shift operations
    let shifts = wat::parse_str(
        r#"(module (func (export "shifts") (param i32 i32) (result i32)
            local.get 0 local.get 1 i32.shl
            local.get 0 local.get 1 i32.shr_s
            local.get 0 local.get 1 i32.shr_u
            local.get 0 local.get 1 i32.rotl
            local.get 0 local.get 1 i32.rotr
            drop drop drop drop drop
            i32.const 0))"#,
    )
    .unwrap();

    group.bench_function("i32_shifts", |b| {
        b.iter(|| {
            let _ = wasm_neovm::translate_module(black_box(&shifts), black_box("Shifts"));
        });
    });

    group.finish();
}

/// Benchmark group: Manifest generation
fn bench_manifest_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("manifest_generation");
    group.measurement_time(Duration::from_secs(10));

    // Benchmark: Simple manifest
    let simple_manifest = wat::parse_str(r#"(module (func (export "main")))"#).unwrap();

    group.bench_function("simple_manifest", |b| {
        b.iter(|| {
            let result = wasm_neovm::translate_module(
                black_box(&simple_manifest),
                black_box("SimpleManifest"),
            );
            // Also measure JSON parsing
            if let Ok(translation) = result {
                let _: Result<serde_json::Value, _> = serde_json::from_str(&translation.manifest);
            }
        });
    });

    // Benchmark: Complex manifest
    let complex_manifest = wat::parse_str(
        r#"(module
            (func (export "method1") (param i32) (result i32))
            (func (export "method2") (param i64 i64) (result i64))
            (func (export "method3") (param i32 i32 i32))
            (func (export "method4") (result i32))
            (func (export "method5") (param i32) (result i64)))"#,
    )
    .unwrap();

    group.bench_function("complex_manifest", |b| {
        b.iter(|| {
            let result = wasm_neovm::translate_module(
                black_box(&complex_manifest),
                black_box("ComplexManifest"),
            );
            if let Ok(translation) = result {
                let _: Result<serde_json::Value, _> = serde_json::from_str(&translation.manifest);
            }
        });
    });

    group.finish();
}

/// Benchmark group: Error handling
fn bench_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");
    group.measurement_time(Duration::from_secs(5));

    // Benchmark: Invalid WASM handling
    let invalid = vec![0x00, 0x00, 0x00, 0x00];

    group.bench_function("invalid_wasm", |b| {
        b.iter(|| {
            let _ = wasm_neovm::translate_module(black_box(&invalid), black_box("Invalid"));
        });
    });

    // Benchmark: Truncated WASM handling
    let truncated = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];

    group.bench_function("truncated_wasm", |b| {
        b.iter(|| {
            let _ = wasm_neovm::translate_module(black_box(&truncated), black_box("Truncated"));
        });
    });

    group.finish();
}

// Register all benchmark groups
criterion_group!(
    benches,
    bench_basic_translation,
    bench_code_size_scaling,
    bench_instruction_complexity,
    bench_memory_operations,
    bench_real_world_patterns,
    bench_comparison_ops,
    bench_bitwise_ops,
    bench_manifest_generation,
    bench_error_handling
);

criterion_main!(benches);
