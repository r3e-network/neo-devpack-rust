use crate::common::{double_sha256_checksum, read_var_uint};
use std::convert::TryInto;
use std::fs;
use tempfile::tempdir;
use wasm_neovm::{translate_module, write_nef, write_nef_with_metadata, MethodToken};

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
        // Valid call_flags: ReadStates(1) | WriteStates(2) | AllowCall(4) | AllowModifyAccount(8)
        // Max valid value is 0x0F (15). Using 0x07 = ReadStates | WriteStates | AllowCall
        call_flags: 0x07,
    };

    write_nef_with_metadata(
        &script,
        Some("ipfs://example"),
        std::slice::from_ref(&token),
        &nef_path,
    )
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
    let expected_checksum = double_sha256_checksum(&bytes[..cursor]);
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
    let add = wasm_neovm::opcodes::lookup("ADD").unwrap().byte;
    assert!(translation.script.contains(&add));
    assert_eq!(translation.script.last().copied(), Some(0x40));

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
    let expected_checksum = double_sha256_checksum(&bytes[..cursor]);
    assert_eq!(checksum, expected_checksum);
    assert_eq!(cursor + 4, bytes.len());
}
