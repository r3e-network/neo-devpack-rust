// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

mod calls;
mod jumps;
mod mask;
mod offsets;
mod opcode;
mod push;
mod statics;
mod try_instructions;
pub(crate) mod peephole;
pub(crate) mod relax;
mod validate;

pub(crate) use calls::{emit_call_placeholder, emit_call_to, patch_call};
pub(crate) use jumps::{emit_jump_placeholder, emit_jump_to, patch_jump};
pub(crate) use mask::emit_mask_u32;
pub(crate) use opcode::lookup_opcode;
pub(crate) use push::{emit_push_data, emit_push_int};
pub(crate) use statics::{emit_load_static, emit_store_static};
pub(crate) use try_instructions::{
    emit_endtry_placeholder, emit_try_placeholder, patch_endtry, patch_try_catch,
};
pub(crate) use validate::validate_script;
