// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use anyhow::{anyhow, bail, Result};

use crate::translator::helpers::*;
use crate::translator::runtime::emit_sign_extend;
use crate::translator::types::StackValue;

mod binary;
mod compare;
mod divrem;
mod shift;

/// Branch prediction hints (Round 85)
///
/// These macros provide likely/unlikely hints for the compiler's branch predictor.
/// Use `likely!` for branches that are expected to be taken ~90%+ of the time.
/// Use `unlikely!` for branches that are expected to be taken ~10%- of the time.
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

pub(crate) use binary::emit_binary_op;

pub(super) use binary::emit_eqz;
pub(super) use compare::{emit_signed_compare, emit_unsigned_compare, CompareOp};
pub(super) use divrem::{
    emit_abort_on_signed_div_overflow, emit_abort_on_zero_divisor, emit_unsigned_binary_op,
    UnsignedOp,
};
pub(super) use shift::{emit_rotate, emit_shift_right, mask_shift_amount, ShiftKind};
