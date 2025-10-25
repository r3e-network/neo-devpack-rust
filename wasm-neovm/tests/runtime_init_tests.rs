use wasm_neovm::{opcodes, translate_module};

#[test]
fn runtime_initialization_runs_once() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "main") (result i32)
                i32.const 0
                i32.load)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "InitGuard").expect("translation succeeds");

    let init_slot = opcodes::lookup("INITSSLOT").unwrap().byte;
    let count = translation
        .script
        .iter()
        .filter(|&&byte| byte == init_slot)
        .count();

    assert_eq!(count, 1, "expected a single INITSSLOT invocation");
}
