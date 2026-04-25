// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

// Required imports
use anyhow::{anyhow, bail, Context, Result};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use wasmparser::{
    CompositeInnerType, DataKind, ExternalKind, FuncType, HeapType, Operator, Parser, Payload,
    RefType, TypeRef, ValType,
};

use crate::adapters::{get_adapter, ChainAdapter};
use crate::manifest::{merge_manifest, ManifestBuilder, ManifestMethod, ManifestParameter};
use crate::metadata::{
    dedup_method_tokens, extract_nef_metadata, parse_method_token_section, update_manifest_metadata,
};
use crate::nef::MethodToken;
use crate::numeric;

use super::constants::*;
use super::helpers::*;
use super::runtime::{
    emit_bit_count, emit_select, emit_sign_extend, emit_sign_extend_via_helper, emit_zero_extend,
    ensure_memory_access, evaluate_global_init, evaluate_offset_expr, infer_contract_tokens,
    translate_data_drop, translate_memory_copy, translate_memory_fill, translate_memory_init,
    translate_memory_load, translate_memory_store, BitHelperKind, FunctionRegistry, RuntimeHelpers,
    StartDescriptor, StartKind, TableHelperKind, TableInfo,
};
use super::types::{StackValue, Translation, TranslationConfig};
use super::{FunctionImport, ModuleFrontend};

mod control;
mod driver;
mod features;
mod function;
mod imports;
mod locals;
mod ops;
mod wasm_utils;

pub(super) use features::FeatureTracker;
pub(crate) use imports::handle_import_call;
pub(crate) use ops::emit_binary_op;

pub use driver::{translate_module, translate_with_config};

use control::{block_result_count, handle_br_table, handle_branch, ControlFrame, ControlKind};
use features::register_import_features;
use imports::{get_import_type_index, try_handle_env_import, try_handle_neo_import};
use locals::{
    emit_load_arg, emit_local_get, emit_local_set, emit_store_arg, LocalKind, LocalState,
};
use ops::{
    emit_abort_on_signed_div_overflow, emit_abort_on_zero_divisor, emit_eqz, emit_rotate,
    emit_shift_right, emit_signed_compare, emit_unsigned_binary_op, emit_unsigned_compare,
    mask_shift_amount, CompareOp, ShiftKind, UnsignedOp,
};
use wasm_utils::{
    describe_float_op, describe_simd_op, ensure_select_type_supported, wasm_val_type_to_manifest,
};

fn normalize_exported_manifest_signature(
    method_name: &str,
    parameters: Vec<ManifestParameter>,
    return_type: String,
) -> (Vec<ManifestParameter>, String) {
    if method_name.eq_ignore_ascii_case("_deploy") {
        return (
            vec![
                ManifestParameter {
                    name: "data".to_string(),
                    kind: "Any".to_string(),
                },
                ManifestParameter {
                    name: "update".to_string(),
                    kind: "Boolean".to_string(),
                },
            ],
            "Void".to_string(),
        );
    }

    (parameters, return_type)
}
