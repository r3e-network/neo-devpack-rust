use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use wasm_neovm::{translate_module, translate_with_config, SourceChain, TranslationConfig};

/// Simple WASM module with basic arithmetic
const SIMPLE_WASM: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, // magic
    0x01, 0x00, 0x00, 0x00, // version
    // Type section
    0x01, 0x07, 0x01, 0x60, 0x02, 0x7f, 0x7f, 0x01, 0x7f, // Function section
    0x03, 0x02, 0x01, 0x00, // Export section
    0x07, 0x07, 0x01, 0x03, 0x61, 0x64, 0x64, 0x00, 0x00, // Code section
    0x0a, 0x09, 0x01, 0x07, 0x00, 0x20, 0x00, 0x20, 0x01, 0x6a, 0x0b,
];

/// WASM module with control flow
const CONTROL_FLOW_WASM: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, // Type section: (i32) -> i32
    0x01, 0x06, 0x01, 0x60, 0x01, 0x7f, 0x01, 0x7f, // Function section
    0x03, 0x02, 0x01, 0x00, // Export section
    0x07, 0x08, 0x01, 0x04, 0x74, 0x65, 0x73, 0x74, 0x00, 0x00,
    // Code section: if-else with comparison
    0x0a, 0x11, 0x01, 0x0f, 0x00, 0x20, 0x00, 0x41, 0x0a, 0x4a, 0x04, 0x7f, 0x41, 0x01, 0x05, 0x41,
    0x00, 0x0b, 0x0b,
];

fn bench_simple_translation(c: &mut Criterion) {
    c.bench_function("translate_simple_wasm", |b| {
        b.iter(|| translate_module(black_box(SIMPLE_WASM), black_box("bench-simple")))
    });
}

fn bench_control_flow_translation(c: &mut Criterion) {
    c.bench_function("translate_control_flow_wasm", |b| {
        b.iter(|| {
            translate_module(
                black_box(CONTROL_FLOW_WASM),
                black_box("bench-control-flow"),
            )
        })
    });
}

fn bench_translation_with_different_configs(c: &mut Criterion) {
    let mut group = c.benchmark_group("translation_configs");

    for source_chain in [SourceChain::Neo, SourceChain::Solana] {
        group.bench_with_input(
            BenchmarkId::new("source-chain", format!("{source_chain:?}")),
            &source_chain,
            |b, &chain| {
                b.iter(|| {
                    let config = TranslationConfig::new("bench-config").with_source_chain(chain);
                    translate_with_config(black_box(SIMPLE_WASM), black_box(config))
                })
            },
        );
    }

    group.finish();
}

fn bench_repeated_translation(c: &mut Criterion) {
    let mut group = c.benchmark_group("repeated_translation");

    for count in [1, 10, 100] {
        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, &count| {
            b.iter(|| {
                for _ in 0..count {
                    let _ = translate_module(black_box(SIMPLE_WASM), black_box("bench-repeat"));
                }
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_simple_translation,
    bench_control_flow_translation,
    bench_translation_with_different_configs,
    bench_repeated_translation
);
criterion_main!(benches);
