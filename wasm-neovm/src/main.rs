mod cli;
mod manifest_tools;
mod move_support;
mod paths;

use std::fs;

use anyhow::{Context, Result};
use clap::Parser;
use log::info;
use serde_json::Value;

use crate::cli::Cli;
use crate::manifest_tools::{apply_source_url, compare_manifest};
use crate::move_support::maybe_translate_move_bytecode;
use crate::paths::derive_output_path;
use wasm_neovm::{
    extract_nef_metadata, translate_with_config, write_nef_with_metadata, ManifestOverlay,
    SourceChain, TranslationConfig,
};

fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    // Parse source chain
    let source_chain = cli.parse_source_chain();

    if source_chain != SourceChain::Neo {
        info!("Cross-chain compilation: {:?} -> NeoVM", source_chain);
    }

    let module = fs::read(&cli.input)
        .with_context(|| format!("failed to read input module {}", cli.input.display()))?;

    let module = maybe_translate_move_bytecode(module, source_chain)?;

    let mut config = TranslationConfig::new(&cli.name);
    config = config.with_source_chain(source_chain);
    if let Some(path) = &cli.manifest_overlay {
        let overlay_bytes = fs::read_to_string(path)
            .with_context(|| format!("failed to read manifest overlay {}", path.display()))?;
        let overlay: Value = serde_json::from_str(&overlay_bytes).with_context(|| {
            format!(
                "failed to parse manifest overlay JSON from {}",
                path.display()
            )
        })?;
        config = config.with_manifest_overlay(ManifestOverlay {
            value: overlay,
            label: Some(path.display().to_string()),
        });
    }
    let translation = translate_with_config(&module, config)?;

    let mut manifest_value = translation.manifest.value.clone();
    if let Some(cli_source) = &cli.source_url {
        apply_source_url(&mut manifest_value, cli_source);
    }

    let manifest_string =
        serde_json::to_string_pretty(&manifest_value).context("failed to render manifest JSON")?;

    let metadata = extract_nef_metadata(&manifest_value)?;

    if let Some(compare_path) = &cli.compare_manifest {
        compare_manifest(compare_path, &manifest_value).with_context(|| {
            format!(
                "failed to compare manifest against {}",
                compare_path.display()
            )
        })?;
    }

    let nef_path = cli
        .nef
        .clone()
        .unwrap_or_else(|| derive_output_path(&cli.input, "nef"));
    write_nef_with_metadata(
        &translation.script,
        metadata.source.as_deref(),
        &metadata.method_tokens,
        &nef_path,
    )?;

    let manifest_path = cli
        .manifest
        .clone()
        .unwrap_or_else(|| derive_output_path(&cli.input, "manifest.json"));
    fs::write(&manifest_path, &manifest_string)?;

    info!(
        "Generated {} and {}",
        nef_path.display(),
        manifest_path.display()
    );

    Ok(())
}
