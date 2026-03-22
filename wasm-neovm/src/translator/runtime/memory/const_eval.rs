// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use super::super::*;

pub(crate) fn evaluate_offset_expr(expr: ConstExpr<'_>) -> Result<i64> {
    let mut reader = expr.get_operators_reader();
    let mut offset: Option<i64> = None;
    while !reader.eof() {
        let op = reader.read()?;
        match op {
            Operator::I32Const { value } => offset = Some(value as i64),
            Operator::I64Const { value } => offset = Some(value),
            Operator::End => break,
            other => {
                bail!(
                    "unsupported instruction {:?} in data segment offset expression",
                    other
                );
            }
        }
    }

    offset.ok_or_else(|| anyhow!("data segment offset expression did not yield a constant"))
}

pub(crate) fn evaluate_global_init(expr: ConstExpr<'_>, value_type: ValType) -> Result<i128> {
    let mut reader = expr.get_operators_reader();
    let mut value: Option<i128> = None;
    while !reader.eof() {
        let op = reader.read()?;
        match op {
            Operator::I32Const { value: v } => {
                value = Some(v as i128);
            }
            Operator::I64Const { value: v } => {
                value = Some(v as i128);
            }
            Operator::End => break,
            other => {
                bail!(
                    "unsupported instruction {:?} in global initialiser expression",
                    other
                );
            }
        }
    }

    let result = value.ok_or_else(|| anyhow!("global initialiser did not yield a constant"))?;
    match value_type {
        ValType::I32 => Ok((result as i32) as i128),
        ValType::I64 => Ok((result as i64) as i128),
        other => bail!(
            "unsupported global value type {:?}; expected i32 or i64",
            other
        ),
    }
}
