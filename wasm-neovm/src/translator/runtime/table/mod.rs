// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::*;

mod access;
mod copy;
mod fill;
mod grow;
mod segments;

pub(super) use access::{emit_table_get_helper, emit_table_set_helper, emit_table_size_helper};
pub(super) use copy::emit_table_copy_helper;
pub(super) use fill::emit_table_fill_helper;
pub(super) use grow::emit_table_grow_helper;
pub(super) use segments::{emit_elem_drop_helper, emit_table_init_from_passive_helper};
