use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use move_neovm::{parse_move_bytecode, translate_to_wasm, MoveModule};

/// Minimal Move bytecode module for benchmarking
/// This represents a simple Move module with basic operations
fn create_simple_move_module() -> Vec<u8> {
    // Simplified Move bytecode structure
    // Magic number + version + minimal module structure
    vec![
        0xA1, 0x1C, 0xEB, 0x0B, // Magic number
        0x01, 0x00, 0x00, 0x00, // Version
        // Minimal module structure (simplified)
        0x00, 0x00, 0x00, 0x00,
    ]
}

/// Move module with function definitions
fn create_module_with_functions() -> Vec<u8> {
    let mut module = create_simple_move_module();
    // Add function table entries (simplified)
    module.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]); // 1 function
    module
}

fn bench_parse_move_bytecode(c: &mut Criterion) {
    let bytecode = create_simple_move_module();

    c.bench_function("parse_simple_move_module", |b| {
        b.iter(|| {
            // Note: This may fail with simplified bytecode, adjust based on actual parser
            let _ = parse_move_bytecode(black_box(&bytecode));
        })
    });
}

fn bench_translate_to_wasm(c: &mut Criterion) {
    let bytecode = create_simple_move_module();

    c.bench_function("translate_simple_to_wasm", |b| {
        b.iter(|| {
            if let Ok(module) = parse_move_bytecode(&bytecode) {
                let _ = translate_to_wasm(black_box(&module));
            }
        })
    });
}

fn bench_module_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("module_sizes");

    for size in [10, 50, 100] {
        let mut bytecode = create_simple_move_module();
        // Pad with dummy data to simulate larger modules
        bytecode.extend(vec![0u8; size]);

        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &bytecode,
            |b, bc| {
                b.iter(|| {
                    let _ = parse_move_bytecode(black_box(bc));
                })
            },
        );
    }

    group.finish();
}

fn bench_end_to_end_translation(c: &mut Criterion) {
    let bytecode = create_module_with_functions();

    c.bench_function("end_to_end_move_to_wasm", |b| {
        b.iter(|| {
            if let Ok(module) = parse_move_bytecode(black_box(&bytecode)) {
                let _ = translate_to_wasm(black_box(&module));
            }
        })
    });
}

criterion_group!(
    benches,
    bench_parse_move_bytecode,
    bench_translate_to_wasm,
    bench_module_sizes,
    bench_end_to_end_translation
);
criterion_main!(benches);
