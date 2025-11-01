use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::convert::TryInto;
use std::fs;
use tempfile::tempdir;
use wasm_neovm::{opcodes, translate_module, write_nef, write_nef_with_metadata, MethodToken};

#[test]
fn write_nef_with_metadata_serializes_tokens_and_source() {
    let dir = tempdir().expect("tempdir");
    let nef_path = dir.path().join("metadata.nef");
    let script = vec![0x01, 0x00, 0x12, 0x40];

    let token = MethodToken {
        contract_hash: [
            0xAA, 0xBB, 0xCC, 0xDD, 0x01, 0x23, 0x45, 0x67, 0x89, 0x10, 0x20, 0x30, 0x40, 0x50,
            0x60, 0x70, 0x80, 0x90, 0xA0, 0xB0,
        ],
        method: "callExternal".to_string(),
        parameters_count: 3,
        has_return_value: false,
        call_flags: 0x11,
    };

    write_nef_with_metadata(&script, Some("ipfs://example"), &[token.clone()], &nef_path)
        .expect("nef written");

    let bytes = fs::read(&nef_path).expect("nef exists");
    let expected_compiler = concat!("neo-llvm wasm-neovm ", env!("CARGO_PKG_VERSION"));
    let mut cursor = 0usize;
    assert_eq!(&bytes[cursor..cursor + 4], b"NEF3");
    cursor += 4;

    let compiler_field = &bytes[cursor..cursor + 64];
    let compiler_bytes = expected_compiler.as_bytes();
    assert_eq!(&compiler_field[..compiler_bytes.len()], compiler_bytes);
    assert!(compiler_field[compiler_bytes.len()..]
        .iter()
        .all(|&b| b == 0));
    cursor += 64;

    let (source_len, consumed) = read_var_uint(&bytes[cursor..]);
    assert_eq!(source_len, "ipfs://example".len() as u64);
    cursor += consumed;
    let source_bytes = &bytes[cursor..cursor + source_len as usize];
    assert_eq!(source_bytes, b"ipfs://example");
    cursor += source_len as usize;

    assert_eq!(bytes[cursor], 0);
    cursor += 1;

    let (token_count, consumed) = read_var_uint(&bytes[cursor..]);
    assert_eq!(token_count, 1);
    cursor += consumed;

    assert_eq!(&bytes[cursor..cursor + 20], token.contract_hash);
    cursor += 20;

    let (method_len, consumed) = read_var_uint(&bytes[cursor..]);
    assert_eq!(method_len, token.method.len() as u64);
    cursor += consumed;
    let method_bytes = &bytes[cursor..cursor + method_len as usize];
    assert_eq!(method_bytes, token.method.as_bytes());
    cursor += method_len as usize;

    let params = u16::from_le_bytes(bytes[cursor..cursor + 2].try_into().unwrap());
    assert_eq!(params, token.parameters_count);
    cursor += 2;

    assert_eq!(bytes[cursor], 0);
    cursor += 1;

    assert_eq!(bytes[cursor], token.call_flags);
    cursor += 1;

    let reserved_after_tokens = u16::from_le_bytes(bytes[cursor..cursor + 2].try_into().unwrap());
    assert_eq!(reserved_after_tokens, 0);
    cursor += 2;

    let (script_len, consumed) = read_var_uint(&bytes[cursor..]);
    assert_eq!(script_len as usize, script.len());
    cursor += consumed;

    let script_bytes = &bytes[cursor..cursor + script.len()];
    assert_eq!(script_bytes, script.as_slice());
    cursor += script.len();

    let checksum = u32::from_le_bytes(bytes[cursor..cursor + 4].try_into().unwrap());
    let expected_checksum = {
        let hash = Sha256::digest(&bytes[..cursor]);
        let hash = Sha256::digest(hash);
        u32::from_le_bytes(hash[..4].try_into().unwrap())
    };
    assert_eq!(checksum, expected_checksum);
    assert_eq!(cursor + 4, bytes.len());
}

#[test]
fn translate_simple_constant_addition() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "main") (result i32)
                i32.const 41
                i32.const 1
                i32.add)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Adder").expect("translation succeeds");
    assert_eq!(translation.script, vec![0x00, 0x29, 0x11, 0x9E, 0x40]);

    let manifest = translation
        .manifest
        .to_string()
        .expect("manifest serialises");
    assert!(manifest.contains("\"name\": \"Adder\""));
    assert!(manifest.contains("\"returntype\": \"Integer\""));
    assert!(translation.method_tokens.is_empty());
    assert!(translation.source_url.is_none());

    let dir = tempdir().expect("tempdir");
    let nef_path = dir.path().join("adder.nef");
    write_nef(&translation.script, &nef_path).expect("nef written");

    let bytes = fs::read(&nef_path).expect("nef exists");

    let expected_compiler = concat!("neo-llvm wasm-neovm ", env!("CARGO_PKG_VERSION"));
    let mut cursor = 0usize;
    assert_eq!(&bytes[cursor..cursor + 4], b"NEF3");
    cursor += 4;

    let compiler_field = &bytes[cursor..cursor + 64];
    let compiler_bytes = expected_compiler.as_bytes();
    assert_eq!(&compiler_field[..compiler_bytes.len()], compiler_bytes);
    assert!(compiler_field[compiler_bytes.len()..]
        .iter()
        .all(|&b| b == 0));
    cursor += 64;

    let (source_len, consumed) = read_var_uint(&bytes[cursor..]);
    assert_eq!(source_len, 0);
    cursor += consumed;

    assert_eq!(bytes[cursor], 0);
    cursor += 1;

    let (token_count, consumed) = read_var_uint(&bytes[cursor..]);
    assert_eq!(token_count, 0);
    cursor += consumed;

    let reserved_after_tokens = u16::from_le_bytes(bytes[cursor..cursor + 2].try_into().unwrap());
    assert_eq!(reserved_after_tokens, 0);
    cursor += 2;

    let (script_len, consumed) = read_var_uint(&bytes[cursor..]);
    assert_eq!(script_len as usize, translation.script.len());
    cursor += consumed;

    let script_bytes = &bytes[cursor..cursor + translation.script.len()];
    assert_eq!(script_bytes, translation.script.as_slice());
    cursor += translation.script.len();

    let checksum = u32::from_le_bytes(bytes[cursor..cursor + 4].try_into().unwrap());
    let expected_checksum = {
        let hash = Sha256::digest(&bytes[..cursor]);
        let hash = Sha256::digest(hash);
        u32::from_le_bytes(hash[..4].try_into().unwrap())
    };
    assert_eq!(checksum, expected_checksum);
    assert_eq!(cursor + 4, bytes.len());
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
fn translate_emits_start_call() {
    let wasm = wat::parse_str(
        r#"(module
              (func $start (export "start")
                i32.const 0
                drop)
              (func (export "main") (result i32)
                i32.const 7)
              (start $start))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Starty").expect("translation succeeds");

    let methods = translation
        .manifest
        .value
        .get("abi")
        .and_then(|abi| abi.get("methods"))
        .and_then(|methods| methods.as_array())
        .expect("manifest methods present");

    let start_method = methods
        .iter()
        .find(|method| method.get("name").and_then(|v| v.as_str()) == Some("start"))
        .expect("start method present");
    let start_offset = start_method
        .get("offset")
        .and_then(|offset| offset.as_u64())
        .expect("start offset present") as isize;

    let call_opcode = opcodes::lookup("CALL_L")
        .expect("CALL_L opcode available")
        .byte;
    let mut found_call = false;
    let script = &translation.script;
    let mut i = 0usize;
    while i + 4 < script.len() {
        if script[i] == call_opcode {
            let delta = i32::from_le_bytes(script[i + 1..i + 5].try_into().unwrap());
            // CALL_L uses a relative offset from the instruction *after* the immediate
            let target = (i + 5) as isize + delta as isize;
            if target == start_offset {
                found_call = true;
                break;
            }
            i += 5;
        } else {
            i += 1;
        }
    }

    assert!(
        found_call,
        "expected CALL_L to start function at offset {start_offset}"
    );
}

#[test]
fn translate_rejects_start_with_result() {
    let wasm = wat::parse_str(
        r#"(module
              (func $start (result i32)
                i32.const 0)
              (func (export "main") (result i32)
                i32.const 1)
              (start $start))"#,
    )
    .expect("valid wat");

    let err = translate_module(&wasm, "HasResult").expect_err("translation should fail");
    let message = err.to_string();
    assert!(
        message.contains("start function must not return values"),
        "unexpected start-function error message: {message}"
    );
}

#[test]
fn translate_calls_imported_start_opcode() {
    let wasm = wat::parse_str(
        r#"(module
              (import "opcode" "NOP" (func $start))
              (func (export "main") (result i32)
                i32.const 2)
              (start $start))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ImportStart").expect("translation succeeds");
    let nop = opcodes::lookup("NOP").expect("NOP opcode available").byte;
    assert!(
        translation.script.contains(&nop),
        "expected emitted script to contain imported NOP start call"
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

#[test]
fn translate_syscall_import() {
    let wasm = wat::parse_str(
        r#"(module
              (import "syscall" "System.Runtime.GetTime" (func $get_time (result i64)))
              (func (export "main") (result i64)
                call $get_time)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Clock").expect("translation succeeds");

    assert_eq!(translation.script.len(), 6);
    let syscall_opcode = wasm_neovm::opcodes::lookup("SYSCALL").unwrap().byte;
    assert_eq!(translation.script[0], syscall_opcode); // SYSCALL
    assert_eq!(&translation.script[1..5], &[0xB7, 0xC3, 0x88, 0x03]);
    assert_eq!(translation.script[5], 0x40); // RET

    let manifest = translation
        .manifest
        .to_string()
        .expect("manifest serialises");
    assert!(manifest.contains("\"name\": \"Clock\""));
    assert!(manifest.contains("\"returntype\": \"Integer\""));
}

#[test]
fn translate_memory_size_uses_runtime_helper() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "size") (result i32)
                memory.size)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MemSize").expect("translation succeeds");

    let guard_ldsfld = wasm_neovm::opcodes::lookup("LDSFLD4").unwrap().byte;
    let guard_jump = wasm_neovm::opcodes::lookup("JMPIF_L").unwrap().byte;
    let call_l = wasm_neovm::opcodes::lookup("CALL_L").unwrap().byte;
    assert_eq!(translation.script[0], guard_ldsfld);
    assert_eq!(translation.script[1], guard_jump);
    assert!(translation.script.iter().any(|&b| b == call_l));

    let ldsfld2 = wasm_neovm::opcodes::lookup("LDSFLD2").unwrap().byte;
    assert!(translation.script.contains(&ldsfld2));

    let manifest = translation
        .manifest
        .to_string()
        .expect("manifest serialises");
    assert!(manifest.contains("\"returntype\": \"Integer\""));
}

#[test]
fn translate_i32_load_uses_helper() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (export "load" (func $load))
              (func $load (result i32)
                i32.const 0
                i32.load)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MemLoad").expect("translation succeeds");

    let push0 = wasm_neovm::opcodes::lookup("PUSH0").unwrap().byte;
    let guard_ldsfld = wasm_neovm::opcodes::lookup("LDSFLD4").unwrap().byte;
    let guard_jump = wasm_neovm::opcodes::lookup("JMPIF_L").unwrap().byte;
    let call_l = wasm_neovm::opcodes::lookup("CALL_L").unwrap().byte;
    let ret = wasm_neovm::opcodes::lookup("RET").unwrap().byte;

    assert_eq!(translation.script[0], push0);
    assert_eq!(translation.script[1], guard_ldsfld);
    assert_eq!(translation.script[2], guard_jump);

    let call_sites: Vec<_> = translation
        .script
        .iter()
        .enumerate()
        .filter(|(_, &byte)| byte == call_l)
        .map(|(idx, _)| idx)
        .collect();
    assert!(call_sites.len() >= 2, "expected helper calls to be emitted");

    assert!(translation
        .script
        .iter()
        .any(|&b| b == wasm_neovm::opcodes::lookup("SUBSTR").unwrap().byte));

    assert!(translation.script.iter().position(|&b| b == ret).is_some());
}

#[test]
fn translate_i32_store_uses_helper() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "store")
                i32.const 0
                i32.const 0xAB
                i32.store8)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MemStore").expect("translation succeeds");

    let call_l = wasm_neovm::opcodes::lookup("CALL_L").unwrap().byte;
    let setitem = wasm_neovm::opcodes::lookup("SETITEM").unwrap().byte;

    assert!(
        translation
            .script
            .windows(1)
            .filter(|window| window[0] == call_l)
            .count()
            >= 2
    );

    assert!(translation.script.contains(&setitem));
}

#[test]
fn translate_i32_bitcounts_fold_and_call_helpers() {
    let clz_wasm = wat::parse_str(
        r#"(module
              (func (export "clz_const") (result i32)
                i32.const 16
                i32.clz))"#,
    )
    .expect("valid wat");

    let clz_translation = translate_module(&clz_wasm, "BitClz").expect("translate clz");
    let call_l = wasm_neovm::opcodes::lookup("CALL_L").unwrap().byte;
    assert!(!clz_translation.script.contains(&call_l));

    let pushint8 = wasm_neovm::opcodes::lookup("PUSHINT8").unwrap().byte;
    assert!(clz_translation.script.starts_with(&[pushint8, 27]));

    let dynamic_wasm = wat::parse_str(
        r#"(module
              (func (export "ctz") (param i32) (result i32)
                local.get 0
                i32.ctz))"#,
    )
    .expect("valid wat");

    let dynamic_translation = translate_module(&dynamic_wasm, "BitCtz").expect("translate ctz");
    let call_count = dynamic_translation
        .script
        .iter()
        .filter(|&&b| b == call_l)
        .count();
    assert_eq!(call_count, 1);
}

#[test]
fn translate_memory_grow_returns_prev_or_fail() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "grow_zero") (result i32)
                i32.const 0
                memory.grow)
              (func (export "grow_fail") (result i32)
                i32.const 1
                memory.grow)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MemGrow").expect("translation succeeds");

    let call_l = wasm_neovm::opcodes::lookup("CALL_L").unwrap().byte;
    let pushm1 = wasm_neovm::opcodes::lookup("PUSHM1").unwrap().byte;

    assert!(translation.script.contains(&call_l));
    assert!(translation.script.contains(&pushm1));
}

#[test]
fn translate_memory_fill_and_copy_helpers() {
    let fill_wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "fill") (param i32 i32 i32)
                local.get 0
                local.get 1
                local.get 2
                memory.fill))"#,
    )
    .expect("valid wat");

    let fill_translation = translate_module(&fill_wasm, "MemFill").expect("translate fill");
    let call_l = wasm_neovm::opcodes::lookup("CALL_L").unwrap().byte;
    assert!(
        fill_translation
            .script
            .iter()
            .filter(|&&b| b == call_l)
            .count()
            >= 2
    );

    let initslot = wasm_neovm::opcodes::lookup("INITSLOT").unwrap().byte;
    let setitem = wasm_neovm::opcodes::lookup("SETITEM").unwrap().byte;
    assert!(fill_translation.script.contains(&initslot));
    assert!(fill_translation.script.contains(&setitem));

    let copy_wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "copy") (param i32 i32 i32)
                local.get 0
                local.get 1
                local.get 2
                memory.copy))"#,
    )
    .expect("valid wat");

    let copy_translation = translate_module(&copy_wasm, "MemCopy").expect("translate copy");
    let memcpy = wasm_neovm::opcodes::lookup("MEMCPY").unwrap().byte;
    assert!(copy_translation.script.contains(&memcpy));
    assert!(copy_translation.script.contains(&initslot));
}

#[test]
fn translate_memory_init_uses_passive_segment() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (data "Hi")
              (func (export "init") (result i32)
                i32.const 0
                i32.const 0
                i32.const 2
                memory.init 0 0
                i32.const 0))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MemInit").expect("translate memory.init");

    let pushdata1 = wasm_neovm::opcodes::lookup("PUSHDATA1").unwrap().byte;
    let memcpy = wasm_neovm::opcodes::lookup("MEMCPY").unwrap().byte;
    let stsfld4 = wasm_neovm::opcodes::lookup("STSFLD4").unwrap().byte;

    let hi_literal = b"\x02Hi";
    let has_inline_literal = translation
        .script
        .windows(hi_literal.len())
        .any(|window| window == hi_literal);

    assert!(
        translation.script.contains(&pushdata1) || has_inline_literal,
        "expected memory.init helper to embed segment literal via PUSHDATA1 or inline push; script did not contain {:?}",
        hi_literal
    );
    assert!(translation.script.contains(&memcpy));
    assert!(translation.script.contains(&stsfld4));
}

#[test]
fn translate_data_drop_emits_helper() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (data "OK")
              (func (export "drop")
                data.drop 0))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MemDrop").expect("translate data.drop");

    let stsfld5 = wasm_neovm::opcodes::lookup("STSFLD5").unwrap().byte;
    let abort = wasm_neovm::opcodes::lookup("ABORT").unwrap().byte;

    assert!(translation.script.contains(&stsfld5));
    assert!(translation.script.contains(&abort));
}

#[test]
fn translate_active_data_segment_initialises_memory() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (data (i32.const 2) "AB")
              (func (export "load") (result i32)
                i32.const 2
                i32.load8_u))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ActiveData").expect("translate active data");

    let memcpy = wasm_neovm::opcodes::lookup("MEMCPY").unwrap().byte;
    let pushdata1 = wasm_neovm::opcodes::lookup("PUSHDATA1").unwrap().byte;

    assert!(translation.script.contains(&memcpy));
    let ab_literal = b"\x02AB";
    let has_inline_literal = translation
        .script
        .windows(ab_literal.len())
        .any(|window| window == ab_literal);

    assert!(
        translation.script.contains(&pushdata1) || has_inline_literal,
        "expected active segment literal to be emitted via PUSHDATA1 or inline push; script did not contain {:?}",
        ab_literal
    );
}

#[test]
fn translate_global_get_constant() {
    let wasm = wat::parse_str(
        r#"(module
              (global $g i32 (i32.const 42))
              (func (export "main") (result i32)
                global.get $g))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "GlobalConst").expect("translate global const");

    let pushint8 = wasm_neovm::opcodes::lookup("PUSHINT8").unwrap().byte;
    assert!(translation.script.starts_with(&[pushint8, 42]));
}

#[test]
fn translate_global_set_mutable() {
    let wasm = wat::parse_str(
        r#"(module
              (global $g (mut i32) (i32.const 0))
              (func (export "set") (param i32)
                local.get 0
                global.set $g)
              (func (export "get") (result i32)
                global.get $g))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "GlobalMutable").expect("translate global mutable");

    let stsfld4 = wasm_neovm::opcodes::lookup("STSFLD4").unwrap().byte;
    let ldsfld4 = wasm_neovm::opcodes::lookup("LDSFLD4").unwrap().byte;

    assert!(translation.script.contains(&stsfld4));
    assert!(translation.script.contains(&ldsfld4));
}

#[test]
fn translate_call_indirect_dispatches() {
    let wasm = wat::parse_str(
        r#"(module
              (type $t (func (result i32)))
              (table funcref (elem $f0 $f1))
              (func $f0 (result i32)
                i32.const 1)
              (func $f1 (result i32)
                i32.const 2)
              (func (export "main") (param i32) (result i32)
                local.get 0
                call_indirect (type $t)))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "CallIndirect").expect("translate call_indirect");

    let call_l = wasm_neovm::opcodes::lookup("CALL_L").unwrap().byte;
    let abort = wasm_neovm::opcodes::lookup("ABORT").unwrap().byte;

    let call_count = translation.script.iter().filter(|&&b| b == call_l).count();
    assert!(call_count >= 2);
    assert!(translation.script.contains(&abort));
}

#[test]
fn translate_opcode_immediate_and_raw() {
    let wasm = wat::parse_str(
        r#"(module
              (import "opcode" "PUSHINT32" (func $push32 (param i32)))
              (import "opcode" "RAW" (func $raw (param i32)))
              (func (export "emit")
                i32.const 1234
                call $push32
                i32.const 222
                call $raw)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Emit").expect("translation succeeds");

    // PUSHINT32 opcode (0x02) followed by little-endian immediate, then raw byte, then RET.
    assert_eq!(translation.script.len(), 7);
    assert_eq!(translation.script[0], 0x02); // PUSHINT32
    assert_eq!(&translation.script[1..5], &1234i32.to_le_bytes());
    assert_eq!(translation.script[5], 222u8);
    assert_eq!(translation.script[6], 0x40); // RET
}

#[test]
fn translate_internal_function_call() {
    let wasm = wat::parse_str(
        r#"(module
              (func $helper (result i32)
                i32.const 5
                i32.const 7
                i32.add)
              (func (export "main") (result i32)
                call $helper))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Call").expect("translate call");

    let call_l = wasm_neovm::opcodes::lookup("CALL_L").unwrap().byte;
    let add = wasm_neovm::opcodes::lookup("ADD").unwrap().byte;

    let call_pos = translation
        .script
        .iter()
        .position(|&b| b == call_l)
        .expect("CALL_L emitted");
    let immediate = &translation.script[call_pos + 1..call_pos + 5];
    assert_ne!(immediate, &[0, 0, 0, 0], "call immediate patched");
    assert!(translation.script.contains(&add));
}

#[test]
fn translate_nop_is_ignored() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "main") (result i32)
                nop
                i32.const 9))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Nop").expect("translate nop");

    let push0 = wasm_neovm::opcodes::lookup("PUSH0").unwrap().byte;
    let const_byte = push0.wrapping_add(9);
    assert!(translation.script.contains(&const_byte));
}

#[test]
fn translate_native_contract_syscall() {
    let wasm = wat::parse_str(
        r#"(module
              (import "syscall" "System.Contract.Call" (func $call (param i32 i32 i32) (result i32)))
              (func (export "main") (result i32)
                i32.const 0
                i32.const 0
                i32.const 0
                call $call)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "NativeCall").expect("translation succeeds");
    assert_eq!(translation.script.last().copied(), Some(0x40)); // RET present
}

#[test]
fn translate_i64_arithmetic() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "sum") (result i64)
                i64.const 5000000000
                i64.const 7
                i64.add)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Sum64").expect("translation succeeds");
    let push64 = wasm_neovm::opcodes::lookup("PUSHINT64").unwrap().byte;
    assert_eq!(translation.script.first().copied(), Some(push64));
    assert_eq!(&translation.script[1..9], &5000000000i64.to_le_bytes());
    let add = wasm_neovm::opcodes::lookup("ADD").unwrap().byte;
    assert!(translation.script.contains(&add));
    assert_eq!(translation.script.last().copied(), Some(0x40));
}

#[test]
fn translate_i32_rotl_constant() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "rot") (result i32)
                i32.const 0x12
                i32.const 8
                i32.rotl)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Rot32").expect("translation succeeds");
    // 0x12 rotated left by 8 bits -> 0x1200.
    assert_eq!(translation.script, vec![0x01, 0x00, 0x12, 0x40]);
}

#[test]
fn translate_i64_rotr_constant() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "rot") (result i64)
                i64.const 0x0102030405060708
                i64.const 16
                i64.rotr)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Rot64").expect("translation succeeds");
    let expected = 0x0102030405060708u64.rotate_right(16).to_le_bytes();
    let mut script = vec![0x03];
    script.extend_from_slice(&expected);
    script.push(0x40);
    assert_eq!(translation.script, script);
}

#[test]
fn translate_i32_rotl_dynamic() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "rot") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.rotl)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "RotDyn32").expect("translation succeeds");
    assert_eq!(translation.script.last().copied(), Some(0x40));
}

#[test]
fn translate_i64_rotr_dynamic() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "rot") (param i64 i64) (result i64)
                local.get 0
                local.get 1
                i64.rotr)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "RotDyn64").expect("translation succeeds");
    assert_eq!(translation.script.last().copied(), Some(0x40));
}

#[test]
fn translate_block_with_branches() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "branch") (param i32)
                block
                  local.get 0
                  br_if 0
                  br 0
                end)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Branch").expect("translation succeeds");
    assert_eq!(translation.script.last().copied(), Some(0x40));
}

#[test]
fn translate_loop_with_back_edge() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "loop")
                loop
                  br 0
                end)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Loop").expect("translation succeeds");
    assert_eq!(translation.script.last().copied(), Some(0x40));
}

#[test]
fn translate_if_else_structure() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "cond") (param i32)
                block
                  local.get 0
                  if
                    i32.const 1
                    drop
                  else
                    i32.const 2
                    drop
                  end
                end)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Cond").expect("translation succeeds");
    assert_eq!(translation.script.last().copied(), Some(0x40));
}

#[test]
fn translate_i32_bitwise_chain() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "bits") (result i32)
                i32.const 6
                i32.const 3
                i32.and
                i32.const 1
                i32.or
                i32.const 2
                i32.xor)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Bits").expect("translation succeeds");
    let and = wasm_neovm::opcodes::lookup("AND").unwrap().byte;
    let or = wasm_neovm::opcodes::lookup("OR").unwrap().byte;
    let xor = wasm_neovm::opcodes::lookup("XOR").unwrap().byte;
    assert_eq!(
        translation.script,
        vec![0x16, 0x13, and, 0x11, or, 0x12, xor, 0x40]
    );
}

#[test]
fn translate_i32_signed_comparisons() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "cmp_const") (result i32)
                i32.const -1
                i32.const 0
                i32.lt_s)
              (func (export "cmp_dyn") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.gt_s)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "SignedCmp").expect("translation succeeds");

    let gt = wasm_neovm::opcodes::lookup("GT").unwrap().byte;
    let lt = wasm_neovm::opcodes::lookup("LT").unwrap().byte;
    assert!(translation.script.contains(&gt));
    assert!(translation.script.contains(&lt));

    let and = wasm_neovm::opcodes::lookup("AND").unwrap().byte;
    let and_count = translation.script.iter().filter(|&&b| b == and).count();
    assert_eq!(
        and_count, 0,
        "signed comparison should not insert masking ANDs"
    );
}

#[test]
fn translate_i32_unsigned_comparison_masks_operands() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "cmp") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.le_u)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "UnsignedCmp32").expect("translation succeeds");

    let and = wasm_neovm::opcodes::lookup("AND").unwrap().byte;
    let swap = wasm_neovm::opcodes::lookup("SWAP").unwrap().byte;
    let le = wasm_neovm::opcodes::lookup("LE").unwrap().byte;

    assert!(translation.script.iter().filter(|&&b| b == and).count() >= 2);
    assert!(translation.script.iter().filter(|&&b| b == swap).count() >= 2);
    assert!(translation.script.contains(&le));
}

#[test]
fn translate_i64_unsigned_comparison_masks_operands() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "cmp") (param i64 i64) (result i32)
                local.get 0
                local.get 1
                i64.ge_u
                i64.eqz)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "UnsignedCmp64").expect("translation succeeds");

    let push128 = wasm_neovm::opcodes::lookup("PUSHINT128").unwrap().byte;
    let and = wasm_neovm::opcodes::lookup("AND").unwrap().byte;
    let swap = wasm_neovm::opcodes::lookup("SWAP").unwrap().byte;
    let ge = wasm_neovm::opcodes::lookup("GE").unwrap().byte;

    assert!(translation.script.iter().filter(|&&b| b == push128).count() >= 1);
    assert!(translation.script.iter().filter(|&&b| b == and).count() >= 2);
    assert!(translation.script.iter().filter(|&&b| b == swap).count() >= 2);
    assert!(translation.script.contains(&ge));
}

#[test]
fn translate_select_dynamic() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "sel") (param i32 i32 i32) (result i32)
                local.get 0
                local.get 1
                local.get 2
                select)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Select").expect("translation succeeds");

    let jmp_if_not = wasm_neovm::opcodes::lookup("JMPIFNOT_L").unwrap().byte;
    let drop = wasm_neovm::opcodes::lookup("DROP").unwrap().byte;
    let jmp = wasm_neovm::opcodes::lookup("JMP_L").unwrap().byte;
    let nip = wasm_neovm::opcodes::lookup("NIP").unwrap().byte;

    let script = &translation.script;
    let jmp_if_pos = script
        .iter()
        .position(|&byte| byte == jmp_if_not)
        .expect("select emits JMPIFNOT_L");
    assert_eq!(script[jmp_if_pos + 5], drop);

    let jmp_pos = script
        .iter()
        .position(|&byte| byte == jmp)
        .expect("select emits JMP_L to skip else body");
    assert!(jmp_pos > jmp_if_pos);
    assert_eq!(script[jmp_pos + 5], nip);

    assert_eq!(script.last().copied(), Some(0x40));
}

#[test]
fn translate_ref_eq_handles_funcref() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "eq") (result i32)
                ref.null func
                ref.null func
                ref.eq))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "RefEq").expect("translate ref.eq");

    let pushm1 = wasm_neovm::opcodes::lookup("PUSHM1").unwrap().byte;
    let equal = wasm_neovm::opcodes::lookup("EQUAL").unwrap().byte;
    let ret = wasm_neovm::opcodes::lookup("RET").unwrap().byte;

    assert_eq!(translation.script, vec![pushm1, pushm1, equal, ret]);
}

#[test]
fn translate_ref_as_non_null_traps_on_const_null() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "trap")
                ref.null func
                ref.as_non_null))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "RefTrap").expect("translate ref.as_non_null");

    let pushm1 = wasm_neovm::opcodes::lookup("PUSHM1").unwrap().byte;
    let abort = wasm_neovm::opcodes::lookup("ABORT").unwrap().byte;
    let ret = wasm_neovm::opcodes::lookup("RET").unwrap().byte;

    assert_eq!(translation.script, vec![pushm1, abort, ret]);
}

#[test]
fn translate_ref_as_non_null_dynamic_guard() {
    let wasm = wat::parse_str(
        r#"(module
              (table funcref (elem $f))
              (func $f)
              (func (export "guard")
                i32.const 0
                table.get 0
                ref.as_non_null
                drop))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "RefGuard").expect("translate guard");

    let dup = wasm_neovm::opcodes::lookup("DUP").unwrap().byte;
    let pushm1 = wasm_neovm::opcodes::lookup("PUSHM1").unwrap().byte;
    let equal = wasm_neovm::opcodes::lookup("EQUAL").unwrap().byte;
    let jmpifnot = wasm_neovm::opcodes::lookup("JMPIFNOT_L").unwrap().byte;
    let drop = wasm_neovm::opcodes::lookup("DROP").unwrap().byte;
    let abort = wasm_neovm::opcodes::lookup("ABORT").unwrap().byte;

    let pattern = [dup, pushm1, equal, jmpifnot];
    let pos = translation
        .script
        .windows(pattern.len())
        .position(|window| window == pattern)
        .expect("ref.as_non_null guard sequence present");

    let abort_pos = translation
        .script
        .iter()
        .enumerate()
        .skip(pos + pattern.len())
        .find(|(_, &byte)| byte == abort)
        .map(|(idx, _)| idx)
        .expect("abort present in trap path");

    assert_eq!(translation.script[abort_pos - 1], drop);
}

#[test]
fn translate_typed_select_validates_type() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "sel") (param i64 i64 i32) (result i64)
                local.get 0
                local.get 1
                local.get 2
                select (result i64))
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "TypedSelect").expect("translation succeeds");

    let nip = wasm_neovm::opcodes::lookup("NIP").unwrap().byte;
    assert!(translation.script.contains(&nip));
    assert_eq!(translation.script.last().copied(), Some(0x40));
}

#[test]
fn translate_br_table_dynamic() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "dispatch") (param i32)
                block
                  block
                    local.get 0
                    br_table 1 0
                  end
                end)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Dispatch").expect("translation succeeds");

    let dup = wasm_neovm::opcodes::lookup("DUP").unwrap().byte;
    let jmp_if = wasm_neovm::opcodes::lookup("JMPIF_L").unwrap().byte;
    let drop = wasm_neovm::opcodes::lookup("DROP").unwrap().byte;

    assert!(translation.script.contains(&dup));
    assert!(translation.script.contains(&jmp_if));

    // Ensure there is at least one DROP to clear the index before branching.
    assert!(translation.script.iter().filter(|&&b| b == drop).count() >= 2);
}

#[test]
fn translate_br_table_constant_index() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "dispatch_const")
                block
                  block
                    i32.const 2
                    br_table 1 0
                  end
                end)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "DispatchConst").expect("translation succeeds");

    let dup = wasm_neovm::opcodes::lookup("DUP").unwrap().byte;
    assert!(
        !translation.script.contains(&dup),
        "constant br_table should not emit DUP comparisons"
    );
}

#[test]
fn translate_i32_wrap_i64_constant() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "wrap") (result i32)
                i64.const 0x1_0000_0001
                i32.wrap_i64)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Wrap").expect("translation succeeds");
    assert_eq!(translation.script, vec![0x11, 0x40]);
}

#[test]
fn translate_i64_extend_i32_signed_dynamic() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "extend") (param i32) (result i64)
                local.get 0
                i64.extend_i32_s)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ExtendS").expect("translation succeeds");
    let and = wasm_neovm::opcodes::lookup("AND").unwrap().byte;
    let shl = wasm_neovm::opcodes::lookup("SHL").unwrap().byte;
    let shr = wasm_neovm::opcodes::lookup("SHR").unwrap().byte;
    assert!(translation.script.contains(&and));
    assert!(translation.script.contains(&shl));
    assert!(translation.script.contains(&shr));
    assert_eq!(translation.script.last().copied(), Some(0x40));
}

#[test]
fn translate_i64_extend_i32_unsigned_masks_only() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "extend") (param i32) (result i64)
                local.get 0
                i64.extend_i32_u)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ExtendU").expect("translation succeeds");
    let and = wasm_neovm::opcodes::lookup("AND").unwrap().byte;
    let shr = wasm_neovm::opcodes::lookup("SHR").unwrap().byte;
    assert!(translation.script.contains(&and));
    assert!(!translation.script.contains(&shr));
}

#[test]
fn translate_i32_extend8_sign_constant() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "extend") (result i32)
                i32.const 0xFF
                i32.extend8_s)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Extend8").expect("translation succeeds");
    let pushm1 = wasm_neovm::opcodes::lookup("PUSHM1").unwrap().byte;
    assert_eq!(translation.script, vec![pushm1, 0x40]);
}

#[test]
fn translate_i64_extend32_sign_constant() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "extend") (result i64)
                i64.const 0xFFFF_FFFF
                i64.extend32_s)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Extend32").expect("translation succeeds");
    let pushm1 = wasm_neovm::opcodes::lookup("PUSHM1").unwrap().byte;
    assert_eq!(translation.script, vec![pushm1, 0x40]);
}

#[test]
fn translate_i64_shift_ops() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "shift") (result i64)
                i64.const 8
                i64.const 1
                i64.shl
                i64.const 1
                i64.shr_s)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Shift").expect("translation succeeds");
    let shl = wasm_neovm::opcodes::lookup("SHL").unwrap().byte;
    let shr = wasm_neovm::opcodes::lookup("SHR").unwrap().byte;
    assert_eq!(translation.script.first().copied(), Some(0x18)); // PUSH8
    let shl_pos = translation.script.iter().position(|&b| b == shl).unwrap();
    assert_eq!(translation.script[shl_pos + 1], 0x11); // PUSH1 for shift amount
    let shr_pos = translation.script.iter().rposition(|&b| b == shr).unwrap();
    assert_eq!(translation.script[shr_pos + 1], 0x40); // RET
    let and = wasm_neovm::opcodes::lookup("AND").unwrap().byte;
    assert!(translation.script[shl_pos..shr_pos].contains(&and));
}

#[test]
fn translate_drop_eliminates_literal() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "main") (result i32)
                i32.const 42
                drop
                i32.const 1)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Drop").expect("translation succeeds");
    // Expect only PUSH1 and RET (DROP eliminates the first literal push).
    assert_eq!(translation.script, vec![0x11, 0x40]);
}

#[test]
fn translate_duplicate_exports_preserve_all_aliases() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "foo") (export "bar"))
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MultiExport").expect("translation succeeds");
    let methods = translation.manifest.value["abi"]["methods"]
        .as_array()
        .expect("methods array");
    let names: HashSet<_> = methods
        .iter()
        .map(|entry| entry["name"].as_str().unwrap())
        .collect();
    assert!(names.contains("foo"));
    assert!(names.contains("bar"));
    assert_eq!(names.len(), 2, "expected both export aliases to remain");

    let offsets: HashSet<_> = methods
        .iter()
        .map(|entry| entry["offset"].as_u64().unwrap())
        .collect();
    assert_eq!(
        offsets.len(),
        1,
        "all aliases should point at the same function offset"
    );
}

#[test]
fn translate_unreachable_emits_abort() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "main") (result i32)
                unreachable)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Trap").expect("translation succeeds");
    let abort = wasm_neovm::opcodes::lookup("ABORT").unwrap().byte;
    assert_eq!(translation.script.first().copied(), Some(abort));
}

#[test]
fn translate_import_reexport_generates_stub() {
    let wasm = wat::parse_str(
        r#"(module
              (import "syscall" "System.Runtime.GetTime" (func (result i64)))
              (export "get_time" (func 0))
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ReExport").expect("translation succeeds");
    assert!(
        !translation.script.is_empty(),
        "stub should emit executable script bytes"
    );
    let methods = translation.manifest.value["abi"]["methods"]
        .as_array()
        .expect("methods array");
    assert_eq!(methods.len(), 1);
    assert_eq!(methods[0]["name"].as_str().unwrap(), "get_time");
}

#[test]
fn translate_param_local_get() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "id") (param i32) (result i32)
                local.get 0)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Id").expect("translation succeeds");
    let ldarg0 = wasm_neovm::opcodes::lookup("LDARG0").unwrap().byte;
    assert_eq!(translation.script, vec![ldarg0, 0x40]);
}

#[test]
fn translate_local_set_and_get() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "mirror") (param i32) (result i32)
                (local i32)
                local.get 0
                local.set 1
                local.get 1)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Mirror").expect("translation succeeds");
    let ldarg0 = wasm_neovm::opcodes::lookup("LDARG0").unwrap().byte;
    let stloc0 = wasm_neovm::opcodes::lookup("STLOC0").unwrap().byte;
    let ldloc0 = wasm_neovm::opcodes::lookup("LDLOC0").unwrap().byte;
    assert_eq!(translation.script, vec![ldarg0, stloc0, ldloc0, 0x40]);
}

#[test]
fn translate_local_tee() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "tee") (param i32) (result i32)
                (local i32)
                local.get 0
                local.tee 1)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Tee").expect("translation succeeds");
    let ldarg0 = wasm_neovm::opcodes::lookup("LDARG0").unwrap().byte;
    let stloc0 = wasm_neovm::opcodes::lookup("STLOC0").unwrap().byte;
    let ldloc0 = wasm_neovm::opcodes::lookup("LDLOC0").unwrap().byte;
    assert_eq!(translation.script, vec![ldarg0, stloc0, ldloc0, 0x40]);
}

#[test]
fn translate_i64_parameter() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "id64") (param i64) (result i64)
                local.get 0)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Id64").expect("translation succeeds");
    let ldarg0 = wasm_neovm::opcodes::lookup("LDARG0").unwrap().byte;
    assert_eq!(translation.script, vec![ldarg0, 0x40]);
}

#[test]
fn translate_i32_shr_u() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "shr") (result i32)
                i32.const -1
                i32.const 1
                i32.shr_u)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "ShrU").expect("translation succeeds");
    let mut expected = Vec::new();
    expected.push(wasm_neovm::opcodes::lookup("PUSHM1").unwrap().byte);
    expected.push(wasm_neovm::opcodes::lookup("PUSH1").unwrap().byte);
    expected.push(wasm_neovm::opcodes::lookup("PUSHINT8").unwrap().byte);
    expected.push(0x1F);
    expected.push(wasm_neovm::opcodes::lookup("AND").unwrap().byte);
    expected.push(wasm_neovm::opcodes::lookup("SWAP").unwrap().byte);
    expected.push(wasm_neovm::opcodes::lookup("PUSHINT64").unwrap().byte);
    expected.extend_from_slice(&0xFFFF_FFFFu64.to_le_bytes());
    expected.push(wasm_neovm::opcodes::lookup("AND").unwrap().byte);
    expected.push(wasm_neovm::opcodes::lookup("SWAP").unwrap().byte);
    expected.push(wasm_neovm::opcodes::lookup("SHR").unwrap().byte);
    expected.push(0x40);
    assert_eq!(translation.script, expected);
}

#[test]
fn translate_table_helpers_cover_operations() {
    use std::borrow::Cow;
    use wasm_encoder::{
        CodeSection, ElementSection, Elements, ExportKind, ExportSection, Function,
        FunctionSection, HeapType, Module, RefType, TableSection, TableType, TypeSection, ValType,
    };

    let mut module = Module::new();

    let mut types = TypeSection::new();
    types.ty().function(vec![], vec![]); // type 0: [] -> []
    types.ty().function(vec![], vec![ValType::I32]); // type 1
    types.ty().function(vec![], vec![ValType::I32]); // type 2
    module.section(&types);

    let mut functions = FunctionSection::new();
    functions.function(0); // $target
    functions.function(1); // ops
    functions.function(2); // size
    functions.function(0); // drop (reuse type 0)
    module.section(&functions);

    let mut tables = TableSection::new();
    tables.table(TableType {
        element_type: RefType::FUNCREF,
        minimum: 4,
        maximum: None,
        shared: false,
        table64: false,
    });
    module.section(&tables);

    let mut exports = ExportSection::new();
    exports.export("ops", ExportKind::Func, 1);
    exports.export("size", ExportKind::Func, 2);
    exports.export("drop", ExportKind::Func, 3);
    module.section(&exports);

    let mut elements = ElementSection::new();
    elements.passive(Elements::Functions(Cow::Owned(vec![0])));
    module.section(&elements);

    let mut codes = CodeSection::new();

    // $target body (empty)
    let mut target_body = Function::new(vec![]);
    target_body.instructions().end();
    codes.function(&target_body);

    // ops body exercising table helpers
    let mut ops_body = Function::new(vec![]);
    ops_body
        .instructions()
        .i32_const(0)
        .ref_null(HeapType::FUNC)
        .i32_const(1)
        .table_fill(0)
        .i32_const(0)
        .i32_const(0)
        .i32_const(1)
        .table_init(0, 0)
        .i32_const(0)
        .table_get(0)
        .drop()
        .i32_const(2)
        .ref_func(0)
        .table_set(0)
        .i32_const(0)
        .i32_const(0)
        .i32_const(1)
        .table_copy(0, 0)
        .ref_func(0)
        .i32_const(1)
        .table_grow(0)
        .end();
    codes.function(&ops_body);

    // size body returns the current table size
    let mut size_body = Function::new(vec![]);
    size_body.instructions().table_size(0).end();
    codes.function(&size_body);

    // drop body releases the passive element segment
    let mut drop_body = Function::new(vec![]);
    drop_body.instructions().elem_drop(0).end();
    codes.function(&drop_body);

    module.section(&codes);

    let wasm = module.finish();

    let translation = translate_module(&wasm, "TableOps").expect("translation succeeds");

    let call_l = wasm_neovm::opcodes::lookup("CALL_L").unwrap().byte;
    let call_count = translation
        .script
        .iter()
        .filter(|&&opcode| opcode == call_l)
        .count();
    assert!(
        call_count >= 6,
        "expected helper calls for table operations"
    );

    let abort = wasm_neovm::opcodes::lookup("ABORT").unwrap().byte;
    assert!(
        translation.script.contains(&abort),
        "table dispatch should include abort traps"
    );

    let manifest = translation
        .manifest
        .to_string()
        .expect("manifest serialises");
    assert!(manifest.contains("\"name\": \"TableOps\""));
    assert!(manifest.contains("\"name\": \"ops\""));
}

#[test]
fn translate_table_inline_initializer() {
    let wasm = wat::parse_str(
        r#"(module
              (func $f0)
              (func $f1)
              (table funcref (elem $f0 $f1))
              (func (export "touch")
                i32.const 0
                table.get 0
                drop
                i32.const 1
                table.get 0
                drop))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "InlineTable").expect("translation succeeds");

    let sts_fld4 = wasm_neovm::opcodes::lookup("STSFLD4").unwrap().byte;
    assert!(
        translation.script.contains(&sts_fld4),
        "runtime init should store the table into static slot 4"
    );

    let lds_fld4 = wasm_neovm::opcodes::lookup("LDSFLD4").unwrap().byte;
    assert!(
        translation.script.contains(&lds_fld4),
        "table.get should load from the table static slot"
    );
}

#[test]
fn translate_multi_table_operations() {
    let wasm = wat::parse_str(
        r#"(module
              (type $t0 (func))
              (func $f0)
              (func $f1)
              (table 3 funcref)
              (table 2 funcref)
              (elem (i32.const 0) func $f0 $f1)
              (elem (table 1) (i32.const 1) func $f1)
              (func (export "manipulate") (result i32)
                i32.const 0
                table.get 0
                drop
                i32.const 1
                table.get 1
                drop
                i32.const 2
                ref.func $f0
                table.set 0
                i32.const 0
                ref.func $f1
                table.set 1
                i32.const 0
                i32.const 0
                i32.const 1
                table.copy 0 1
                i32.const 1
                ref.null func
                i32.const 1
                table.fill 1
                table.size 1))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "MultiTable").expect("translation succeeds");

    let lds_fld4 = wasm_neovm::opcodes::lookup("LDSFLD4").unwrap().byte;
    let lds_fld5 = wasm_neovm::opcodes::lookup("LDSFLD5").unwrap().byte;
    assert!(
        translation.script.contains(&lds_fld4),
        "table operations should load from table 0 slot"
    );
    assert!(
        translation.script.contains(&lds_fld5),
        "table operations should load from table 1 slot"
    );

    let sts_fld4 = wasm_neovm::opcodes::lookup("STSFLD4").unwrap().byte;
    let sts_fld5 = wasm_neovm::opcodes::lookup("STSFLD5").unwrap().byte;
    assert!(
        translation.script.contains(&sts_fld4),
        "runtime init should store table 0"
    );
    assert!(
        translation.script.contains(&sts_fld5),
        "runtime init should store table 1"
    );

    let call_l = wasm_neovm::opcodes::lookup("CALL_L").unwrap().byte;
    let helper_calls = translation
        .script
        .iter()
        .filter(|&&opcode| opcode == call_l)
        .count();
    assert!(
        helper_calls >= 4,
        "expected helper invocations for table ops"
    );
}

#[test]
fn translate_table_init_and_drop_guards() {
    use std::borrow::Cow;
    use wasm_encoder::{
        CodeSection, ElementSection, Elements, ExportKind, ExportSection, Function,
        FunctionSection, Module, TableSection, TableType, TypeSection,
    };

    let mut module = Module::new();

    let mut types = TypeSection::new();
    types.ty().function(vec![], vec![]);
    module.section(&types);

    let mut functions = FunctionSection::new();
    functions.function(0); // $f0
    functions.function(0); // init
    functions.function(0); // drop
    functions.function(0); // reuse
    module.section(&functions);

    let mut tables = TableSection::new();
    tables.table(TableType {
        element_type: wasm_encoder::RefType::FUNCREF,
        minimum: 2,
        maximum: None,
        shared: false,
        table64: false,
    });
    module.section(&tables);

    let mut exports = ExportSection::new();
    exports.export("init", ExportKind::Func, 1);
    exports.export("drop", ExportKind::Func, 2);
    exports.export("reuse", ExportKind::Func, 3);
    module.section(&exports);

    let mut elements = ElementSection::new();
    elements.passive(Elements::Functions(Cow::Owned(vec![0])));
    module.section(&elements);

    let mut codes = CodeSection::new();

    let mut f0_body = Function::new(vec![]);
    f0_body.instructions().end();
    codes.function(&f0_body);

    let mut init_body = Function::new(vec![]);
    init_body
        .instructions()
        .i32_const(0)
        .i32_const(0)
        .i32_const(1)
        .table_init(0, 0)
        .end();
    codes.function(&init_body);

    let mut drop_body = Function::new(vec![]);
    drop_body.instructions().elem_drop(0).end();
    codes.function(&drop_body);

    let mut reuse_body = Function::new(vec![]);
    reuse_body
        .instructions()
        .i32_const(0)
        .i32_const(0)
        .i32_const(1)
        .table_init(0, 0)
        .end();
    codes.function(&reuse_body);

    module.section(&codes);

    let wasm = module.finish();

    let translation = translate_module(&wasm, "TableReuse").expect("translation succeeds");

    let lds_drop = wasm_neovm::opcodes::lookup("LDSFLD6").unwrap().byte;
    assert!(
        translation.script.contains(&lds_drop),
        "table.init helper should load the passive element drop slot"
    );

    let sts_drop = wasm_neovm::opcodes::lookup("STSFLD6").unwrap().byte;
    assert!(
        translation.script.contains(&sts_drop),
        "elem.drop should mark the segment as dropped"
    );

    let equal = wasm_neovm::opcodes::lookup("EQUAL").unwrap().byte;
    assert!(
        translation.script.contains(&equal),
        "table.init guard should compare the drop flag against zero"
    );
}

#[test]
fn translate_table_passive_expression_segment() {
    let wasm = wat::parse_str(
        r#"(module
              (func $f0)
              (table 2 funcref)
              (elem (i32.const 0) func $f0)
              (elem funcref (ref.func $f0) (ref.null func))
              (func (export "init")
                i32.const 0
                i32.const 0
                i32.const 2
                table.init 0 1
                elem.drop 1
                i32.const 0
                i32.const 0
                i32.const 2
                table.init 0 1))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "TableExpr").expect("translation succeeds");

    let pushm1 = wasm_neovm::opcodes::lookup("PUSHM1").unwrap().byte;
    assert!(
        translation.script.contains(&pushm1),
        "passive expression segments should encode ref.null as -1"
    );

    let equal = wasm_neovm::opcodes::lookup("EQUAL").unwrap().byte;
    assert!(
        translation.script.contains(&equal),
        "table.init helper should compare drop flag against zero"
    );
}

#[test]
fn translate_table_grow_with_maximum() {
    let wasm = wat::parse_str(
        r#"(module
              (table 1 2 funcref)
              (func (export "grow_ok") (result i32)
                ref.null func
                i32.const 1
                table.grow 0)
              (func (export "grow_fail") (result i32)
                ref.null func
                i32.const 2
                table.grow 0))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "TableGrow").expect("translation succeeds");

    let call_l = wasm_neovm::opcodes::lookup("CALL_L").unwrap().byte;
    let helper_calls = translation
        .script
        .iter()
        .filter(|&&opcode| opcode == call_l)
        .count();
    assert!(
        helper_calls >= 2,
        "expected grow helpers to be invoked for both functions"
    );

    let pushm1 = wasm_neovm::opcodes::lookup("PUSHM1").unwrap().byte;
    assert!(
        translation.script.contains(&pushm1),
        "table.grow helper should return -1 when exceeding the maximum"
    );
}

#[test]
fn translate_i32_div_signed() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "divs") (result i32)
                i32.const -6
                i32.const 2
                i32.div_s)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "DivS").expect("translation succeeds");
    let div = wasm_neovm::opcodes::lookup("DIV").unwrap().byte;
    assert_eq!(
        translation.script,
        vec![
            wasm_neovm::opcodes::lookup("PUSHINT8").unwrap().byte,
            0xFA,
            0x12,
            div,
            0x40
        ]
    );
}

#[test]
fn translate_i32_div_unsigned_masks_operands() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "divu") (result i32)
                i32.const -1
                i32.const 3
                i32.div_u)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "DivU").expect("translation succeeds");
    let mut expected = Vec::new();
    expected.push(wasm_neovm::opcodes::lookup("PUSHM1").unwrap().byte);
    expected.push(wasm_neovm::opcodes::lookup("PUSH3").unwrap().byte);
    expected.push(wasm_neovm::opcodes::lookup("PUSHINT64").unwrap().byte);
    expected.extend_from_slice(&0xFFFF_FFFFu64.to_le_bytes());
    expected.push(wasm_neovm::opcodes::lookup("AND").unwrap().byte);
    expected.push(wasm_neovm::opcodes::lookup("SWAP").unwrap().byte);
    expected.push(wasm_neovm::opcodes::lookup("PUSHINT64").unwrap().byte);
    expected.extend_from_slice(&0xFFFF_FFFFu64.to_le_bytes());
    expected.push(wasm_neovm::opcodes::lookup("AND").unwrap().byte);
    expected.push(wasm_neovm::opcodes::lookup("SWAP").unwrap().byte);
    expected.push(wasm_neovm::opcodes::lookup("DIV").unwrap().byte);
    expected.push(0x40);
    assert_eq!(translation.script, expected);
}

#[test]
fn translate_i32_rem_unsigned_masks_operands() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "remu") (result i32)
                i32.const -1
                i32.const 3
                i32.rem_u)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "RemU").expect("translation succeeds");
    let mut expected = Vec::new();
    expected.push(wasm_neovm::opcodes::lookup("PUSHM1").unwrap().byte);
    expected.push(wasm_neovm::opcodes::lookup("PUSH3").unwrap().byte);
    expected.push(wasm_neovm::opcodes::lookup("PUSHINT64").unwrap().byte);
    expected.extend_from_slice(&0xFFFF_FFFFu64.to_le_bytes());
    expected.push(wasm_neovm::opcodes::lookup("AND").unwrap().byte);
    expected.push(wasm_neovm::opcodes::lookup("SWAP").unwrap().byte);
    expected.push(wasm_neovm::opcodes::lookup("PUSHINT64").unwrap().byte);
    expected.extend_from_slice(&0xFFFF_FFFFu64.to_le_bytes());
    expected.push(wasm_neovm::opcodes::lookup("AND").unwrap().byte);
    expected.push(wasm_neovm::opcodes::lookup("SWAP").unwrap().byte);
    expected.push(wasm_neovm::opcodes::lookup("MOD").unwrap().byte);
    expected.push(0x40);
    assert_eq!(translation.script, expected);
}

#[test]
fn translate_i64_rem_unsigned_masks_operands() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "remu64") (result i64)
                i64.const -1
                i64.const 3
                i64.rem_u)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "RemU64").expect("translation succeeds");
    let mut expected = Vec::new();
    expected.push(wasm_neovm::opcodes::lookup("PUSHM1").unwrap().byte);
    expected.push(wasm_neovm::opcodes::lookup("PUSH3").unwrap().byte);
    expected.push(wasm_neovm::opcodes::lookup("PUSHINT128").unwrap().byte);
    expected.extend_from_slice(&(((1u128 << 64) - 1).to_le_bytes()));
    expected.push(wasm_neovm::opcodes::lookup("AND").unwrap().byte);
    expected.push(wasm_neovm::opcodes::lookup("SWAP").unwrap().byte);
    expected.push(wasm_neovm::opcodes::lookup("PUSHINT128").unwrap().byte);
    expected.extend_from_slice(&(((1u128 << 64) - 1).to_le_bytes()));
    expected.push(wasm_neovm::opcodes::lookup("AND").unwrap().byte);
    expected.push(wasm_neovm::opcodes::lookup("SWAP").unwrap().byte);
    expected.push(wasm_neovm::opcodes::lookup("MOD").unwrap().byte);
    expected.push(0x40);
    assert_eq!(translation.script, expected);
}

fn read_var_uint(bytes: &[u8]) -> (u64, usize) {
    let prefix = bytes[0];
    match prefix {
        n if n < 0xFD => (u64::from(n), 1),
        0xFD => {
            let value = u16::from_le_bytes(bytes[1..3].try_into().unwrap());
            (u64::from(value), 3)
        }
        0xFE => {
            let value = u32::from_le_bytes(bytes[1..5].try_into().unwrap());
            (u64::from(value), 5)
        }
        0xFF => {
            let value = u64::from_le_bytes(bytes[1..9].try_into().unwrap());
            (value, 9)
        }
        _ => unreachable!(),
    }
}

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
fn translate_reports_float_unsupported() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "main") (result f32)
                f32.const 0
                f32.const 1
                f32.add)
            )"#,
    )
    .expect("valid wat");

    let err = translate_module(&wasm, "FloatOp").expect_err("float should be unsupported");
    let msg = format!("{:#}", err);
    assert!(msg.contains("floating point operation"), "message: {msg}");
    assert!(
        msg.contains("docs/wasm-neovm-status.md"),
        "hint missing: {msg}"
    );
}

#[test]
fn translate_reports_simd_unsupported() {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "main")
                v128.const i32x4 1 2 3 4
                drop)
            )"#,
    )
    .expect("valid wat");

    let err = translate_module(&wasm, "SimdOp").expect_err("simd should be unsupported");
    let msg = format!("{:#}", err);
    let lower = msg.to_lowercase();
    assert!(lower.contains("simd"), "message: {msg}");
    assert!(
        msg.contains("docs/wasm-neovm-status.md"),
        "hint missing: {msg}"
    );
}
