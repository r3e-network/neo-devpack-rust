use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::Parser;
use serde_json::{Map, Value};

use wasm_neovm::{
    extract_nef_metadata, translate_with_config, write_nef_with_metadata, ManifestOverlay,
    SourceChain, TranslationConfig,
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

    /// Path to an existing manifest to compare against (translation fails when they differ)
    #[arg(long = "compare-manifest")]
    compare_manifest: Option<PathBuf>,

    /// Source blockchain for cross-chain compilation (neo, solana, move)
    #[arg(long = "source-chain", default_value = "neo")]
    source_chain: String,
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

    // Parse source chain
    let source_chain = SourceChain::from_str(&cli.source_chain).unwrap_or_else(|| {
        eprintln!(
            "Warning: unknown source chain '{}', defaulting to 'neo'",
            cli.source_chain
        );
        SourceChain::Neo
    });

    if source_chain != SourceChain::Neo {
        println!("Cross-chain compilation: {:?} -> NeoVM", source_chain);
    }

    let module = fs::read(&cli.input)
        .with_context(|| format!("failed to read input module {}", cli.input.display()))?;

    let mut config = TranslationConfig::new(&cli.name);
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

    println!(
        "Generated {} and {}",
        nef_path.display(),
        manifest_path.display()
    );

    Ok(())
}

fn compare_manifest(reference_path: &Path, generated: &Value) -> Result<()> {
    let bytes = fs::read_to_string(reference_path)
        .with_context(|| format!("failed to read manifest {}", reference_path.display()))?;
    let reference: Value = serde_json::from_str(&bytes).with_context(|| {
        format!(
            "failed to parse manifest JSON from {}",
            reference_path.display()
        )
    })?;
    if &reference == generated {
        println!("Manifest matches {}", reference_path.display());
        return Ok(());
    }

    let expected = serde_json::to_string_pretty(&reference)?;
    let actual = serde_json::to_string_pretty(generated)?;
    println!("Manifest differs from {}:", reference_path.display());
    for diff in diff::lines(&expected, &actual) {
        use diff::Result::{Both, Left, Right};
        match diff {
            Left(line) => println!("-{}", line),
            Right(line) => println!("+{}", line),
            Both(_, _) => {}
        }
    }
    bail!(
        "generated manifest does not match {}",
        reference_path.display()
    );
}

#[cfg(test)]
mod tests {
    use super::{compare_manifest, derive_output_path};
    use serde_json::json;
    use std::path::Path;
    use tempfile::NamedTempFile;

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

    #[test]
    fn compare_manifest_matches_reference_file() {
        let file = NamedTempFile::new().unwrap();
        let value = json!({"name": "Contract"});
        std::fs::write(file.path(), serde_json::to_string(&value).unwrap()).unwrap();
        compare_manifest(file.path(), &value).unwrap();
    }

    #[test]
    fn compare_manifest_detects_difference() {
        let file = NamedTempFile::new().unwrap();
        let reference = json!({"name": "Reference"});
        let generated = json!({"name": "Generated"});
        std::fs::write(file.path(), serde_json::to_string(&reference).unwrap()).unwrap();
        let err = compare_manifest(file.path(), &generated).unwrap_err();
        assert!(err
            .to_string()
            .contains("generated manifest does not match"));
    }
}

fn apply_source_url(manifest: &mut Value, source: &str) {
    if let Some(obj) = manifest.as_object_mut() {
        obj.insert("source".to_string(), Value::String(source.to_string()));
        let extra = obj
            .entry("extra")
            .or_insert_with(|| Value::Object(Map::new()));
        if let Some(extra_obj) = extra.as_object_mut() {
            extra_obj.insert("nefSource".to_string(), Value::String(source.to_string()));
        }
    }
}
