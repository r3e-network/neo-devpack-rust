use super::lowering::{SCRATCH_KEY_OFFSET, SCRATCH_VALUE_OFFSET};
use crate::bytecode::{AbilitySet, MoveModule, MoveOpcode, StructDef, TypeTag};
use anyhow::{anyhow, bail, Result};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use wasm_encoder::{Function, Instruction};

pub(super) fn struct_hash(name: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    name.hash(&mut hasher);
    hasher.finish()
}

pub(super) fn build_struct_lookup(structs: &[StructDef]) -> HashMap<String, AbilitySet> {
    let mut map = HashMap::new();
    for s in structs {
        map.insert(s.name.clone(), s.abilities);
    }
    map
}

pub(super) fn ensure_has_key(def: &StructDef) -> Result<()> {
    if !def.abilities.key {
        bail!(
            "struct {} does not have the 'key' ability required for global operations",
            def.name
        );
    }
    Ok(())
}

pub(super) fn ensure_copy_allowed(
    tag: &TypeTag,
    lookup: &HashMap<String, AbilitySet>,
) -> Result<()> {
    if let TypeTag::Struct(name) = tag {
        if let Some(abilities) = lookup.get(name) {
            if !abilities.copy {
                bail!("copy of resource {} is not allowed", name);
            }
        }
    }
    Ok(())
}

pub(super) fn struct_for_index(module: &MoveModule, idx: u16) -> Result<&StructDef> {
    module
        .structs
        .get(idx as usize)
        .ok_or_else(|| anyhow!("struct index {} out of range", idx))
}

pub(super) fn write_resource_key(func: &mut Function, def: &StructDef, addr_local: u32) {
    let hash = struct_hash(&def.name);
    func.instruction(&Instruction::I32Const(SCRATCH_KEY_OFFSET));
    func.instruction(&Instruction::I64Const(hash as i64));
    func.instruction(&Instruction::I64Store(wasm_encoder::MemArg {
        offset: 0,
        align: 3,
        memory_index: 0,
    }));

    func.instruction(&Instruction::I32Const(SCRATCH_KEY_OFFSET + 8));
    func.instruction(&Instruction::LocalGet(addr_local));
    func.instruction(&Instruction::I64Store(wasm_encoder::MemArg {
        offset: 0,
        align: 3,
        memory_index: 0,
    }));
}

pub(super) fn write_resource_value(func: &mut Function, value_local: u32) {
    func.instruction(&Instruction::I32Const(SCRATCH_VALUE_OFFSET));
    func.instruction(&Instruction::LocalGet(value_local));
    func.instruction(&Instruction::I64Store(wasm_encoder::MemArg {
        offset: 0,
        align: 3,
        memory_index: 0,
    }));
}

pub(super) fn is_resource_opcode(op: &MoveOpcode) -> bool {
    matches!(
        op,
        MoveOpcode::MoveTo(_)
            | MoveOpcode::MoveFrom(_)
            | MoveOpcode::Exists(_)
            | MoveOpcode::BorrowGlobal(_)
            | MoveOpcode::MutBorrowGlobal(_)
    )
}
