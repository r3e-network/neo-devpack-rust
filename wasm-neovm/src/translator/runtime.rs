use std::collections::BTreeMap;

use anyhow::{anyhow, bail, Context, Result};
use wasmparser::{ConstExpr, FuncType, Operator, ValType};

use crate::adapters::ChainAdapter;
use crate::nef::{MethodToken, HASH160_LENGTH};

const BASE_MEMORY_STATIC_SLOTS: usize = 4;
pub(crate) const INIT_FLAG_SLOT: usize = BASE_MEMORY_STATIC_SLOTS;
const BASE_STATIC_SLOTS: usize = INIT_FLAG_SLOT + 1;

use super::constants::*;
use super::helpers::*;
use super::translation::{emit_binary_op, handle_import_call, FeatureTracker};
use super::types::StackValue;
use super::FunctionImport;

mod bits;
mod data;
mod helpers_impl;
mod init;
mod memory;
mod registry;
mod table;
mod tokens;
mod types;

pub(crate) use bits::{emit_bit_count, emit_select, emit_sign_extend, emit_zero_extend};
pub(crate) use memory::{
    ensure_memory_access, evaluate_global_init, evaluate_offset_expr, translate_data_drop,
    translate_memory_copy, translate_memory_fill, translate_memory_init, translate_memory_load,
    translate_memory_store,
};
pub(crate) use registry::FunctionRegistry;
pub(crate) use tokens::infer_contract_tokens;

use types::{
    ActiveSegmentLayout, DataSegmentInfo, DataSegmentKind, ElementSegmentInfo, ElementSegmentKind,
    GlobalDescriptor, GlobalLayout, HelperRecord, MemoryConfig, MemoryHelperKind,
    PassiveElementLayout, PassiveSegmentLayout, TableDescriptor, TableLayout,
};
pub(crate) use types::{BitHelperKind, CallTarget, TableHelperKind, TableInfo};

use bits::{emit_clz_helper, emit_ctz_helper, emit_popcnt_helper};
use data::{emit_data_drop_helper, emit_data_init_helper};
use init::emit_runtime_init_helper;
use memory::{
    emit_env_memcpy_helper, emit_env_memmove_helper, emit_env_memset_helper,
    emit_memory_copy_helper, emit_memory_fill_helper, emit_memory_grow_helper,
    emit_memory_load_helper, emit_memory_store_helper,
};
use table::{
    emit_elem_drop_helper, emit_table_copy_helper, emit_table_fill_helper, emit_table_get_helper,
    emit_table_grow_helper, emit_table_init_from_passive_helper, emit_table_set_helper,
    emit_table_size_helper,
};

#[derive(Default)]
pub(crate) struct RuntimeHelpers {
    memory_init_offset: Option<usize>,
    memory_init_calls: Vec<usize>,
    memory_init_suppressed: bool,
    memory_config: MemoryConfig,
    memory_defined: bool,
    memory_helpers: BTreeMap<MemoryHelperKind, HelperRecord>,
    bit_helpers: BTreeMap<BitHelperKind, HelperRecord>,
    table_helpers: BTreeMap<TableHelperKind, HelperRecord>,
    data_segments: Vec<DataSegmentInfo>,
    element_segments: Vec<ElementSegmentInfo>,
    next_data_index: usize,
    next_element_index: usize,
    globals: Vec<GlobalDescriptor>,
    tables: Vec<TableDescriptor>,
    start_slot: Option<usize>,
    start_call_positions: Vec<usize>,
}

pub(crate) struct StartDescriptor {
    pub(crate) function_index: u32,
    pub(crate) kind: StartKind,
}

pub(crate) enum StartKind {
    Defined { offset: usize },
    Import,
}

pub(crate) struct StartHelper<'a> {
    slot: usize,
    descriptor: &'a StartDescriptor,
}
