use anyhow::{anyhow, bail, Result};

use crate::translator::helpers::*;
use crate::translator::runtime::emit_sign_extend;
use crate::translator::types::StackValue;

mod binary;
mod compare;
mod divrem;
mod shift;

pub(crate) use binary::emit_binary_op;

pub(super) use binary::emit_eqz;
pub(super) use compare::{emit_signed_compare, emit_unsigned_compare, CompareOp};
pub(super) use divrem::{
    emit_abort_on_signed_div_overflow, emit_abort_on_zero_divisor, emit_unsigned_binary_op,
    UnsignedOp,
};
pub(super) use shift::{emit_rotate, emit_shift_right, mask_shift_amount, ShiftKind};
