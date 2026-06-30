// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT
//! Debugging surface: disassemble a contract + single-step trace a method.
use neo_vm_test::Contract;

#[test]
fn disassemble_and_trace() {
    let c = Contract::compile("contracts/feature-arithmetic").expect("compile");
    let asm = c.disassemble().expect("disasm");
    assert!(asm.contains("SYSCALL") || asm.contains("RET"), "disasm produced opcodes:\n{}", &asm[..asm.len().min(200)]);
    assert!(asm.lines().count() > 20, "disasm should have many instructions");

    let tr = c.trace("calc", &[20.into(), 6.into()]).expect("trace");
    assert!(tr.contains("estack="), "trace shows the eval stack");
    assert!(tr.contains("final state="), "trace shows the final state");
}
