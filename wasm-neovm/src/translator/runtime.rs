// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

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

/// Branch prediction macros (Round 85)
#[allow(unused_macros)]
macro_rules! likely {
    ($e:expr) => {
        $e
    };
}
#[allow(unused_macros)]
macro_rules! unlikely {
    ($e:expr) => {
        $e
    };
}

/// Memory pool for reusable allocations (Round 89)
///
/// Reduces allocation overhead by reusing common buffer sizes.
#[allow(dead_code)]
const POOL_BUCKET_SIZES: [usize; 4] = [256, 1024, 4096, 16384];

/// Thread-local memory pool for translation buffers (Round 89)
#[derive(Default)]
pub struct TranslationMemoryPool {
    buckets: [Vec<Vec<u8>>; 4],
}

impl TranslationMemoryPool {
    /// Get a pooled buffer of at least the requested capacity
    ///
    /// Round 81: Inline hot path
    #[inline]
    pub fn acquire(&mut self, capacity: usize) -> Vec<u8> {
        let bucket_idx = self.bucket_index(capacity);

        // Round 85: Pool hit is likely for common sizes
        if likely!(bucket_idx < self.buckets.len()) {
            if let Some(mut buf) = self.buckets[bucket_idx].pop() {
                buf.clear();
                return buf;
            }
        }

        Vec::with_capacity(capacity)
    }

    /// Return a buffer to the pool
    ///
    /// Round 81: Inline hot path
    #[inline]
    pub fn release(&mut self, buf: Vec<u8>) {
        let bucket_idx = self.bucket_index(buf.capacity());

        if bucket_idx < self.buckets.len() {
            // Round 85: Small buckets are likely to have space
            if likely!(self.buckets[bucket_idx].len() < 16) {
                self.buckets[bucket_idx].push(buf);
            }
        }
    }

    /// Find the appropriate bucket for a capacity
    #[inline(always)]
    fn bucket_index(&self, capacity: usize) -> usize {
        // Round 87: Use bit manipulation for bucket selection
        match capacity {
            0..=256 => 0,
            257..=1024 => 1,
            1025..=4096 => 2,
            4097..=16384 => 3,
            _ => 4, // Too large for pooling
        }
    }
}

mod bits;
mod data;
mod helpers_impl;
mod init;
mod memory;
mod registry;
mod table;
mod tokens;
mod types;

pub(crate) use bits::{
    emit_bit_count, emit_select, emit_sign_extend, emit_sign_extend_via_helper, emit_zero_extend,
};
pub(crate) use helpers_impl::finalize::FinalizeParams;
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
pub(crate) use types::{
    BitHelperKind, CallIndirectHelperKey, CallTarget, TableHelperKind, TableInfo,
};

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

/// Runtime helpers with optimized memory layout (Round 84 - Cache Locality)
///
/// Fields ordered by access frequency:
/// - Hot fields: First cache line (64 bytes)
/// - Warm fields: Second cache line
/// - Cold fields: Subsequent lines
#[derive(Default)]
#[repr(C)]
pub(crate) struct RuntimeHelpers {
    // === Hot fields (frequently accessed during translation) ===
    /// Memory initialization tracking
    memory_init_offset: Option<usize>,
    memory_init_calls: Vec<usize>,
    memory_init_suppressed: bool,
    /// Whether an init guard has been emitted in the current function.
    function_init_emitted: bool,
    memory_defined: bool,

    // Memory helper cache (Round 88: Static dispatch via direct indexing)
    memory_helpers: HashMap<MemoryHelperKind, HelperRecord>,

    // === Warm fields (accessed per memory operation) ===
    memory_config: MemoryConfig,
    bit_helpers: HashMap<BitHelperKind, HelperRecord>,
    /// Shared i32 sign-extension helper: called at each use site instead of inlining.
    sign_extend_32_helper: HelperRecord,
    /// Shared i64 sign-extension helper.
    sign_extend_64_helper: HelperRecord,
    /// Shared i32 parameter normalization helper (null-check + type-check + sign-extend).
    param_normalize_i32_helper: HelperRecord,
    /// Shared i64 parameter normalization helper.
    param_normalize_i64_helper: HelperRecord,
    table_helpers: HashMap<TableHelperKind, HelperRecord>,
    call_indirect_helpers: HashMap<CallIndirectHelperKey, HelperRecord>,

    // === Cold fields (accessed during setup/finalization) ===
    data_segments: Vec<DataSegmentInfo>,
    element_segments: Vec<ElementSegmentInfo>,
    next_data_index: usize,
    next_element_index: usize,
    globals: Vec<GlobalDescriptor>,
    tables: Vec<TableDescriptor>,
    ref_func_constants: BTreeSet<u32>,
    start_slot: Option<usize>,
    start_call_positions: Vec<usize>,

    // Round 89: Memory pool for reusable allocations
    buffer_pool: Option<Arc<std::sync::Mutex<TranslationMemoryPool>>>,
}

impl RuntimeHelpers {
    /// Create with pre-allocated capacities based on expected usage (Rounds 62, 63, 83 optimizations)
    pub(crate) fn with_capacity(
        expected_data_segments: usize,
        expected_element_segments: usize,
        expected_globals: usize,
    ) -> Self {
        Self {
            memory_init_offset: None,
            memory_init_calls: Vec::with_capacity(4),
            memory_init_suppressed: false,
            function_init_emitted: false,
            memory_defined: false,
            // Pre-sized HashMaps to avoid rehashing (Round 63 optimization)
            memory_helpers: HashMap::with_capacity(16),
            memory_config: MemoryConfig::default(),
            bit_helpers: HashMap::with_capacity(8),
            table_helpers: HashMap::with_capacity(8),
            call_indirect_helpers: HashMap::with_capacity(8),
            sign_extend_32_helper: HelperRecord::default(),
            sign_extend_64_helper: HelperRecord::default(),
            param_normalize_i32_helper: HelperRecord::default(),
            param_normalize_i64_helper: HelperRecord::default(),
            data_segments: Vec::with_capacity(expected_data_segments),
            element_segments: Vec::with_capacity(expected_element_segments),
            next_data_index: 0,
            next_element_index: 0,
            globals: Vec::with_capacity(expected_globals),
            tables: Vec::with_capacity(4),
            ref_func_constants: BTreeSet::new(),
            start_slot: None,
            start_call_positions: Vec::with_capacity(2),
            buffer_pool: None,
        }
    }

    /// Enable memory pooling for this runtime (Round 89)
    #[allow(dead_code)]
    pub(crate) fn with_memory_pool(
        mut self,
        pool: Arc<std::sync::Mutex<TranslationMemoryPool>>,
    ) -> Self {
        self.buffer_pool = Some(pool);
        self
    }

    /// Acquire a buffer from the pool or allocate new (Round 89)
    #[inline]
    #[allow(dead_code)]
    pub(crate) fn acquire_buffer(&self, capacity: usize) -> Vec<u8> {
        match &self.buffer_pool {
            Some(pool) => match pool.lock() {
                Ok(mut guard) => guard.acquire(capacity),
                Err(poisoned) => poisoned.into_inner().acquire(capacity),
            },
            None => Vec::with_capacity(capacity),
        }
    }

    /// Return a buffer to the pool (Round 89)
    #[inline]
    #[allow(dead_code)]
    pub(crate) fn release_buffer(&self, buf: Vec<u8>) {
        if let Some(pool) = &self.buffer_pool {
            match pool.lock() {
                Ok(mut guard) => guard.release(buf),
                Err(poisoned) => poisoned.into_inner().release(buf),
            }
        }
    }
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

#[cfg(test)]
mod tests {
    use super::{RuntimeHelpers, TranslationMemoryPool};
    use std::sync::{Arc, Mutex};

    #[test]
    fn acquire_buffer_recovers_from_poisoned_pool_lock() {
        let pool = Arc::new(Mutex::new(TranslationMemoryPool::default()));
        let _ = std::panic::catch_unwind({
            let pool = Arc::clone(&pool);
            move || {
                let _guard = pool.lock().unwrap();
                panic!("poison pool lock");
            }
        });

        let helpers = RuntimeHelpers::with_capacity(0, 0, 0).with_memory_pool(Arc::clone(&pool));
        let buffer = helpers.acquire_buffer(256);
        assert!(buffer.capacity() >= 256);

        helpers.release_buffer(buffer);
    }
}
