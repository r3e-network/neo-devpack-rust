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
fn on_nep17_payment_translation_emits_gas_caller_guards_and_adapter_markers() {
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
    assert!(
        count_subsequence(script, &calling_hash_pattern) >= 2,
        "onNEP17Payment should load calling script hash for LE/BE GAS checks"
    );

    assert!(
        script
            .windows(GAS_HASH_LE.len())
            .any(|window| window == GAS_HASH_LE),
        "little-endian GAS hash guard missing"
    );
    assert!(
        script
            .windows(GAS_HASH_BE.len())
            .any(|window| window == GAS_HASH_BE),
        "big-endian GAS hash guard missing"
    );

    let assert_opcode = opcodes::lookup("ASSERT").expect("ASSERT opcode").byte;
    let istype_opcode = opcodes::lookup("ISTYPE").expect("ISTYPE opcode").byte;
    let convert_opcode = opcodes::lookup("CONVERT").expect("CONVERT opcode").byte;
    let pushint8_opcode = opcodes::lookup("PUSHINT8").expect("PUSHINT8 opcode").byte;
    assert!(script.contains(&assert_opcode), "missing ASSERT guard");
    assert!(
        script.contains(&istype_opcode),
        "missing adapter ISTYPE checks"
    );
    assert!(
        script.contains(&convert_opcode),
        "missing adapter CONVERT path"
    );
    assert!(
        script
            .windows(2)
            .any(|window| window == [pushint8_opcode, 101]),
        "missing adapter invalid packet sentinel (101)"
    );
}
