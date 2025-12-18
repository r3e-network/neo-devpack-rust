use anyhow::{Context, Result};

use move_neovm::{is_move_bytecode, translate_move_to_wasm};
use wasm_neovm::SourceChain;

pub(crate) fn maybe_translate_move_bytecode(
    bytes: Vec<u8>,
    source_chain: SourceChain,
) -> Result<Vec<u8>> {
    if source_chain == SourceChain::Move && is_move_bytecode(&bytes) {
        let translated =
            translate_move_to_wasm(&bytes, "move-module").context("translate Move bytecode")?;
        Ok(translated.wasm)
    } else {
        Ok(bytes)
    }
}
