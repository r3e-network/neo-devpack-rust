use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Parser;

use wasm_neovm::{
    extract_nef_metadata,
    manifest::{merge_manifest, propagate_safe_flags},
    translate_module, write_nef_with_metadata,
};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Translate a Wasm module into NeoVM NEF artefacts"
)]
struct Cli {
    /// Path to the WebAssembly module compiled from Rust
    #[arg(short, long)]
    input: PathBuf,

    /// Output NEF path (default: <input_basename>.nef)
    #[arg(long)]
    nef: Option<PathBuf>,

    /// Output manifest path (default: <input_basename>.manifest.json)
    #[arg(long)]
    manifest: Option<PathBuf>,

    /// Contract name stored in the manifest
    #[arg(long, default_value = "Contract")]
    name: String,

    /// Path to a JSON file providing manifest overlay data
    #[arg(long = "manifest-overlay")]
    manifest_overlay: Option<PathBuf>,

    /// Source URL recorded in the NEF header
    #[arg(long = "source-url")]
    source_url: Option<String>,
}

fn derive_output_path(input: &Path, extension: &str) -> PathBuf {
    if input.file_name().is_some() {
        input.with_extension(extension)
    } else {
        let mut fallback = input.to_path_buf();
        fallback.push("contract");
        fallback.set_extension(extension);
        fallback
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let module = fs::read(&cli.input)
        .with_context(|| format!("failed to read input module {}", cli.input.display()))?;

    let translation = translate_module(&module, &cli.name)?;

    let mut manifest_value = translation.manifest.value.clone();
    if let Some(path) = &cli.manifest_overlay {
        let overlay_bytes = fs::read_to_string(path)
            .with_context(|| format!("failed to read manifest overlay {}", path.display()))?;
        let overlay: serde_json::Value =
            serde_json::from_str(&overlay_bytes).with_context(|| {
                format!(
                    "failed to parse manifest overlay JSON from {}",
                    path.display()
                )
            })?;
        merge_manifest(&mut manifest_value, &overlay);
    }
    propagate_safe_flags(&mut manifest_value);

    let manifest_string =
        serde_json::to_string_pretty(&manifest_value).context("failed to render manifest JSON")?;

    let metadata = extract_nef_metadata(&manifest_value)?;
    let mut source_url = metadata.source.clone();
    if let Some(cli_source) = &cli.source_url {
        source_url = Some(cli_source.clone());
    }

    let nef_path = cli
        .nef
        .clone()
        .unwrap_or_else(|| derive_output_path(&cli.input, "nef"));
    write_nef_with_metadata(
        &translation.script,
        source_url.as_deref(),
        &metadata.method_tokens,
        &nef_path,
    )?;

    let manifest_path = cli
        .manifest
        .clone()
        .unwrap_or_else(|| derive_output_path(&cli.input, "manifest.json"));
    fs::write(&manifest_path, &manifest_string)?;

    println!(
        "Generated {} and {}",
        nef_path.display(),
        manifest_path.display()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::derive_output_path;
    use std::path::Path;

    #[test]
    fn derive_output_preserves_directory_for_nef() {
        let input = Path::new("contracts/example/target/release/contract.wasm");
        let derived = derive_output_path(input, "nef");
        assert_eq!(
            derived,
            Path::new("contracts/example/target/release/contract.nef")
        );
    }

    #[test]
    fn derive_output_handles_multi_part_extension() {
        let input = Path::new("contracts/example/contract.wasm");
        let derived = derive_output_path(input, "manifest.json");
        assert_eq!(
            derived,
            Path::new("contracts/example/contract.manifest.json")
        );
    }

    #[test]
    fn derive_output_handles_missing_filename() {
        let input = Path::new(".");
        let derived = derive_output_path(input, "nef");
        assert_eq!(derived, Path::new("./contract.nef"));
    }
}
