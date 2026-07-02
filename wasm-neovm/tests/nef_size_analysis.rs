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
            0x22 | 0x24 | 0x26 | 0x28 | 0x2A | 0x2C | 0x2E | 0x30 | 0x32 | 0x3D => jumps_s += 1,
            0x23 | 0x25 | 0x27 | 0x29 | 0x2B | 0x2D | 0x2F | 0x31 | 0x33 | 0x3E => jumps_l += 1,
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

/// Sample contracts shared by the size report and byte-attribution tests.
const SAMPLE_CONTRACTS: &[(&str, &str)] = &[
    (
        "simple_add",
        r#"(module (func (export "add") (param i32 i32) (result i32)
            local.get 0 local.get 1 i32.add))"#,
    ),
    (
        "if_else",
        r#"(module (func (export "max") (param i32 i32) (result i32)
            local.get 0 local.get 1 i32.gt_s
            if (result i32) local.get 0 else local.get 1 end))"#,
    ),
    (
        "memory_load_store",
        r#"(module (memory 1)
            (func (export "load") (result i32) i32.const 0 i32.load)
            (func (export "store") (param i32 i32) local.get 0 local.get 1 i32.store))"#,
    ),
    (
        "br_table_4",
        r#"(module (func (export "dispatch") (param i32) (result i32)
            block $b3 block $b2 block $b1 block $b0
                local.get 0 br_table $b0 $b1 $b2 $b3
            end i32.const 10 return
            end i32.const 20 return
            end i32.const 30 return
            end i32.const 40))"#,
    ),
    (
        "recursive_factorial",
        r#"(module
            (func $fac (param i32) (result i32)
                local.get 0 i32.const 1 i32.le_s
                if (result i32) i32.const 1
                else local.get 0 local.get 0 i32.const 1 i32.sub call $fac i32.mul end)
            (func (export "main") (result i32) i32.const 10 call $fac))"#,
    ),
    (
        "memory_fill_copy",
        r#"(module (memory 1)
            (func (export "fill") (param i32 i32 i32)
                local.get 0 local.get 1 local.get 2 memory.fill)
            (func (export "copy") (param i32 i32 i32)
                local.get 0 local.get 1 local.get 2 memory.copy))"#,
    ),
    (
        "globals",
        r#"(module (global $g (mut i32) (i32.const 0))
            (func (export "set") (param i32) local.get 0 global.set $g)
            (func (export "get") (result i32) global.get $g))"#,
    ),
    (
        "multi_function",
        r#"(module
            (func $a (param i32) (result i32) local.get 0 i32.const 1 i32.add)
            (func $b (param i32) (result i32) local.get 0 call $a call $a)
            (func (export "main") (result i32) i32.const 5 call $b))"#,
    ),
];

#[test]
fn nef_size_report() {
    eprintln!("\n=== NEF Script Size Report ===\n");

    let total: usize = SAMPLE_CONTRACTS
        .iter()
        .map(|(name, wat)| analyze(name, wat))
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
        (
            "br_table_4",
            r#"(module (func (export "dispatch") (param i32) (result i32)
            block $b3 block $b2 block $b1 block $b0
                local.get 0 br_table $b0 $b1 $b2 $b3
            end i32.const 10 return
            end i32.const 20 return
            end i32.const 30 return
            end i32.const 40))"#,
        ),
        (
            "globals",
            r#"(module (global $g (mut i32) (i32.const 0))
            (func (export "set") (param i32) local.get 0 global.set $g)
            (func (export "get") (result i32) global.get $g))"#,
        ),
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
    )
    .expect("valid wat");
    let t = translate_module(&wasm, "memory_load_store").expect("translate");
    let table = build_opcode_table();
    eprintln!(
        "\n=== memory_load_store detail ({} bytes) ===",
        t.script.len()
    );
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

#[test]
fn nef_opcode_histogram() {
    let cases = vec![(
        "memory_fill_copy",
        r#"(module (memory 1)
            (func (export "fill") (param i32 i32 i32)
                local.get 0 local.get 1 local.get 2 memory.fill)
            (func (export "copy") (param i32 i32 i32)
                local.get 0 local.get 1 local.get 2 memory.copy))"#,
    )];
    let table = build_opcode_table();
    for (name, wat) in &cases {
        let wasm = wat::parse_str(wat).expect("valid wat");
        let t = translate_module(&wasm, name).expect("translate");
        let s = &t.script;
        let mut hist: std::collections::BTreeMap<&str, (usize, usize)> =
            std::collections::BTreeMap::new();
        let mut pc = 0usize;
        while pc < s.len() {
            let info = table[s[pc] as usize];
            let (iname, size) = match info {
                Some(i) => {
                    let sz = if i.operand_size_prefix == 0 {
                        1 + i.operand_size as usize
                    } else {
                        let ps = pc + 1;
                        let pf = i.operand_size_prefix as usize;
                        let ol = match pf {
                            1 => s.get(ps).copied().unwrap_or(0) as usize,
                            _ => 0,
                        };
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
        sorted.sort_by_key(|(_, (_, bytes))| std::cmp::Reverse(*bytes));
        for (op, (count, bytes)) in sorted.iter().take(15) {
            eprintln!("  {op:15} {count:3}x  {bytes:4}B");
        }
    }
}

#[test]
fn nef_multi_function_detail() {
    let wasm = wat::parse_str(
        r#"(module
        (func $a (param i32) (result i32) local.get 0 i32.const 1 i32.add)
        (func $b (param i32) (result i32) local.get 0 call $a call $a)
        (func (export "main") (result i32) i32.const 5 call $b))"#,
    )
    .expect("valid wat");
    let t = translate_module(&wasm, "multi_function").expect("translate");
    let table = build_opcode_table();
    eprintln!("\n=== multi_function detail ({} bytes) ===", t.script.len());
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
                    let pf = i.operand_size_prefix as usize;
                    let ol = match pf {
                        1 => t.script.get(ps).copied().unwrap_or(0) as usize,
                        _ => 0,
                    };
                    1 + pf + ol
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

#[test]
fn nef_factorial_detail() {
    let wasm = wat::parse_str(
        r#"(module
        (func $fac (param i32) (result i32)
            local.get 0 i32.const 1 i32.le_s
            if (result i32) i32.const 1
            else local.get 0 local.get 0 i32.const 1 i32.sub call $fac i32.mul end)
        (func (export "main") (result i32) i32.const 10 call $fac))"#,
    )
    .expect("valid wat");
    let t = translate_module(&wasm, "recursive_factorial").expect("translate");
    let table = build_opcode_table();
    eprintln!(
        "\n=== recursive_factorial detail ({} bytes) ===",
        t.script.len()
    );
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
                    let pf = i.operand_size_prefix as usize;
                    let ol = match pf {
                        1 => t.script.get(ps).copied().unwrap_or(0) as usize,
                        _ => 0,
                    };
                    1 + pf + ol
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

#[test]
fn nef_if_else_detail() {
    let wasm = wat::parse_str(
        r#"(module (func (export "max") (param i32 i32) (result i32)
        local.get 0 local.get 1 i32.gt_s
        if (result i32) local.get 0 else local.get 1 end))"#,
    )
    .expect("valid wat");
    let t = translate_module(&wasm, "if_else").expect("translate");
    let table = build_opcode_table();
    eprintln!("\n=== if_else detail ({} bytes) ===", t.script.len());
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
                    let pf = i.operand_size_prefix as usize;
                    let ol = match pf {
                        1 => t.script.get(ps).copied().unwrap_or(0) as usize,
                        _ => 0,
                    };
                    1 + pf + ol
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

// ---------------------------------------------------------------------------
// Byte attribution: decode each translated script and attribute every byte to
// a structural category so NEF-size-reduction work can be measured, not
// guessed. Categories are derived from what the script + manifest expose:
//   - INITSLOT prologue      per-method INITSLOT instruction
//   - arg normalization      LDARGn/CALL/STARGn triples right after INITSLOT
//   - user code              bytes in regions starting at a manifest offset
//   - helpers/internal fns   bytes in regions only reachable via CALL/CALL_L
//                            (runtime helpers + non-exported user functions)
//   - entry/init/dispatch    bytes before the first method/helper boundary
//   - data (PUSHDATA)        PUSHDATA1/2/4 instructions incl. payload
//   - syscalls               SYSCALL instructions
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
struct DecodedIns {
    pc: usize,
    byte: u8,
    name: &'static str,
    size: usize,
}

/// Linearly decode a script into instructions (opcode + operand sizes).
fn decode_script(
    script: &[u8],
    table: &[Option<&'static wasm_neovm::opcodes::OpcodeInfo>; 256],
) -> Vec<DecodedIns> {
    let mut out = Vec::new();
    let mut pc = 0usize;
    while pc < script.len() {
        let byte = script[pc];
        let (name, size) = match table[byte as usize] {
            Some(i) => {
                let sz = if i.operand_size_prefix == 0 {
                    1 + i.operand_size as usize
                } else {
                    let ps = pc + 1;
                    let pf = i.operand_size_prefix as usize;
                    let mut operand_len = 0usize;
                    for k in 0..pf {
                        operand_len |=
                            (script.get(ps + k).copied().unwrap_or(0) as usize) << (8 * k);
                    }
                    1 + pf + operand_len
                };
                (i.name, sz)
            }
            None => ("???", 1),
        };
        let size = size.min(script.len() - pc).max(1);
        out.push(DecodedIns {
            pc,
            byte,
            name,
            size,
        });
        pc += size;
    }
    out
}

/// Absolute targets of every CALL/CALL_L instruction in the script.
fn call_targets(script: &[u8], instructions: &[DecodedIns]) -> std::collections::BTreeSet<usize> {
    let mut targets = std::collections::BTreeSet::new();
    for ins in instructions {
        let rel: Option<i64> = match ins.byte {
            // CALL: signed 8-bit offset relative to the instruction start.
            0x34 => script.get(ins.pc + 1).map(|&b| b as i8 as i64),
            // CALL_L: signed 32-bit little-endian offset.
            0x35 if ins.pc + 5 <= script.len() => {
                let mut le = [0u8; 4];
                le.copy_from_slice(&script[ins.pc + 1..ins.pc + 5]);
                Some(i32::from_le_bytes(le) as i64)
            }
            _ => None,
        };
        if let Some(rel) = rel {
            let target = ins.pc as i64 + rel;
            if (0..script.len() as i64).contains(&target) {
                targets.insert(target as usize);
            }
        }
    }
    targets
}

/// Method entry offsets as published in the translated manifest ABI.
fn manifest_method_offsets(manifest: &serde_json::Value) -> std::collections::BTreeSet<usize> {
    manifest
        .get("abi")
        .and_then(|abi| abi.get("methods"))
        .and_then(serde_json::Value::as_array)
        .map(|methods| {
            methods
                .iter()
                .filter_map(|m| m.get("offset").and_then(serde_json::Value::as_u64))
                .map(|offset| offset as usize)
                .collect()
        })
        .unwrap_or_default()
}

/// Category order used by the attribution report columns.
const ATTRIBUTION_CATEGORIES: &[(&str, &str)] = &[
    ("INITSLOT prologue", "initslot"),
    ("arg normalization", "argnorm"),
    ("user code", "user"),
    ("helpers/internal fns", "helpers"),
    ("entry/init/dispatch", "entry"),
    ("data (PUSHDATA)", "data"),
    ("syscalls", "syscall"),
];

/// Attribute every byte of the translated script to a category.
fn attribute_bytes(name: &str, wat: &str) -> std::collections::BTreeMap<&'static str, usize> {
    let wasm = wat::parse_str(wat).expect("valid wat");
    let t = translate_module(&wasm, name).expect("translate");
    let table = build_opcode_table();
    let instructions = decode_script(&t.script, &table);
    let calls = call_targets(&t.script, &instructions);
    let methods = manifest_method_offsets(&t.manifest.value);

    // Region boundaries: every method entry and every call target starts a
    // new region. Bytes before the first boundary are entry/init/dispatch.
    let mut boundaries: Vec<usize> = methods.iter().chain(calls.iter()).copied().collect();
    boundaries.sort_unstable();
    boundaries.dedup();

    // Mark the per-arg normalization runs the translator emits right after
    // each INITSLOT: repeated (LDARGn, CALL <norm helper>, STARGn) triples.
    let mut argnorm = vec![false; instructions.len()];
    let mut i = 0usize;
    while i < instructions.len() {
        if instructions[i].name == "INITSLOT" {
            let mut j = i + 1;
            while j + 2 < instructions.len()
                && instructions[j].name.starts_with("LDARG")
                && instructions[j + 1].name.starts_with("CALL")
                && instructions[j + 2].name.starts_with("STARG")
            {
                argnorm[j] = true;
                argnorm[j + 1] = true;
                argnorm[j + 2] = true;
                j += 3;
            }
            i = j;
        } else {
            i += 1;
        }
    }

    let mut by_category: std::collections::BTreeMap<&'static str, usize> =
        std::collections::BTreeMap::new();
    for (idx, ins) in instructions.iter().enumerate() {
        let region_start = boundaries.iter().rev().find(|&&b| b <= ins.pc).copied();
        let category = if ins.name == "INITSLOT" {
            "INITSLOT prologue"
        } else if argnorm[idx] {
            "arg normalization"
        } else if ins.name.starts_with("PUSHDATA") {
            "data (PUSHDATA)"
        } else if ins.name == "SYSCALL" {
            "syscalls"
        } else {
            match region_start {
                Some(start) if methods.contains(&start) => "user code",
                Some(_) => "helpers/internal fns",
                None => "entry/init/dispatch",
            }
        };
        *by_category.entry(category).or_insert(0) += ins.size;
    }

    let attributed: usize = by_category.values().sum();
    assert_eq!(
        attributed,
        t.script.len(),
        "{name}: attributed bytes must cover the whole script"
    );
    by_category
}

#[test]
fn nef_byte_attribution_report() {
    eprintln!("\n=== NEF Byte Attribution Report ===\n");

    let mut header = format!("  {:22} {:>6}", "contract", "total");
    for (_, short) in ATTRIBUTION_CATEGORIES {
        header.push_str(&format!(" {short:>8}"));
    }
    eprintln!("{header}");

    let mut totals: std::collections::BTreeMap<&'static str, usize> =
        std::collections::BTreeMap::new();
    let mut grand_total = 0usize;
    for (name, wat) in SAMPLE_CONTRACTS {
        let by_category = attribute_bytes(name, wat);
        let contract_total: usize = by_category.values().sum();
        grand_total += contract_total;
        let mut row = format!("  {name:22} {contract_total:>6}");
        for (category, _) in ATTRIBUTION_CATEGORIES {
            let bytes = by_category.get(category).copied().unwrap_or(0);
            row.push_str(&format!(" {bytes:>8}"));
            *totals.entry(category).or_insert(0) += bytes;
        }
        eprintln!("{row}");
    }

    let mut total_row = format!("  {:22} {:>6}", "TOTAL", grand_total);
    for (category, _) in ATTRIBUTION_CATEGORIES {
        total_row.push_str(&format!(
            " {:>8}",
            totals.get(category).copied().unwrap_or(0)
        ));
    }
    eprintln!("{total_row}");

    // Ranked view: which category dominates across all sample contracts.
    eprintln!("\n  Ranked categories (all samples):");
    let mut ranked: Vec<(&str, usize)> = totals.iter().map(|(c, b)| (*c, *b)).collect();
    ranked.sort_by_key(|(_, bytes)| std::cmp::Reverse(*bytes));
    for (category, bytes) in &ranked {
        let pct = 100.0 * *bytes as f64 / grand_total as f64;
        eprintln!("    {category:22} {bytes:5} B  ({pct:4.1}%)");
    }
    eprintln!("\n=== End Attribution Report ===\n");
}
