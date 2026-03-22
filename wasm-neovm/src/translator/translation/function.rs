// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::*;
use crate::opcodes;
use crate::syscalls;

mod op_calls;
mod op_control;
mod op_locals;
mod op_memory;
mod op_numeric;
mod op_refs;
mod op_tables;

/// Context for function translation to reduce parameter count
///
/// This struct groups together all the context needed for translating
/// a WebAssembly function to NeoVM bytecode.
pub struct TranslationContext<'a> {
    pub func_type: &'a FuncType,
    pub body: &'a wasmparser::FunctionBody<'a>,
    pub script: &'a mut Vec<u8>,
    pub imports: &'a [FunctionImport],
    pub types: &'a [FuncType],
    pub func_type_indices: &'a [u32],
    pub runtime: &'a mut RuntimeHelpers,
    pub tables: &'a [TableInfo],
    pub functions: &'a mut FunctionRegistry,
    pub function_index: usize,
    pub start_function: Option<u32>,
    pub function_name: &'a str,
    pub features: &'a mut FeatureTracker,
    pub adapter: &'a dyn ChainAdapter,
}

const ON_NEP17_PAYMENT_CONFIG_SLOT_COUNT: u32 = 1;
const STACKITEMTYPE_ARRAY: u8 = 0x40;
const STACKITEMTYPE_STRUCT: u8 = 0x41;
const STACKITEMTYPE_BYTESTRING: u8 = 0x28;
const ON_NEP17_ADAPTER_BASE: i128 = 1_000_000_000_000;
const ON_NEP17_ADAPTER_TYPE_MULTIPLIER: i128 = 1_000_000_000_000;
const ON_NEP17_ADAPTER_EXPIRY_MULTIPLIER: i128 = 1_000;
const ON_NEP17_INVALID_PACKET_COUNT: i128 = 101;
const GAS_HASH_BE: [u8; 20] = [
    0xd2, 0xa4, 0xcf, 0xf3, 0x19, 0x13, 0x01, 0x61, 0x55, 0xe3, 0x8e, 0x47, 0x4a, 0x2c, 0x06, 0xd0,
    0x8b, 0xe2, 0x76, 0xcf,
];
const GAS_HASH_LE: [u8; 20] = [
    0xcf, 0x76, 0xe2, 0x8b, 0xd0, 0x06, 0x2c, 0x4a, 0x47, 0x8e, 0xe3, 0x55, 0x61, 0x01, 0x13, 0x19,
    0xf3, 0xcf, 0xa4, 0xd2,
];

fn emit_indexed_opcode(script: &mut Vec<u8>, base_opcode: &str, index: u32) -> Result<()> {
    if index <= 6 {
        let indexed_name = format!("{base_opcode}{index}");
        if let Ok(opcode) = lookup_opcode(&indexed_name) {
            script.push(opcode.byte);
            return Ok(());
        }
    }

    let opcode =
        lookup_opcode(base_opcode).map_err(|_| anyhow!("unknown opcode: {base_opcode}"))?;
    if index > u8::MAX as u32 {
        bail!(
            "{} index {} exceeds NeoVM operand limit (0-255)",
            base_opcode,
            index
        );
    }
    script.push(opcode.byte);
    script.push(index as u8);
    Ok(())
}

fn emit_load_local_slot(script: &mut Vec<u8>, slot: u32) -> Result<()> {
    emit_indexed_opcode(script, "LDLOC", slot)
}

fn emit_store_local_slot(script: &mut Vec<u8>, slot: u32) -> Result<()> {
    emit_indexed_opcode(script, "STLOC", slot)
}

// neo-red-envelope-runtime-guards-v3:
// Add entry guards so Rust i64 wrappers preserve critical C# runtime invariants:
// - onNEP17Payment must be called by GAS
// - direct-user methods must be EntryScriptHash-invoked
// - selected account args must satisfy CheckWitness
// Also normalize Null/ByteString parameters into integers before i32/i64 sign extension.
fn emit_descriptor_syscall(script: &mut Vec<u8>, descriptor: &str) -> Result<()> {
    let syscall = syscalls::lookup_extended(descriptor)
        .ok_or_else(|| anyhow!("syscall '{}' not found", descriptor))?;
    let opcode =
        opcodes::lookup("SYSCALL").ok_or_else(|| anyhow!("SYSCALL opcode metadata missing"))?;
    if opcode.operand_size != 4 || opcode.operand_size_prefix != 0 {
        bail!("unexpected SYSCALL operand metadata");
    }
    script.push(opcode.byte);
    script.extend_from_slice(&syscall.hash.to_le_bytes());
    Ok(())
}

fn emit_direct_user_invocation_guard(script: &mut Vec<u8>) -> Result<()> {
    emit_descriptor_syscall(script, "System.Runtime.GetCallingScriptHash")?;
    emit_descriptor_syscall(script, "System.Runtime.GetEntryScriptHash")?;
    script.push(lookup_opcode("EQUAL")?.byte);
    script.push(lookup_opcode("ASSERT")?.byte);
    Ok(())
}

fn emit_witness_guard_for_arg(script: &mut Vec<u8>, arg_index: u32) -> Result<()> {
    emit_load_arg(script, arg_index)?;
    script.push(lookup_opcode("CONVERT")?.byte);
    script.push(STACKITEMTYPE_BYTESTRING);
    emit_descriptor_syscall(script, "System.Runtime.CheckWitness")?;
    script.push(lookup_opcode("ASSERT")?.byte);
    Ok(())
}

fn emit_on_nep17_gas_caller_guard(script: &mut Vec<u8>) -> Result<()> {
    emit_descriptor_syscall(script, "System.Runtime.GetCallingScriptHash")?;
    emit_push_data(script, &GAS_HASH_LE)?;
    script.push(lookup_opcode("EQUAL")?.byte);

    script.push(lookup_opcode("DUP")?.byte);
    let match_little_endian = emit_jump_placeholder(script, "JMPIF_L")?;
    script.push(lookup_opcode("DROP")?.byte);

    emit_descriptor_syscall(script, "System.Runtime.GetCallingScriptHash")?;
    emit_push_data(script, &GAS_HASH_BE)?;
    script.push(lookup_opcode("EQUAL")?.byte);

    let match_little_endian_label = script.len();
    patch_jump(script, match_little_endian, match_little_endian_label)?;

    script.push(lookup_opcode("ASSERT")?.byte);
    Ok(())
}

fn method_name_matches(function_name_lower: &str, exported_name_lower: &str) -> bool {
    function_name_lower == exported_name_lower
        || function_name_lower.ends_with(&format!("::{exported_name_lower}"))
}

fn requires_direct_user_invocation(function_name_lower: &str) -> bool {
    method_name_matches(function_name_lower, "claimfrompool")
        || method_name_matches(function_name_lower, "openclaim")
        || method_name_matches(function_name_lower, "transferclaim")
        || method_name_matches(function_name_lower, "reclaimpool")
        || method_name_matches(function_name_lower, "transfer")
        || method_name_matches(function_name_lower, "openenvelope")
        || method_name_matches(function_name_lower, "transferenvelope")
        || method_name_matches(function_name_lower, "reclaimenvelope")
}

fn witness_guard_arg_index(function_name_lower: &str) -> Option<u32> {
    if method_name_matches(function_name_lower, "claimfrompool")
        || method_name_matches(function_name_lower, "openclaim")
        || method_name_matches(function_name_lower, "reclaimpool")
        || method_name_matches(function_name_lower, "openenvelope")
        || method_name_matches(function_name_lower, "reclaimenvelope")
    {
        return Some(1);
    }

    if method_name_matches(function_name_lower, "transferclaim")
        || method_name_matches(function_name_lower, "transferenvelope")
    {
        return Some(1);
    }

    None
}

// neo-red-envelope-onnep17-object-array-compat:
// Canonicalize onNEP17Payment `data` (arg #2) so Rust handlers using `i64` can safely accept:
// - `null`         -> 0
// - `object[]`     -> adapter integer:
//                     spread => BASE + packetCount + expiryMs * MULTIPLIER
//                     pool   => BASE + TYPE_MULTIPLIER + packetCount + expiryMs * MULTIPLIER
// - `Integer`      -> unchanged (legacy packed-integer path)
fn emit_on_nep17_payment_config_adapter(script: &mut Vec<u8>, base_temp_slot: u32) -> Result<()> {
    let data_slot = base_temp_slot;

    // if data is null -> data = 0, then exit adapter.
    emit_load_arg(script, 2)?;
    script.push(lookup_opcode("ISNULL")?.byte);
    let non_null = emit_jump_placeholder(script, "JMPIFNOT_L")?;
    let _ = emit_push_int(script, 0);
    emit_store_arg(script, 2)?;
    let adapter_end_fixup = emit_jump_placeholder(script, "JMP_L")?;

    let non_null_label = script.len();
    patch_jump(script, non_null, non_null_label)?;

    // if data is neither array nor struct -> keep original value untouched.
    emit_load_arg(script, 2)?;
    script.push(lookup_opcode("ISTYPE")?.byte);
    script.push(STACKITEMTYPE_ARRAY);
    let data_is_seq_fixup = emit_jump_placeholder(script, "JMPIF_L")?;

    emit_load_arg(script, 2)?;
    script.push(lookup_opcode("ISTYPE")?.byte);
    script.push(STACKITEMTYPE_STRUCT);
    let not_array_fixup = emit_jump_placeholder(script, "JMPIFNOT_L")?;

    let data_is_seq_label = script.len();
    patch_jump(script, data_is_seq_fixup, data_is_seq_label)?;

    // temp0 = data(array), packetCount default = 1
    emit_load_arg(script, 2)?;
    emit_store_local_slot(script, data_slot)?;
    let _ = emit_push_int(script, 1);
    emit_store_arg(script, 2)?;

    // object[0] => packetCount (Integer, >0)
    emit_load_local_slot(script, data_slot)?;
    script.push(lookup_opcode("SIZE")?.byte);
    let _ = emit_push_int(script, 0);
    script.push(lookup_opcode("GT")?.byte);
    let skip_packet_parse = emit_jump_placeholder(script, "JMPIFNOT_L")?;

    emit_load_local_slot(script, data_slot)?;
    let _ = emit_push_int(script, 0);
    script.push(lookup_opcode("PICKITEM")?.byte);

    script.push(lookup_opcode("DUP")?.byte);
    script.push(lookup_opcode("ISTYPE")?.byte);
    script.push(STACKITEMTYPE_INTEGER);
    let packet_int_ready_fixup = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(lookup_opcode("DUP")?.byte);
    script.push(lookup_opcode("ISTYPE")?.byte);
    script.push(STACKITEMTYPE_BYTESTRING);
    let packet_drop_fixup = emit_jump_placeholder(script, "JMPIFNOT_L")?;
    script.push(lookup_opcode("CONVERT")?.byte);
    script.push(STACKITEMTYPE_INTEGER);

    let packet_int_ready_label = script.len();
    patch_jump(script, packet_int_ready_fixup, packet_int_ready_label)?;

    script.push(lookup_opcode("DUP")?.byte);
    let _ = emit_push_int(script, 0);
    script.push(lookup_opcode("GT")?.byte);
    let packet_non_positive_fixup = emit_jump_placeholder(script, "JMPIFNOT_L")?;

    emit_store_arg(script, 2)?;
    let packet_done_fixup = emit_jump_placeholder(script, "JMP_L")?;

    let packet_drop_label = script.len();
    patch_jump(script, packet_drop_fixup, packet_drop_label)?;
    patch_jump(script, packet_non_positive_fixup, packet_drop_label)?;
    script.push(lookup_opcode("DROP")?.byte);
    let _ = emit_push_int(script, ON_NEP17_INVALID_PACKET_COUNT);
    emit_store_arg(script, 2)?;

    let packet_done_label = script.len();
    patch_jump(script, skip_packet_parse, packet_done_label)?;
    patch_jump(script, packet_done_fixup, packet_done_label)?;

    // object[1] => expiryMs (Integer, >0). If provided, fold into adapter payload:
    // encoded = packetCount + expiryMs * MULTIPLIER
    emit_load_local_slot(script, data_slot)?;
    script.push(lookup_opcode("SIZE")?.byte);
    let _ = emit_push_int(script, 1);
    script.push(lookup_opcode("GT")?.byte);
    let skip_expiry_parse = emit_jump_placeholder(script, "JMPIFNOT_L")?;

    emit_load_local_slot(script, data_slot)?;
    let _ = emit_push_int(script, 1);
    script.push(lookup_opcode("PICKITEM")?.byte);

    script.push(lookup_opcode("DUP")?.byte);
    script.push(lookup_opcode("ISTYPE")?.byte);
    script.push(STACKITEMTYPE_INTEGER);
    let expiry_int_ready_fixup = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(lookup_opcode("DUP")?.byte);
    script.push(lookup_opcode("ISTYPE")?.byte);
    script.push(STACKITEMTYPE_BYTESTRING);
    let expiry_drop_fixup = emit_jump_placeholder(script, "JMPIFNOT_L")?;
    script.push(lookup_opcode("CONVERT")?.byte);
    script.push(STACKITEMTYPE_INTEGER);

    let expiry_int_ready_label = script.len();
    patch_jump(script, expiry_int_ready_fixup, expiry_int_ready_label)?;

    script.push(lookup_opcode("DUP")?.byte);
    let _ = emit_push_int(script, 0);
    script.push(lookup_opcode("GT")?.byte);
    let expiry_non_positive_fixup = emit_jump_placeholder(script, "JMPIFNOT_L")?;

    let _ = emit_push_int(script, ON_NEP17_ADAPTER_EXPIRY_MULTIPLIER);
    script.push(lookup_opcode("MUL")?.byte);
    emit_load_arg(script, 2)?;
    script.push(lookup_opcode("ADD")?.byte);
    emit_store_arg(script, 2)?;
    let expiry_done_fixup = emit_jump_placeholder(script, "JMP_L")?;

    let expiry_drop_label = script.len();
    patch_jump(script, expiry_drop_fixup, expiry_drop_label)?;
    patch_jump(script, expiry_non_positive_fixup, expiry_drop_label)?;
    script.push(lookup_opcode("DROP")?.byte);
    let expiry_done_label = script.len();
    patch_jump(script, skip_expiry_parse, expiry_done_label)?;
    patch_jump(script, expiry_done_fixup, expiry_done_label)?;

    // Adapter v3 encoding:
    // spread => base + packetCount + expiryMs * MULTIPLIER
    // pool   => base + TYPE_MULTIPLIER + packetCount + expiryMs * MULTIPLIER
    emit_load_arg(script, 2)?;
    let _ = emit_push_int(script, ON_NEP17_ADAPTER_BASE);
    script.push(lookup_opcode("ADD")?.byte);
    emit_store_arg(script, 2)?;

    // envelopeType parse follows C# layout:
    // config[5] when size > 5; otherwise default spreading.
    emit_load_local_slot(script, data_slot)?;
    script.push(lookup_opcode("SIZE")?.byte);
    let _ = emit_push_int(script, 5);
    script.push(lookup_opcode("GT")?.byte);
    let skip_type_parse = emit_jump_placeholder(script, "JMPIFNOT_L")?;

    emit_load_local_slot(script, data_slot)?;
    let _ = emit_push_int(script, 5);
    script.push(lookup_opcode("PICKITEM")?.byte);

    script.push(lookup_opcode("DUP")?.byte);
    script.push(lookup_opcode("ISTYPE")?.byte);
    script.push(STACKITEMTYPE_INTEGER);
    let type_int_ready_fixup = emit_jump_placeholder(script, "JMPIF_L")?;

    script.push(lookup_opcode("DUP")?.byte);
    script.push(lookup_opcode("ISTYPE")?.byte);
    script.push(STACKITEMTYPE_BYTESTRING);
    let type_drop_fixup = emit_jump_placeholder(script, "JMPIFNOT_L")?;
    script.push(lookup_opcode("CONVERT")?.byte);
    script.push(STACKITEMTYPE_INTEGER);

    let type_int_ready_label = script.len();
    patch_jump(script, type_int_ready_fixup, type_int_ready_label)?;

    // type == 1 => pool (add TYPE_MULTIPLIER marker)
    script.push(lookup_opcode("DUP")?.byte);
    let _ = emit_push_int(script, 1);
    script.push(lookup_opcode("EQUAL")?.byte);
    let type_is_pool_fixup = emit_jump_placeholder(script, "JMPIF_L")?;

    // type == 0 => spreading (keep positive)
    script.push(lookup_opcode("DUP")?.byte);
    let _ = emit_push_int(script, 0);
    script.push(lookup_opcode("EQUAL")?.byte);
    let type_is_spread_fixup = emit_jump_placeholder(script, "JMPIF_L")?;

    // Invalid type value => force invalid packet sentinel.
    script.push(lookup_opcode("DROP")?.byte);
    let _ = emit_push_int(
        script,
        ON_NEP17_ADAPTER_BASE + ON_NEP17_INVALID_PACKET_COUNT,
    );
    emit_store_arg(script, 2)?;
    let type_invalid_done_fixup = emit_jump_placeholder(script, "JMP_L")?;

    let type_is_pool_label = script.len();
    patch_jump(script, type_is_pool_fixup, type_is_pool_label)?;
    script.push(lookup_opcode("DROP")?.byte);
    emit_load_arg(script, 2)?;
    let _ = emit_push_int(script, ON_NEP17_ADAPTER_TYPE_MULTIPLIER);
    script.push(lookup_opcode("ADD")?.byte);
    emit_store_arg(script, 2)?;
    let type_pool_done_fixup = emit_jump_placeholder(script, "JMP_L")?;

    let type_is_spread_label = script.len();
    patch_jump(script, type_is_spread_fixup, type_is_spread_label)?;
    script.push(lookup_opcode("DROP")?.byte);
    let type_spread_done_fixup = emit_jump_placeholder(script, "JMP_L")?;

    // Non-int/non-bytes type => force invalid sentinel.
    let type_drop_label = script.len();
    patch_jump(script, type_drop_fixup, type_drop_label)?;
    script.push(lookup_opcode("DROP")?.byte);
    let _ = emit_push_int(
        script,
        ON_NEP17_ADAPTER_BASE + ON_NEP17_INVALID_PACKET_COUNT,
    );
    emit_store_arg(script, 2)?;
    let type_drop_done_fixup = emit_jump_placeholder(script, "JMP_L")?;

    let type_done_label = script.len();
    patch_jump(script, skip_type_parse, type_done_label)?;
    patch_jump(script, type_invalid_done_fixup, type_done_label)?;
    patch_jump(script, type_pool_done_fixup, type_done_label)?;
    patch_jump(script, type_spread_done_fixup, type_done_label)?;
    patch_jump(script, type_drop_done_fixup, type_done_label)?;

    let adapter_end_label = script.len();
    patch_jump(script, not_array_fixup, adapter_end_label)?;
    patch_jump(script, adapter_end_fixup, adapter_end_label)?;

    Ok(())
}

pub(super) fn translate_function(ctx: &mut TranslationContext<'_>) -> Result<String> {
    let params = ctx.func_type.params();
    for ty in params {
        match ty {
            ValType::I32 | ValType::I64 => {}
            other => bail!("only i32/i64 parameters are supported (found {:?})", other),
        }
    }
    let param_count = params.len();

    let returns = ctx.func_type.results();
    if returns.len() > 1 {
        bail!("multi-value returns are not supported");
    }

    if let Some(start_idx) = ctx.start_function {
        if start_idx as usize != ctx.function_index {
            ctx.runtime.emit_memory_init_call(ctx.script)?;
        }
    }

    let return_kind = returns.first().map(wasm_val_type_to_manifest).transpose()?;

    let locals_reader = ctx.body.get_locals_reader()?;
    let mut local_states: Vec<LocalState> = Vec::new();
    for i in 0..param_count {
        local_states.push(LocalState {
            kind: LocalKind::Param(i as u32),
            const_value: None,
        });
    }

    let mut local_count: u32 = 0;
    for entry in ctx.body.get_locals_reader()? {
        let (count, ty) = entry?;
        if ty != ValType::I32 && ty != ValType::I64 {
            bail!("only i32/i64 locals are supported (found {:?})", ty);
        }
        local_count = local_count
            .checked_add(count)
            .ok_or_else(|| anyhow!("function {} local count overflow", ctx.function_name))?;
    }

    let function_name_lower = ctx.function_name.to_ascii_lowercase();
    let is_on_nep17_payment = function_name_lower.contains("onnep17payment")
        || function_name_lower.contains("on_nep17_payment");
    let is_deploy_entry =
        function_name_lower == "_deploy" || function_name_lower.ends_with("::_deploy");
    let is_check_witness_probe = function_name_lower.contains("debug_check_witness")
        || function_name_lower.contains("debugcheckwitness");

    let use_on_nep17_adapter = is_on_nep17_payment && param_count >= 3;
    let helper_local_base = local_count;
    if use_on_nep17_adapter {
        local_count = local_count
            .checked_add(ON_NEP17_PAYMENT_CONFIG_SLOT_COUNT)
            .ok_or_else(|| anyhow!("function {} local count overflow", ctx.function_name))?;
    }

    if param_count > u8::MAX as usize {
        bail!(
            "function {} has too many parameters ({}) for NeoVM INITSLOT",
            ctx.function_name,
            param_count
        );
    }
    if local_count > u8::MAX as u32 {
        bail!(
            "function {} has too many locals ({}) for NeoVM INITSLOT",
            ctx.function_name,
            local_count
        );
    }

    if local_count > 0 || param_count > 0 {
        ctx.script.push(lookup_opcode("INITSLOT")?.byte);
        ctx.script.push(local_count as u8);
        ctx.script.push(param_count as u8);
    }

    if is_on_nep17_payment {
        emit_on_nep17_gas_caller_guard(ctx.script)?;
    }

    if requires_direct_user_invocation(&function_name_lower) {
        emit_direct_user_invocation_guard(ctx.script)?;
    }

    if let Some(arg_index) = witness_guard_arg_index(&function_name_lower) {
        if (arg_index as usize) >= param_count {
            bail!(
                "function {} requires witness guard on arg {} but has only {} parameter(s)",
                ctx.function_name,
                arg_index,
                param_count
            );
        }
        emit_witness_guard_for_arg(ctx.script, arg_index)?;
    }

    if use_on_nep17_adapter {
        emit_on_nep17_payment_config_adapter(ctx.script, helper_local_base)?;
    }

    // NeoVM parameters are arbitrary-precision integers. Normalise them to the Wasm bit-width
    // so arithmetic, comparisons, and shifts observe WebAssembly's i32/i64 semantics.
    //
    // Some Neo entry points carry non-integer stack items (`Any`/`Hash160`) in practice.
    // For those methods, integer coercion can fault before contract logic runs.
    let skip_param_normalization = is_deploy_entry || is_on_nep17_payment || is_check_witness_probe;

    if !skip_param_normalization {
        for (index, ty) in params.iter().enumerate() {
            emit_load_arg(ctx.script, index as u32)?;

            // Neo entry wrappers may pass `Hash160`/`ByteString`/`Any`.
            // Canonicalize those into integers before Wasm i32/i64 sign extension.
            ctx.script.push(lookup_opcode("DUP")?.byte);
            ctx.script.push(lookup_opcode("ISNULL")?.byte);
            let not_null_fixup = emit_jump_placeholder(ctx.script, "JMPIFNOT_L")?;
            ctx.script.push(lookup_opcode("DROP")?.byte);
            let _ = emit_push_int(ctx.script, 0);
            let null_done_fixup = emit_jump_placeholder(ctx.script, "JMP_L")?;

            let not_null_label = ctx.script.len();
            patch_jump(ctx.script, not_null_fixup, not_null_label)?;

            ctx.script.push(lookup_opcode("DUP")?.byte);
            ctx.script.push(lookup_opcode("ISTYPE")?.byte);
            ctx.script.push(STACKITEMTYPE_BYTESTRING);
            let not_bytes_fixup = emit_jump_placeholder(ctx.script, "JMPIFNOT_L")?;
            ctx.script.push(lookup_opcode("CONVERT")?.byte);
            ctx.script.push(STACKITEMTYPE_INTEGER);

            let not_bytes_label = ctx.script.len();
            patch_jump(ctx.script, not_bytes_fixup, not_bytes_label)?;

            let null_done_label = ctx.script.len();
            patch_jump(ctx.script, null_done_fixup, null_done_label)?;

            let value = StackValue {
                const_value: None,
                bytecode_start: None,
            };
            match ty {
                ValType::I32 => {
                    let _ = emit_sign_extend(ctx.script, value, 32, 32)?;
                }
                ValType::I64 => {
                    let _ = emit_sign_extend(ctx.script, value, 64, 64)?;
                }
                _ => unreachable!("parameter types validated earlier"),
            }
            emit_store_arg(ctx.script, index as u32)?;
        }
    }

    let mut next_local_slot: u32 = 0;
    for entry in locals_reader {
        let (count, ty) = entry?;
        if ty != ValType::I32 && ty != ValType::I64 {
            bail!("only i32/i64 locals are supported (found {:?})", ty);
        }
        for _ in 0..count {
            local_states.push(LocalState {
                kind: LocalKind::Local(next_local_slot),
                const_value: Some(0),
            });
            next_local_slot += 1;
        }
    }

    // WebAssembly locals are zero-initialized. NeoVM locals default to Null,
    // so materialize explicit zero stores to preserve Wasm semantics.
    for slot in 0..next_local_slot {
        let _ = emit_push_int(ctx.script, 0);
        emit_store_local_slot(ctx.script, slot)?;
    }

    let op_reader = ctx.body.get_operators_reader()?;
    let mut value_stack: Vec<StackValue> = Vec::new();
    let mut control_stack: Vec<ControlFrame> = Vec::new();
    let mut is_unreachable = false;

    // Push implicit function-level control frame
    // In WASM, the function body itself is an implicit block that can be targeted by branches
    // stack_height is 0 because branches to the function can occur at any point
    // result_count tracks how many values must be on stack when branching to function exit
    control_stack.push(ControlFrame {
        kind: ControlKind::Function,
        stack_height: 0,
        result_count: returns.len(), // Function expects return values
        start_offset: ctx.script.len(),
        end_fixups: Vec::new(),
        if_false_fixup: None,
        has_else: false,
        entry_reachable: true,
        end_reachable_from_branch: false,
        if_then_end_reachable: None,
    });

    // Ensure the current function offset is known to callers (already registered before entry).
    // This assertion helps catch internal misuse during development.
    if !ctx.functions.contains_index(ctx.function_index) {
        bail!(
            "function index {} out of range for translation",
            ctx.function_index
        );
    }

    for op in op_reader {
        let op = op?;

        // In WASM, code after an unconditional branch/return/unreachable is unreachable with a
        // polymorphic stack. We still must translate structured control operators to keep the
        // control stack balanced and patch jump fixups, but can skip translating other operators.
        if is_unreachable {
            if op_control::try_handle(
                &op,
                ctx.script,
                ctx.types,
                &mut value_stack,
                &mut control_stack,
                &mut is_unreachable,
            )? {
                continue;
            }
            continue;
        }

        if op_numeric::try_handle(&op, ctx.script, ctx.runtime, &mut value_stack)? {
            continue;
        }

        if op_control::try_handle(
            &op,
            ctx.script,
            ctx.types,
            &mut value_stack,
            &mut control_stack,
            &mut is_unreachable,
        )? {
            continue;
        }

        if op_memory::try_handle(&op, ctx.script, ctx.runtime, &mut value_stack)? {
            continue;
        }

        if op_tables::try_handle(&op, ctx.script, ctx.runtime, &mut value_stack)? {
            continue;
        }

        if op_locals::try_handle(
            &op,
            ctx.script,
            ctx.runtime,
            &mut value_stack,
            &mut local_states,
        )? {
            continue;
        }

        if op_calls::try_handle(
            &op,
            ctx.script,
            ctx.imports,
            ctx.types,
            ctx.func_type_indices,
            ctx.runtime,
            ctx.tables,
            ctx.functions,
            &mut value_stack,
            ctx.features,
            ctx.adapter,
            &mut is_unreachable,
        )? {
            continue;
        }

        if op_refs::try_handle(
            &op,
            ctx.script,
            ctx.imports,
            ctx.func_type_indices,
            ctx.runtime,
            &mut value_stack,
            &mut is_unreachable,
        )? {
            continue;
        }

        if let Some(desc) = describe_float_op(&op) {
            let context = format!("{} in function {}", desc, ctx.function_name);
            return numeric::unsupported_float(&context);
        }
        if let Some(desc) = describe_simd_op(&op) {
            let context = format!("{} in function {}", desc, ctx.function_name);
            return numeric::unsupported_simd(&context);
        }
        bail!(format!(
            "unsupported Wasm operator {:?} ({}).",
            op, UNSUPPORTED_FEATURE_DOC
        ));
    }

    // Always end with an epilogue RET so `br` to the function-level implicit block has a
    // well-defined jump target.
    ctx.script.push(RET);

    if let Some(frame) = control_stack.last() {
        bail!(
            "unclosed block detected at end of function (kind: {:?})",
            frame.kind
        );
    }

    Ok(return_kind.unwrap_or_else(|| "Void".to_string()))
}

fn pop_value(stack: &mut Vec<StackValue>, context: &str) -> Result<StackValue> {
    stack
        .pop()
        .ok_or_else(|| anyhow!("stack underflow while processing {}", context))
}

fn pop_value_maybe_unreachable(
    stack: &mut Vec<StackValue>,
    context: &str,
    is_unreachable: bool,
) -> Result<StackValue> {
    if let Some(value) = stack.pop() {
        return Ok(value);
    }
    if is_unreachable {
        return Ok(StackValue {
            const_value: None,
            bytecode_start: None,
        });
    }
    Err(anyhow!("stack underflow while processing {}", context))
}

fn set_stack_height_polymorphic(stack: &mut Vec<StackValue>, height: usize) {
    while stack.len() < height {
        stack.push(StackValue {
            const_value: None,
            bytecode_start: None,
        });
    }
    stack.truncate(height);
}
