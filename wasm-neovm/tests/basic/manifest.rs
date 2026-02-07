use serde_json::Value;
use wasm_neovm::translate_module;

#[test]
fn translate_marks_storage_feature_when_storage_syscall_used() {
    let wasm = wat::parse_str(
        r#"(module
              (import "syscall" "System.Storage.Get" (func $storage_get (param i32 i32) (result i32)))
              (func (export "main") (result i32)
                i32.const 0
                i32.const 0
                call $storage_get))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "StorageUser").expect("translation succeeds");
    let manifest_json: Value = serde_json::from_str(
        &translation
            .manifest
            .to_string()
            .expect("manifest serialises"),
    )
    .expect("manifest parses");
    assert_eq!(manifest_json["features"], serde_json::json!({}));
}

#[test]
fn translate_marks_payable_feature_for_on_nep17_payment() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "onNEP17Payment") (param i32 i32 i32) (result i32)
                i32.const 0))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "PayableContract").expect("translation succeeds");
    let manifest_json: Value = serde_json::from_str(
        &translation
            .manifest
            .to_string()
            .expect("manifest serialises"),
    )
    .expect("manifest parses");
    assert_eq!(manifest_json["features"], serde_json::json!({}));
}

#[test]
fn translate_marks_safe_methods() {
    let wasm = wat::parse_str(
        r#"(module
              (@custom "neo.manifest" "{\"abi\":{\"methods\":[{\"name\":\"main\",\"safe\":true}]}}")
              (func (export "main") (result i32)
                i32.const 1)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Adder").expect("translation succeeds");

    let methods = translation
        .manifest
        .value
        .get("abi")
        .and_then(|abi| abi.get("methods"))
        .and_then(|methods| methods.as_array())
        .expect("manifest methods present");

    let main_method = methods
        .iter()
        .find(|method| method.get("name").and_then(|v| v.as_str()) == Some("main"))
        .expect("main method present");

    assert_eq!(
        main_method.get("safe").and_then(|v| v.as_bool()),
        Some(true)
    );
}

#[test]
fn translate_missing_safe_method_errors() {
    let wasm = wat::parse_str(
        r#"(module
              (@custom "neo.manifest" "{\"abi\":{\"methods\":[{\"name\":\"unknown\",\"safe\":true}]}}")
              (func (export "main") (result i32)
                i32.const 1)
            )"#,
    )
    .expect("valid wat");

    let error = translate_module(&wasm, "Adder").expect_err("unknown safe method should error");

    let message = error.to_string();
    assert!(message.contains("manifest overlays"));
    assert!(message.contains("unknown"));
}

#[test]
fn translate_manifest_custom_section_merges_metadata() {
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

    let overlay_primary = r#"{
        "abi": {
            "events": [
                {"name": "Transfer", "parameters": [
                    {"name": "from", "type": "Hash160"},
                    {"name": "to", "type": "Hash160"}
                ]}
            ]
        },
        "permissions": [
            {"contract": "0xff", "methods": ["balanceOf"]}
        ],
        "supportedstandards": ["NEP-17"],
        "trusts": ["*"]
    }"#;

    let custom_primary = CustomSection {
        name: Cow::Borrowed("neo.manifest"),
        data: Cow::Borrowed(overlay_primary.as_bytes()),
    };
    module.section(&custom_primary);

    let overlay_secondary = r#"{
        "abi": {
            "events": [
                {"name": "Approval", "parameters": [
                    {"name": "owner", "type": "Hash160"},
                    {"name": "spender", "type": "Hash160"}
                ]}
            ]
        },
        "supportedstandards": ["NEP-11"],
        "trusts": ["0x01"],
        "extra": {
            "nefSource": "ipfs://meta-contract",
            "nefMethodTokens": [{
                "hash": "0102030405060708090a0b0c0d0e0f1011121314",
                "method": "balanceOf",
                "paramcount": 2,
                "hasreturnvalue": true,
                "callflags": 3
            }]
        }
    }"#;

    let custom_secondary = CustomSection {
        name: Cow::Borrowed("neo.manifest"),
        data: Cow::Borrowed(overlay_secondary.as_bytes()),
    };
    module.section(&custom_secondary);

    let wasm = module.finish();

    let translation = translate_module(&wasm, "Meta").expect("translation succeeds");

    let events = translation
        .manifest
        .value
        .get("abi")
        .and_then(|abi| abi.get("events"))
        .and_then(|events| events.as_array())
        .expect("events present");
    assert_eq!(events.len(), 2);
    assert_eq!(
        events[0].get("name").and_then(|v| v.as_str()),
        Some("Transfer")
    );
    assert_eq!(
        events[1].get("name").and_then(|v| v.as_str()),
        Some("Approval")
    );
    let event_params = events[0]
        .get("parameters")
        .and_then(|p| p.as_array())
        .expect("event params present");
    assert_eq!(event_params.len(), 2);
    assert_eq!(
        event_params[0].get("type").and_then(|v| v.as_str()),
        Some("Hash160")
    );

    let permissions = translation
        .manifest
        .value
        .get("permissions")
        .and_then(|p| p.as_array())
        .expect("permissions present");
    assert_eq!(permissions.len(), 1);
    assert_eq!(
        permissions[0]
            .get("contract")
            .and_then(|v| v.as_str())
            .unwrap(),
        "0xff"
    );
    let methods = permissions[0]
        .get("methods")
        .and_then(|m| m.as_array())
        .expect("permission methods present");
    assert_eq!(methods[0].as_str().unwrap(), "balanceOf");

    let standards = translation
        .manifest
        .value
        .get("supportedstandards")
        .and_then(|s| s.as_array())
        .expect("supported standards present");
    assert_eq!(standards.len(), 2);
    assert_eq!(standards[0].as_str().unwrap(), "NEP-17");
    assert_eq!(standards[1].as_str().unwrap(), "NEP-11");

    let trusts = translation
        .manifest
        .value
        .get("trusts")
        .and_then(|t| t.as_array())
        .expect("trusts present");
    assert_eq!(trusts.len(), 2);
    assert_eq!(trusts[0].as_str().unwrap(), "*");
    assert_eq!(trusts[1].as_str().unwrap(), "0x01");

    assert_eq!(
        translation.source_url.as_deref(),
        Some("ipfs://meta-contract")
    );
    let tokens = &translation.method_tokens;
    assert_eq!(tokens.len(), 1);
    let token = &tokens[0];
    let mut expected_hash = [0u8; 20];
    expected_hash
        .copy_from_slice(&hex::decode("0102030405060708090a0b0c0d0e0f1011121314").unwrap());
    assert_eq!(token.contract_hash, expected_hash);
    assert_eq!(token.method, "balanceOf");
    assert_eq!(token.parameters_count, 2);
    assert!(token.has_return_value);
    assert_eq!(token.call_flags, 3);
}
