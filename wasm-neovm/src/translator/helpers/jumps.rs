// Re-export unified offset functions for backward compatibility
pub use super::offsets::{
    emit_jump_to, emit_placeholder as emit_jump_placeholder,
    emit_placeholder_short as emit_jump_placeholder_short, patch_offset as patch_jump,
    patch_offset_short as patch_jump_short,
};

// Kept for backward compatibility - all functionality is now in offsets.rs
// TODO: Migrate all callers to use offsets module directly, then deprecate this module
