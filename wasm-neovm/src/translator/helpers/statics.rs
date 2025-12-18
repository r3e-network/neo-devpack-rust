use anyhow::Result;

use super::lookup_opcode;

/// Emit a load from a static field slot
pub(crate) fn emit_load_static(script: &mut Vec<u8>, slot: usize) -> Result<()> {
    // NeoVM has optimized opcodes for slots 0-6
    let opcode = match slot {
        0 => "LDSFLD0",
        1 => "LDSFLD1",
        2 => "LDSFLD2",
        3 => "LDSFLD3",
        4 => "LDSFLD4",
        5 => "LDSFLD5",
        6 => "LDSFLD6",
        _ => {
            // For slots >= 7, use LDSFLD with explicit slot index
            script.push(lookup_opcode("LDSFLD")?.byte);
            script.push(slot as u8);
            return Ok(());
        }
    };

    script.push(lookup_opcode(opcode)?.byte);
    Ok(())
}

/// Emit a store to a static field slot
pub(crate) fn emit_store_static(script: &mut Vec<u8>, slot: usize) -> Result<()> {
    // NeoVM has optimized opcodes for slots 0-6
    let opcode = match slot {
        0 => "STSFLD0",
        1 => "STSFLD1",
        2 => "STSFLD2",
        3 => "STSFLD3",
        4 => "STSFLD4",
        5 => "STSFLD5",
        6 => "STSFLD6",
        _ => {
            // For slots >= 7, use STSFLD with explicit slot index
            script.push(lookup_opcode("STSFLD")?.byte);
            script.push(slot as u8);
            return Ok(());
        }
    };

    script.push(lookup_opcode(opcode)?.byte);
    Ok(())
}
