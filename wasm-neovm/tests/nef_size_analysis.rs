// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! Diagnostic test: measures NEF script sizes and bytecode composition.
//! Run with `cargo test -p wasm-neovm --test nef_size_analysis -- --nocapture`

use wasm_neovm::translate_module;

fn analyze(name: &str, wat: &str) -> usize {
    let wasm = wat::parse_str(wat).expect("valid wat");
    let t = translate_module(&wasm, name).expect("translate");
    let s = &t.script;

    let mut jumps_s = 0u32;
    let mut jumps_l = 0u32;
    let mut calls_s = 0u32;
    let mut calls_l = 0u32;

    for &b in s.iter() {
        match b {
            0x22 | 0x24 | 0x26 | 0x28 | 0x2A | 0x2C | 0x2E | 0x30 | 0x32 | 0x3D => {
                jumps_s += 1
            }
            0x23 | 0x25 | 0x27 | 0x29 | 0x2B | 0x2D | 0x2F | 0x31 | 0x33 | 0x3E => {
                jumps_l += 1
            }
            0x34 => calls_s += 1,
            0x35 => calls_l += 1,
            _ => {}
        }
    }

    eprintln!(
        "  {name:30} {len:5} bytes | jmp {js:2}s/{jl:2}l | call {cs:2}s/{cl:2}l",
        len = s.len(),
        js = jumps_s,
        jl = jumps_l,
        cs = calls_s,
        cl = calls_l,
    );
    s.len()
}

#[test]
fn nef_size_report() {
    eprintln!("\n=== NEF Script Size Report ===\n");

    let total: usize = [
        analyze(
            "simple_add",
            r#"(module (func (export "add") (param i32 i32) (result i32)
                local.get 0 local.get 1 i32.add))"#,
        ),
        analyze(
            "if_else",
            r#"(module (func (export "max") (param i32 i32) (result i32)
                local.get 0 local.get 1 i32.gt_s
                if (result i32) local.get 0 else local.get 1 end))"#,
        ),
        analyze(
            "memory_load_store",
            r#"(module (memory 1)
                (func (export "load") (result i32) i32.const 0 i32.load)
                (func (export "store") (param i32 i32) local.get 0 local.get 1 i32.store))"#,
        ),
        analyze(
            "br_table_4",
            r#"(module (func (export "dispatch") (param i32) (result i32)
                block $b3 block $b2 block $b1 block $b0
                    local.get 0 br_table $b0 $b1 $b2 $b3
                end i32.const 10 return
                end i32.const 20 return
                end i32.const 30 return
                end i32.const 40))"#,
        ),
        analyze(
            "recursive_factorial",
            r#"(module
                (func $fac (param i32) (result i32)
                    local.get 0 i32.const 1 i32.le_s
                    if (result i32) i32.const 1
                    else local.get 0 local.get 0 i32.const 1 i32.sub call $fac i32.mul end)
                (func (export "main") (result i32) i32.const 10 call $fac))"#,
        ),
        analyze(
            "memory_fill_copy",
            r#"(module (memory 1)
                (func (export "fill") (param i32 i32 i32)
                    local.get 0 local.get 1 local.get 2 memory.fill)
                (func (export "copy") (param i32 i32 i32)
                    local.get 0 local.get 1 local.get 2 memory.copy))"#,
        ),
        analyze(
            "globals",
            r#"(module (global $g (mut i32) (i32.const 0))
                (func (export "set") (param i32) local.get 0 global.set $g)
                (func (export "get") (result i32) global.get $g))"#,
        ),
        analyze(
            "multi_function",
            r#"(module
                (func $a (param i32) (result i32) local.get 0 i32.const 1 i32.add)
                (func $b (param i32) (result i32) local.get 0 call $a call $a)
                (func (export "main") (result i32) i32.const 5 call $b))"#,
        ),
    ]
    .into_iter()
    .sum();

    eprintln!("\n  {label:30} {total:5} bytes", label = "TOTAL");
    eprintln!("\n=== End Report ===\n");

    // Dump bytecode for simple_add to understand overhead
    let wasm = wat::parse_str(
        r#"(module (func (export "add") (param i32 i32) (result i32)
            local.get 0 local.get 1 i32.add))"#,
    )
    .expect("valid wat");
    let t = translate_module(&wasm, "simple_add").expect("translate");
    eprintln!("=== simple_add bytecode dump ===");
    let table = build_opcode_table();
    let mut pc = 0usize;
    while pc < t.script.len() {
        let byte = t.script[pc];
        let info = table[byte as usize];
        let (name, size) = match info {
            Some(i) => {
                let s = if i.operand_size_prefix == 0 {
                    1 + i.operand_size as usize
                } else {
                    let ps = pc + 1;
                    let prefix = i.operand_size_prefix as usize;
                    let ol = match prefix {
                        1 => t.script.get(ps).copied().unwrap_or(0) as usize,
                        2 => {
                            let a = t.script.get(ps).copied().unwrap_or(0);
                            let b = t.script.get(ps + 1).copied().unwrap_or(0);
                            u16::from_le_bytes([a, b]) as usize
                        }
                        _ => 0,
                    };
                    1 + prefix + ol
                };
                (i.name, s)
            }
            None => ("???", 1),
        };
        let hex: String = t.script[pc..pc + size.min(t.script.len() - pc)]
            .iter()
            .map(|b| format!("{b:02X}"))
            .collect::<Vec<_>>()
            .join(" ");
        eprintln!("  {pc:4}: {hex:20} {name}");
        pc += size;
    }
    eprintln!("=== end dump ===\n");
}

fn build_opcode_table() -> [Option<&'static wasm_neovm::opcodes::OpcodeInfo>; 256] {
    let mut table: [Option<&'static wasm_neovm::opcodes::OpcodeInfo>; 256] = [None; 256];
    for info in wasm_neovm::opcodes::all() {
        table[info.byte as usize] = Some(info);
    }
    table
}

#[test]
fn nef_size_details() {
    let cases = vec![
        ("br_table_4", r#"(module (func (export "dispatch") (param i32) (result i32)
            block $b3 block $b2 block $b1 block $b0
                local.get 0 br_table $b0 $b1 $b2 $b3
            end i32.const 10 return
            end i32.const 20 return
            end i32.const 30 return
            end i32.const 40))"#),
        ("globals", r#"(module (global $g (mut i32) (i32.const 0))
            (func (export "set") (param i32) local.get 0 global.set $g)
            (func (export "get") (result i32) global.get $g))"#),
    ];
    let table = build_opcode_table();
    for (name, wat) in &cases {
        let wasm = wat::parse_str(wat).expect("valid wat");
        let t = translate_module(&wasm, name).expect("translate");
        eprintln!("\n=== {name} bytecode ({} bytes) ===", t.script.len());
        let mut pc = 0usize;
        while pc < t.script.len() {
            let byte = t.script[pc];
            let info = table[byte as usize];
            let (iname, size) = match info {
                Some(i) => {
                    let s = if i.operand_size_prefix == 0 {
                        1 + i.operand_size as usize
                    } else {
                        let ps = pc + 1;
                        let prefix = i.operand_size_prefix as usize;
                        let ol = match prefix {
                            1 => t.script.get(ps).copied().unwrap_or(0) as usize,
                            2 => {
                                let a = t.script.get(ps).copied().unwrap_or(0);
                                let b = t.script.get(ps + 1).copied().unwrap_or(0);
                                u16::from_le_bytes([a, b]) as usize
                            }
                            _ => 0,
                        };
                        1 + prefix + ol
                    };
                    (i.name, s)
                }
                None => ("???", 1),
            };
            let hex: String = t.script[pc..pc + size.min(t.script.len() - pc)]
                .iter()
                .map(|b| format!("{b:02X}"))
                .collect::<Vec<_>>()
                .join(" ");
            eprintln!("  {pc:4}: {hex:20} {iname}");
            pc += size;
        }
    }
}

#[test]
fn nef_size_memory_detail() {
    let wasm = wat::parse_str(
        r#"(module (memory 1)
            (func (export "load") (result i32) i32.const 0 i32.load)
            (func (export "store") (param i32 i32) local.get 0 local.get 1 i32.store))"#,
    ).expect("valid wat");
    let t = translate_module(&wasm, "memory_load_store").expect("translate");
    let table = build_opcode_table();
    eprintln!("\n=== memory_load_store detail ({} bytes) ===", t.script.len());
    let mut pc = 0usize;
    while pc < t.script.len() {
        let byte = t.script[pc];
        let info = table[byte as usize];
        let (iname, size) = match info {
            Some(i) => {
                let s = if i.operand_size_prefix == 0 {
                    1 + i.operand_size as usize
                } else {
                    let ps = pc + 1;
                    let prefix = i.operand_size_prefix as usize;
                    let ol = match prefix {
                        1 => t.script.get(ps).copied().unwrap_or(0) as usize,
                        2 => { let a = t.script.get(ps).copied().unwrap_or(0); let b = t.script.get(ps+1).copied().unwrap_or(0); u16::from_le_bytes([a, b]) as usize }
                        _ => 0,
                    };
                    1 + prefix + ol
                };
                (i.name, s)
            }
            None => ("???", 1),
        };
        let hex: String = t.script[pc..pc + size.min(t.script.len() - pc)]
            .iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" ");
        eprintln!("  {pc:4}: {hex:20} {iname}");
        pc += size;
    }
}

#[test]
fn nef_opcode_histogram() {
    let cases = vec![
        ("memory_fill_copy", r#"(module (memory 1)
            (func (export "fill") (param i32 i32 i32)
                local.get 0 local.get 1 local.get 2 memory.fill)
            (func (export "copy") (param i32 i32 i32)
                local.get 0 local.get 1 local.get 2 memory.copy))"#),
    ];
    let table = build_opcode_table();
    for (name, wat) in &cases {
        let wasm = wat::parse_str(wat).expect("valid wat");
        let t = translate_module(&wasm, name).expect("translate");
        let s = &t.script;
        let mut hist: std::collections::BTreeMap<&str, (usize, usize)> = std::collections::BTreeMap::new();
        let mut pc = 0usize;
        while pc < s.len() {
            let info = table[s[pc] as usize];
            let (iname, size) = match info {
                Some(i) => {
                    let sz = if i.operand_size_prefix == 0 { 1 + i.operand_size as usize }
                    else {
                        let ps = pc + 1; let pf = i.operand_size_prefix as usize;
                        let ol = match pf { 1 => s.get(ps).copied().unwrap_or(0) as usize, _ => 0 };
                        1 + pf + ol
                    };
                    (i.name, sz)
                }
                None => ("???", 1),
            };
            let e = hist.entry(iname).or_insert((0, 0));
            e.0 += 1;
            e.1 += size;
            pc += size;
        }
        eprintln!("\n=== {name} opcode histogram ({} bytes) ===", s.len());
        let mut sorted: Vec<_> = hist.into_iter().collect();
        sorted.sort_by(|a, b| b.1.1.cmp(&a.1.1));
        for (op, (count, bytes)) in sorted.iter().take(15) {
            eprintln!("  {op:15} {count:3}x  {bytes:4}B");
        }
    }
}

#[test]
fn nef_multi_function_detail() {
    let wasm = wat::parse_str(r#"(module
        (func $a (param i32) (result i32) local.get 0 i32.const 1 i32.add)
        (func $b (param i32) (result i32) local.get 0 call $a call $a)
        (func (export "main") (result i32) i32.const 5 call $b))"#)
    .expect("valid wat");
    let t = translate_module(&wasm, "multi_function").expect("translate");
    let table = build_opcode_table();
    eprintln!("\n=== multi_function detail ({} bytes) ===", t.script.len());
    let mut pc = 0usize;
    while pc < t.script.len() {
        let byte = t.script[pc];
        let info = table[byte as usize];
        let (iname, size) = match info {
            Some(i) => { let s = if i.operand_size_prefix == 0 { 1 + i.operand_size as usize } else { let ps = pc+1; let pf = i.operand_size_prefix as usize; let ol = match pf { 1 => t.script.get(ps).copied().unwrap_or(0) as usize, _ => 0 }; 1 + pf + ol }; (i.name, s) }
            None => ("???", 1),
        };
        let hex: String = t.script[pc..pc+size.min(t.script.len()-pc)].iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" ");
        eprintln!("  {pc:4}: {hex:20} {iname}");
        pc += size;
    }
}

#[test]
fn nef_factorial_detail() {
    let wasm = wat::parse_str(r#"(module
        (func $fac (param i32) (result i32)
            local.get 0 i32.const 1 i32.le_s
            if (result i32) i32.const 1
            else local.get 0 local.get 0 i32.const 1 i32.sub call $fac i32.mul end)
        (func (export "main") (result i32) i32.const 10 call $fac))"#)
    .expect("valid wat");
    let t = translate_module(&wasm, "recursive_factorial").expect("translate");
    let table = build_opcode_table();
    eprintln!("\n=== recursive_factorial detail ({} bytes) ===", t.script.len());
    let mut pc = 0usize;
    while pc < t.script.len() {
        let byte = t.script[pc];
        let info = table[byte as usize];
        let (iname, size) = match info {
            Some(i) => { let s = if i.operand_size_prefix == 0 { 1 + i.operand_size as usize } else { let ps = pc+1; let pf = i.operand_size_prefix as usize; let ol = match pf { 1 => t.script.get(ps).copied().unwrap_or(0) as usize, _ => 0 }; 1 + pf + ol }; (i.name, s) }
            None => ("???", 1),
        };
        let hex: String = t.script[pc..pc+size.min(t.script.len()-pc)].iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" ");
        eprintln!("  {pc:4}: {hex:20} {iname}");
        pc += size;
    }
}

#[test]
fn nef_if_else_detail() {
    let wasm = wat::parse_str(r#"(module (func (export "max") (param i32 i32) (result i32)
        local.get 0 local.get 1 i32.gt_s
        if (result i32) local.get 0 else local.get 1 end))"#)
    .expect("valid wat");
    let t = translate_module(&wasm, "if_else").expect("translate");
    let table = build_opcode_table();
    eprintln!("\n=== if_else detail ({} bytes) ===", t.script.len());
    let mut pc = 0usize;
    while pc < t.script.len() {
        let byte = t.script[pc];
        let info = table[byte as usize];
        let (iname, size) = match info {
            Some(i) => { let s = if i.operand_size_prefix == 0 { 1 + i.operand_size as usize } else { let ps = pc+1; let pf = i.operand_size_prefix as usize; let ol = match pf { 1 => t.script.get(ps).copied().unwrap_or(0) as usize, _ => 0 }; 1 + pf + ol }; (i.name, s) }
            None => ("???", 1),
        };
        let hex: String = t.script[pc..pc+size.min(t.script.len()-pc)].iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" ");
        eprintln!("  {pc:4}: {hex:20} {iname}");
        pc += size;
    }
}
