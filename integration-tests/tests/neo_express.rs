use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("integration-tests should sit under repo root")
        .to_path_buf()
}

fn neoxp_bin() -> String {
    std::env::var("NEOXP_BIN")
        .or_else(|_| std::env::var("NEO_EXPRESS_CLI"))
        .unwrap_or_else(|_| "neoxp".to_string())
}

fn run_command(command: &mut Command, label: &str) -> String {
    let output = command
        .output()
        .unwrap_or_else(|e| panic!("{label} failed to start: {e}"));
    if !output.status.success() {
        panic!(
            "{label} failed (status: {:?})\nstdout:\n{}\nstderr:\n{}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }
    String::from_utf8(output.stdout).unwrap_or_else(|e| panic!("{label} output was not UTF-8: {e}"))
}

fn run_neoxp_json(neoxp: &str, args: &[String], label: &str) -> Value {
    let output = run_command(Command::new(neoxp).args(args), label);
    serde_json::from_str(&output).unwrap_or_else(|e| panic!("{label} returned invalid JSON: {e}"))
}

fn ensure_cross_chain_artifacts() -> PathBuf {
    static BUILD_ONCE: OnceLock<()> = OnceLock::new();
    let root = repo_root();
    BUILD_ONCE.get_or_init(|| {
        let status = Command::new("make")
            .arg("cross-chain")
            .current_dir(&root)
            .status()
            .expect("failed to run `make cross-chain`");
        assert!(
            status.success(),
            "`make cross-chain` failed with status: {:?}",
            status.code()
        );
    });
    root.join("build")
}

fn unique_chain_path(tag: &str) -> (PathBuf, PathBuf) {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic from UNIX_EPOCH")
        .as_nanos();
    let base = std::env::temp_dir().join(format!(
        "neo-devpack-rust-neoxp-{tag}-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&base).expect("failed to create temporary Neo Express directory");
    (base.clone(), base.join("default.neo-express"))
}

fn assert_halt_and_stack_value(result: &Value, expected: &str, label: &str) {
    assert_eq!(
        result["state"].as_str().unwrap_or_default(),
        "HALT",
        "{label} should HALT",
    );
    assert_eq!(
        result["stack"][0]["value"].as_str().unwrap_or_default(),
        expected,
        "{label} returned an unexpected stack value",
    );
}

fn deploy_contract(neoxp: &str, chain: &Path, nef: &Path, expected_name: &str) {
    let args = vec![
        "contract".to_string(),
        "deploy".to_string(),
        "-i".to_string(),
        chain.display().to_string(),
        "-j".to_string(),
        "-f".to_string(),
        nef.display().to_string(),
        "genesis".to_string(),
    ];
    let deployed = run_neoxp_json(neoxp, &args, "neo-express contract deploy");
    assert_eq!(
        deployed["contract-name"].as_str().unwrap_or_default(),
        expected_name,
        "deployed contract name should match manifest name",
    );
    assert!(
        deployed["contract-hash"].as_str().is_some(),
        "deployed contract hash should be present",
    );
}

#[test]
#[ignore = "requires Neo Express CLI and translated contract artifacts"]
fn hello_world_nef_is_deployable() {
    let neoxp = neoxp_bin();
    let build_dir = ensure_cross_chain_artifacts();
    let nef = build_dir.join("solana_hello.nef");
    let manifest = build_dir.join("solana_hello.manifest.json");
    assert!(nef.exists(), "missing artifact: {}", nef.display());
    assert!(
        manifest.exists(),
        "missing artifact: {}",
        manifest.display()
    );

    let (chain_dir, chain) = unique_chain_path("solana-hello");
    let create_args = vec![
        "create".to_string(),
        "-o".to_string(),
        chain.display().to_string(),
        "-f".to_string(),
    ];
    let _ = run_command(
        Command::new(&neoxp).args(&create_args),
        "neo-express create chain",
    );

    deploy_contract(&neoxp, &chain, &nef, "solana-hello");

    let main_args = vec![
        "contract".to_string(),
        "run".to_string(),
        "-i".to_string(),
        chain.display().to_string(),
        "-r".to_string(),
        "-j".to_string(),
        "solana-hello".to_string(),
        "main".to_string(),
        "1".to_string(),
        "2".to_string(),
    ];
    let main_result = run_neoxp_json(&neoxp, &main_args, "neo-express contract run main");
    assert_halt_and_stack_value(&main_result, "0", "solana-hello.main");

    let time_args = vec![
        "contract".to_string(),
        "run".to_string(),
        "-i".to_string(),
        chain.display().to_string(),
        "-r".to_string(),
        "-j".to_string(),
        "solana-hello".to_string(),
        "get_time".to_string(),
    ];
    let time_result = run_neoxp_json(&neoxp, &time_args, "neo-express contract run get_time");
    assert_eq!(
        time_result["state"].as_str().unwrap_or_default(),
        "HALT",
        "solana-hello.get_time should HALT",
    );
    let time_value = time_result["stack"][0]["value"]
        .as_str()
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(-1);
    assert!(time_value >= 1, "solana-hello.get_time should return >= 1");

    fs::remove_dir_all(chain_dir).expect("failed to clean temporary chain directory");
}

#[test]
#[ignore = "requires Neo Express CLI and translated contract artifacts"]
fn move_coin_nef_is_available() {
    let neoxp = neoxp_bin();
    let build_dir = ensure_cross_chain_artifacts();
    let nef = build_dir.join("MoveCoin.nef");
    let manifest = build_dir.join("MoveCoin.manifest.json");
    assert!(nef.exists(), "missing artifact: {}", nef.display());
    assert!(
        manifest.exists(),
        "missing artifact: {}",
        manifest.display()
    );

    let (chain_dir, chain) = unique_chain_path("move-coin");
    let create_args = vec![
        "create".to_string(),
        "-o".to_string(),
        chain.display().to_string(),
        "-f".to_string(),
    ];
    let _ = run_command(
        Command::new(&neoxp).args(&create_args),
        "neo-express create chain",
    );

    deploy_contract(&neoxp, &chain, &nef, "MoveCoin");

    let total_supply_args = vec![
        "contract".to_string(),
        "run".to_string(),
        "-i".to_string(),
        chain.display().to_string(),
        "-r".to_string(),
        "-j".to_string(),
        "MoveCoin".to_string(),
        "total_supply".to_string(),
    ];
    let total_supply = run_neoxp_json(
        &neoxp,
        &total_supply_args,
        "neo-express contract run total_supply",
    );
    assert_halt_and_stack_value(&total_supply, "1000000", "MoveCoin.total_supply");

    let has_coin_args = vec![
        "contract".to_string(),
        "run".to_string(),
        "-i".to_string(),
        chain.display().to_string(),
        "-r".to_string(),
        "-j".to_string(),
        "MoveCoin".to_string(),
        "has_coin".to_string(),
        "1".to_string(),
    ];
    let has_coin = run_neoxp_json(&neoxp, &has_coin_args, "neo-express contract run has_coin");
    assert_halt_and_stack_value(&has_coin, "1", "MoveCoin.has_coin");

    let mint_args = vec![
        "contract".to_string(),
        "run".to_string(),
        "-i".to_string(),
        chain.display().to_string(),
        "-r".to_string(),
        "-j".to_string(),
        "MoveCoin".to_string(),
        "mint".to_string(),
        "1".to_string(),
        "10".to_string(),
    ];
    let mint = run_neoxp_json(&neoxp, &mint_args, "neo-express contract run mint");
    assert_halt_and_stack_value(&mint, "1", "MoveCoin.mint");

    let transfer_args = vec![
        "contract".to_string(),
        "run".to_string(),
        "-i".to_string(),
        chain.display().to_string(),
        "-r".to_string(),
        "-j".to_string(),
        "MoveCoin".to_string(),
        "transfer".to_string(),
        "1".to_string(),
        "2".to_string(),
        "5".to_string(),
    ];
    let transfer = run_neoxp_json(&neoxp, &transfer_args, "neo-express contract run transfer");
    assert_halt_and_stack_value(&transfer, "1", "MoveCoin.transfer");

    let burn_args = vec![
        "contract".to_string(),
        "run".to_string(),
        "-i".to_string(),
        chain.display().to_string(),
        "-r".to_string(),
        "-j".to_string(),
        "MoveCoin".to_string(),
        "burn".to_string(),
        "1".to_string(),
        "5".to_string(),
    ];
    let burn = run_neoxp_json(&neoxp, &burn_args, "neo-express contract run burn");
    assert_halt_and_stack_value(&burn, "1", "MoveCoin.burn");

    let balance_args = vec![
        "contract".to_string(),
        "run".to_string(),
        "-i".to_string(),
        chain.display().to_string(),
        "-r".to_string(),
        "-j".to_string(),
        "MoveCoin".to_string(),
        "balance".to_string(),
        "1".to_string(),
    ];
    let balance = run_neoxp_json(&neoxp, &balance_args, "neo-express contract run balance");
    assert_halt_and_stack_value(&balance, "1000", "MoveCoin.balance");

    fs::remove_dir_all(chain_dir).expect("failed to clean temporary chain directory");
}
