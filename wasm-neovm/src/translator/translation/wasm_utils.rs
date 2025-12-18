use anyhow::{bail, Result};
use wasmparser::{Operator, ValType};

use crate::numeric;

pub(super) fn wasm_val_type_to_manifest(ty: &ValType) -> Result<String> {
    let repr = match ty {
        ValType::I32 => "Integer",
        ValType::I64 => "Integer",
        ValType::F32 | ValType::F64 => return numeric::unsupported_float("manifest numeric type"),
        ValType::V128 => return numeric::unsupported_simd("manifest v128"),
        ValType::Ref(_) => return numeric::unsupported_reference_type("manifest reference type"),
    };
    Ok(repr.to_string())
}

pub(super) fn describe_float_op(op: &Operator) -> Option<String> {
    let name = format!("{:?}", op);
    if name.starts_with("F32") || name.starts_with("F64") {
        return Some(name.to_lowercase());
    }
    None
}

pub(super) fn describe_simd_op(op: &Operator) -> Option<String> {
    const PREFIXES: &[&str] = &["I8x16", "I16x8", "I32x4", "I64x2", "F32x4", "F64x2", "V128"];
    let name = format!("{:?}", op);
    if PREFIXES.iter().any(|prefix| name.starts_with(prefix)) {
        return Some(name.to_lowercase());
    }
    None
}

pub(super) fn ensure_select_type_supported(tys: &[ValType]) -> Result<()> {
    if tys.len() > 1 {
        bail!("typed select results with more than one value are not supported");
    }
    for ty in tys {
        match ty {
            ValType::I32 | ValType::I64 => {}
            other => bail!(
                "typed select with unsupported value type {:?}; only i32/i64 are supported",
                other
            ),
        }
    }
    Ok(())
}
