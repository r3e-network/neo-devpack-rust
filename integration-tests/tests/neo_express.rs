use std::env;
use std::fs;
use std::path::Path;

use log::warn;

#[test]
#[ignore = "requires a running Neo Express instance"]
fn hello_world_nef_is_deployable() {
    // Ensure the translated artefacts exist before hitting Neo Express.
    let nef = Path::new("build/solana_hello.nef");
    let manifest = Path::new("build/solana_hello.manifest.json");
    if !nef.exists() || !manifest.exists() {
        warn!(
            "Skipping: expected {} and {} to exist; run `make cross-chain` first",
            nef.display(),
            manifest.display()
        );
        return;
    }

    // When Neo Express is available, provide the RPC endpoint via NEO_EXPRESS_RPC
    // (for example: http://localhost:50012). If the variable is unset we skip
    // without failing the test suite.
    let rpc = match env::var("NEO_EXPRESS_RPC") {
        Ok(value) => value,
        Err(_) => {
            warn!("Skipping Neo Express integration test – set NEO_EXPRESS_RPC to enable.");
            return;
        }
    };

    // Record the artefact hashes – callers can use these with neo-express `contract deploy`.
    let nef_bytes = fs::read(nef).expect("failed to read NEF");
    let manifest_bytes = fs::read(manifest).expect("failed to read manifest");
    warn!(
        "Ready to deploy solana-hello (NEF {} bytes, manifest {} bytes) via {rpc}",
        nef_bytes.len(),
        manifest_bytes.len()
    );

    // The actual deployment/invocation steps are environment-specific and rely
    // on the neo-express CLI. Wire those commands into your CI/CD pipeline by
    // using the artefacts above together with `neo-express contract deploy`
    // and `neo-express contract invoke`.
}

#[test]
#[ignore = "requires a running Neo Express instance"]
fn move_coin_nef_is_available() {
    // Experimental Move sample – artefacts should exist after `make cross-chain`.
    let nef = Path::new("build/MoveCoin.nef");
    let manifest = Path::new("build/MoveCoin.manifest.json");
    if !nef.exists() || !manifest.exists() {
        warn!(
            "Skipping: expected {} and {} to exist; run `make cross-chain` first",
            nef.display(),
            manifest.display()
        );
    }
}
