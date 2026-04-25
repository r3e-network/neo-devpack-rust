// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

mod access;
mod chunked;
mod const_eval;
mod env;
mod helpers;
mod translate;

pub(crate) use access::ensure_memory_access;
pub(crate) use const_eval::{evaluate_global_init, evaluate_offset_expr};
pub(crate) use translate::{
    translate_data_drop, translate_memory_copy, translate_memory_fill, translate_memory_init,
    translate_memory_load, translate_memory_store,
};

pub(super) use chunked::{
    emit_chunked_copy_literal_to_memory, emit_chunked_load_byte_at_local, emit_chunked_new_page,
    emit_chunked_store_byte_at_local,
};
pub(super) use env::{
    emit_chunked_env_memcpy_helper, emit_chunked_env_memmove_helper,
    emit_chunked_env_memset_helper, emit_env_memcpy_helper, emit_env_memmove_helper,
    emit_env_memset_helper,
};
pub(super) use helpers::{
    emit_chunked_memory_copy_helper, emit_chunked_memory_fill_helper,
    emit_chunked_memory_grow_helper, emit_chunked_memory_load_helper,
    emit_chunked_memory_store_helper, emit_memory_copy_helper, emit_memory_fill_helper,
    emit_memory_grow_helper, emit_memory_load_helper, emit_memory_store_helper,
};
