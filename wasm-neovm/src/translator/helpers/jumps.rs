// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

// Re-export unified offset functions for backward compatibility
pub use super::offsets::{
    emit_jump_to, emit_placeholder as emit_jump_placeholder,
    emit_placeholder_short as emit_jump_placeholder_short, patch_offset as patch_jump,
    patch_offset_short as patch_jump_short,
};
