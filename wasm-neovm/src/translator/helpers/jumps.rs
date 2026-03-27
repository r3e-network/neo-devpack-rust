// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

// Re-export unified offset functions for backward compatibility
pub use super::offsets::{
    emit_jump_to, emit_placeholder as emit_jump_placeholder, patch_offset as patch_jump,
};
