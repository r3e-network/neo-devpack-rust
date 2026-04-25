// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use wasm_neovm::{opcodes, syscalls, translate_module};

const GAS_HASH_BE: [u8; 20] = [
    0xd2, 0xa4, 0xcf, 0xf3, 0x19, 0x13, 0x01, 0x61, 0x55, 0xe3, 0x8e, 0x47, 0x4a, 0x2c, 0x06, 0xd0,
    0x8b, 0xe2, 0x76, 0xcf,
];
const GAS_HASH_LE: [u8; 20] = [
    0xcf, 0x76, 0xe2, 0x8b, 0xd0, 0x06, 0x2c, 0x4a, 0x47, 0x8e, 0xe3, 0x55, 0x61, 0x01, 0x13, 0x19,
    0xf3, 0xcf, 0xa4, 0xd2,
];

fn count_subsequence(haystack: &[u8], needle: &[u8]) -> usize {
    haystack
        .windows(needle.len())
        .filter(|w| *w == needle)
        .count()
}

#[test]
fn on_nep17_payment_translation_does_not_inject_app_specific_guards() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "onNEP17Payment") (param i32 i32 i32) (result i32)
                i32.const 0))"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "OnNep17Guard").expect("translation succeeds");
    let script = &translation.script;

    let syscall_opcode = opcodes::lookup("SYSCALL").expect("SYSCALL opcode").byte;
    let calling_hash_syscall = syscalls::lookup("System.Runtime.GetCallingScriptHash")
        .expect("calling hash syscall")
        .hash
        .to_le_bytes();
    let mut calling_hash_pattern = vec![syscall_opcode];
    calling_hash_pattern.extend_from_slice(&calling_hash_syscall);
    assert_eq!(
        count_subsequence(script, &calling_hash_pattern),
        0,
        "generic onNEP17Payment translation must not inject caller-hash policy"
    );

    assert!(
        !script
            .windows(GAS_HASH_LE.len())
            .any(|window| window == GAS_HASH_LE),
        "generic onNEP17Payment translation must not inject GAS hash policy"
    );
    assert!(
        !script
            .windows(GAS_HASH_BE.len())
            .any(|window| window == GAS_HASH_BE),
        "generic onNEP17Payment translation must not inject GAS hash policy"
    );

    let assert_opcode = opcodes::lookup("ASSERT").expect("ASSERT opcode").byte;
    let istype_opcode = opcodes::lookup("ISTYPE").expect("ISTYPE opcode").byte;
    let convert_opcode = opcodes::lookup("CONVERT").expect("CONVERT opcode").byte;
    let pushint8_opcode = opcodes::lookup("PUSHINT8").expect("PUSHINT8 opcode").byte;
    assert!(
        !script.contains(&assert_opcode),
        "generic translation must not inject ASSERT guards"
    );
    assert!(
        !script.contains(&istype_opcode),
        "generic translation must not inject data-shape adapter checks"
    );
    assert!(
        !script.contains(&convert_opcode),
        "generic translation must not inject data conversion adapters"
    );
    assert!(
        !script
            .windows(2)
            .any(|window| window == [pushint8_opcode, 101]),
        "generic translation must not inject red-envelope sentinel payloads"
    );
}
