mod calls;
mod jumps;
mod mask;
mod offsets;
mod opcode;
mod push;
mod statics;
mod try_instructions;
mod validate;

pub(crate) use calls::{emit_call_placeholder, patch_call};
pub(crate) use jumps::{
    emit_jump_placeholder, emit_jump_placeholder_short, emit_jump_to, patch_jump, patch_jump_short,
};
pub(crate) use mask::emit_mask_u32;
pub(crate) use opcode::lookup_opcode;
pub(crate) use push::{emit_push_data, emit_push_int};
pub(crate) use statics::{emit_load_static, emit_store_static};
pub(crate) use try_instructions::{
    emit_endtry_placeholder, emit_try_placeholder, patch_endtry, patch_try_catch,
};
pub(crate) use validate::validate_script;
