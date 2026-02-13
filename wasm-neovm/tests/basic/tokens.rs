use serde_json::Value;
use wasm_neovm::translate_module;

#[test]
fn translate_method_token_section_populates_metadata() {
    use std::borrow::Cow;
    use wasm_encoder::{
        CodeSection, CustomSection, ExportKind, ExportSection, Function, FunctionSection, Module,
        TypeSection, ValType,
    };

    let mut module = Module::new();

    let mut types = TypeSection::new();
    types.ty().function(vec![], vec![ValType::I32]);
    module.section(&types);

    let mut functions = FunctionSection::new();
    functions.function(0);
    module.section(&functions);

    let mut exports = ExportSection::new();
    exports.export("main", ExportKind::Func, 0);
    module.section(&exports);

    let mut codes = CodeSection::new();
    let mut body = Function::new(vec![]);
    body.instructions().i32_const(1).end();
    codes.function(&body);
    module.section(&codes);

    let method_tokens_section = r#"{
        "nefSource": "ipfs://token-section",
        "nefMethodTokens": [{
            "hash": "0f1e2d3c4b5a69788796a5b4c3d2e1f0aabbccdd",
            "method": "transfer",
            "paramcount": 3,
            "hasreturnvalue": true,
            "callflags": 5
        }]
    }"#;

    let custom_section = CustomSection {
        name: Cow::Borrowed("neo.methodtokens"),
        data: Cow::Borrowed(method_tokens_section.as_bytes()),
    };
    module.section(&custom_section);

    let wasm = module.finish();
    let translation = translate_module(&wasm, "TokenMeta").expect("translation succeeds");

    assert_eq!(
        translation.source_url.as_deref(),
        Some("ipfs://token-section")
    );
    assert_eq!(translation.method_tokens.len(), 1);

    let token = &translation.method_tokens[0];
    assert_eq!(token.method, "transfer");
    assert_eq!(token.parameters_count, 3);
    assert!(token.has_return_value);
    assert_eq!(token.call_flags, 5);

    let mut expected_hash = [0u8; 20];
    expected_hash
        .copy_from_slice(&hex::decode("0f1e2d3c4b5a69788796a5b4c3d2e1f0aabbccdd").unwrap());
    assert_eq!(token.contract_hash, expected_hash);

    let extra = translation
        .manifest
        .value
        .get("extra")
        .and_then(Value::as_object)
        .expect("manifest extra present");

    let tokens_value = extra
        .get("nefMethodTokens")
        .and_then(Value::as_array)
        .expect("nefMethodTokens array present");
    assert_eq!(tokens_value.len(), 1);
    assert_eq!(tokens_value[0]["method"].as_str(), Some("transfer"));
}

#[test]
fn translate_infers_contract_call_tokens() {
    let wasm = wat::parse_str(
        r#"(module
              (import "opcode" "RAW" (func $raw (param i32)))
              (import "opcode" "PUSHINT32" (func $pushint32 (param i32)))
              (import "opcode" "NEWARRAY0" (func $newarray0))
              (import "syscall" "System.Contract.Call" (func $call (result i32)))
              (func (export "main") (result i32)
                ;; PUSHDATA1 (0x0C) with length 20
                i32.const 12
                call $raw
                i32.const 20
                call $raw
                i32.const 1
                call $raw
                i32.const 2
                call $raw
                i32.const 3
                call $raw
                i32.const 4
                call $raw
                i32.const 5
                call $raw
                i32.const 6
                call $raw
                i32.const 7
                call $raw
                i32.const 8
                call $raw
                i32.const 9
                call $raw
                i32.const 10
                call $raw
                i32.const 11
                call $raw
                i32.const 12
                call $raw
                i32.const 13
                call $raw
                i32.const 14
                call $raw
                i32.const 15
                call $raw
                i32.const 16
                call $raw
                i32.const 17
                call $raw
                i32.const 18
                call $raw
                i32.const 19
                call $raw
                i32.const 20
                call $raw
                ;; PUSHDATA1 (0x0C) with length 4 for "ping"
                i32.const 12
                call $raw
                i32.const 4
                call $raw
                i32.const 112
                call $raw
                i32.const 105
                call $raw
                i32.const 110
                call $raw
                i32.const 103
                call $raw
                i32.const 5
                call $pushint32
                call $newarray0
                call $call)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "StaticCaller").expect("translation succeeds");
    assert_eq!(translation.method_tokens.len(), 1);

    let token = &translation.method_tokens[0];
    let expected_hash: Vec<u8> = (1u8..=20).collect();
    assert_eq!(token.contract_hash.to_vec(), expected_hash);
    assert_eq!(token.method, "ping");
    assert_eq!(token.parameters_count, 0);
    assert_eq!(token.call_flags, 5);
    assert!(token.has_return_value);

    let extra = translation
        .manifest
        .value
        .get("extra")
        .and_then(Value::as_object)
        .expect("manifest extra present");
    let tokens_value = extra
        .get("nefMethodTokens")
        .and_then(Value::as_array)
        .expect("nefMethodTokens array present");
    assert_eq!(tokens_value.len(), 1);
    assert_eq!(tokens_value[0]["method"].as_str(), Some("ping"));
}

#[test]
fn translate_skips_contract_call_tokens_with_excessive_parameter_count() {
    let wasm = wat::parse_str(
        r#"(module
              (import "opcode" "RAW" (func $raw (param i32)))
              (import "opcode" "PUSHINT32" (func $pushint32 (param i32)))
              (import "opcode" "NEWARRAY0" (func $newarray0))
              (import "syscall" "System.Contract.Call" (func $call (result i32)))
              (func (export "main") (result i32)
                ;; PUSHDATA1 (0x0C) with length 20
                i32.const 12
                call $raw
                i32.const 20
                call $raw
                i32.const 1
                call $raw
                i32.const 2
                call $raw
                i32.const 3
                call $raw
                i32.const 4
                call $raw
                i32.const 5
                call $raw
                i32.const 6
                call $raw
                i32.const 7
                call $raw
                i32.const 8
                call $raw
                i32.const 9
                call $raw
                i32.const 10
                call $raw
                i32.const 11
                call $raw
                i32.const 12
                call $raw
                i32.const 13
                call $raw
                i32.const 14
                call $raw
                i32.const 15
                call $raw
                i32.const 16
                call $raw
                i32.const 17
                call $raw
                i32.const 18
                call $raw
                i32.const 19
                call $raw
                i32.const 20
                call $raw
                ;; PUSHDATA1 (0x0C) with length 4 for "ping"
                i32.const 12
                call $raw
                i32.const 4
                call $raw
                i32.const 112
                call $raw
                i32.const 105
                call $raw
                i32.const 110
                call $raw
                i32.const 103
                call $raw
                ;; 70000 args would overflow u16 if truncated
                i32.const 70000
                call $pushint32
                call $newarray0
                call $call)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "OverflowCaller").expect("translation succeeds");

    assert!(translation
        .method_tokens
        .iter()
        .all(|token| token.method != "ping"));
}

#[test]
fn translate_rejects_empty_contract_name() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "main")
                nop)
            )"#,
    )
    .expect("valid wat");

    let err = translate_module(&wasm, "").expect_err("empty contract name should error");
    let message = err.to_string();
    assert!(message
        .to_ascii_lowercase()
        .contains("contract name cannot be empty"));
}
