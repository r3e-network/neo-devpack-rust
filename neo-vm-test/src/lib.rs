// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

//! # neo-vm-test — run compiled Neo N3 contracts on a real NeoVM from Rust
//!
//! The Rust-on-Neo analogue of Solana's `solana-program-test` / the Move VM
//! test harness: compile a contract to a NEF and execute its methods on a real
//! NeoVM (the neo-go embedded VM, via the conformance oracle), then assert on
//! the result — all from an ordinary `#[test]`.
//!
//! Three layers of testing complement each other in this repo:
//! 1. **Unit / logic** — `neo-test` mock runtime (host execution, fast).
//! 2. **Bytecode e2e (this crate)** — the *translated* NEF runs on a real
//!    NeoVM, catching translator/lowering bugs unit tests can't.
//! 3. **Full-chain e2e** — Neo Express (`integration-tests`), for
//!    storage/runtime/deploy against a live chain.
//!
//! ```no_run
//! use neo_vm_test::Contract;
//! let c = Contract::compile("contracts/feature-arithmetic").unwrap();
//! c.invoke("calc", &[20.into(), 6.into()]).assert_returns_i64(81);
//! ```
//!
//! ## Backend
//! Execution shells out to the conformance oracle (`conformance/oracle`,
//! neo-go's VM). The binary is resolved from `$NEO_VM_ORACLE`, else built on
//! demand with `go build`. Translation uses the `wasm-neovm` binary
//! (`$NEO_WASM_NEOVM`, else built with cargo). Both are repo-local dev tools.
//!
//! The oracle services `System.Storage.*`, `System.Runtime.*` and
//! `System.Contract.GetCallFlags` against in-memory state (seeded via the
//! [`Contract::call`] builder: storage, signers, time, network), mirroring
//! neo-go's interop ABI — so pure compute *and* stateful storage/witness/notify
//! contracts run end-to-end. Deploy/upgrade, multi-contract calls and
//! native-contract interop need a live chain (Neo Express); unsupported
//! syscalls surface as a FAULT rather than a wrong answer.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

/// Repo root = parent of this crate's manifest dir (dev-only, always in-tree).
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("neo-vm-test lives one level below the repo root")
        .to_path_buf()
}

/// An argument passed to a contract method.
#[derive(Clone, Debug)]
pub enum VmArg {
    Int(i64),
    Bool(bool),
    Bytes(Vec<u8>),
    Str(String),
}

impl From<i64> for VmArg {
    fn from(v: i64) -> Self {
        VmArg::Int(v)
    }
}
impl From<i32> for VmArg {
    fn from(v: i32) -> Self {
        VmArg::Int(v as i64)
    }
}
impl From<bool> for VmArg {
    fn from(v: bool) -> Self {
        VmArg::Bool(v)
    }
}
impl From<&[u8]> for VmArg {
    fn from(v: &[u8]) -> Self {
        VmArg::Bytes(v.to_vec())
    }
}
impl From<&str> for VmArg {
    fn from(v: &str) -> Self {
        VmArg::Str(v.to_string())
    }
}

impl VmArg {
    fn to_json(&self) -> serde_json::Value {
        use serde_json::json;
        match self {
            VmArg::Int(v) => json!({"type": "integer", "value": v.to_string()}),
            VmArg::Bool(v) => json!({"type": "boolean", "value": v.to_string()}),
            VmArg::Bytes(b) => json!({"type": "bytes", "value": hex(b)}),
            VmArg::Str(s) => json!({"type": "string", "value": s}),
        }
    }
}

fn hex(b: &[u8]) -> String {
    let mut s = String::with_capacity(b.len() * 2);
    for byte in b {
        s.push_str(&format!("{byte:02x}"));
    }
    s
}

/// A `System.Runtime.Notify` captured during execution.
#[derive(Clone, Debug)]
pub struct Event {
    pub name: String,
    /// State items, formatted by the oracle (decimal ints, hex bytes, …).
    pub state: Vec<String>,
}

/// The outcome of executing one contract method on the NeoVM.
#[derive(Clone, Debug)]
pub struct VmOutcome {
    /// "HALT" (success) or "FAULT" (the VM aborted).
    pub state: String,
    /// Stack items left by the method, top first, formatted by the oracle
    /// (integers as decimal, byte arrays as hex, etc.).
    pub return_stack: Vec<String>,
    pub gas_consumed: i64,
    /// Final contract storage after the call (key, value) as raw bytes.
    pub storage_diff: Vec<(Vec<u8>, Vec<u8>)>,
    /// Notifications emitted via `System.Runtime.Notify`, in order.
    pub events: Vec<Event>,
    pub error_message: Option<String>,
}

impl VmOutcome {
    pub fn is_halt(&self) -> bool {
        self.state == "HALT"
    }
    pub fn is_fault(&self) -> bool {
        self.state == "FAULT"
    }
    /// The top stack item (the method's return value), if any.
    pub fn top(&self) -> Option<&str> {
        self.return_stack.first().map(|s| s.as_str())
    }
    pub fn top_i64(&self) -> Option<i64> {
        self.top().and_then(|s| s.parse().ok())
    }
    pub fn top_bool(&self) -> Option<bool> {
        match self.top() {
            Some("true") | Some("1") => Some(true),
            Some("false") | Some("0") => Some(false),
            _ => None,
        }
    }

    // ---- assertions ----
    pub fn assert_halt(&self) -> &Self {
        assert!(
            self.is_halt(),
            "expected HALT, got {} ({:?})",
            self.state,
            self.error_message
        );
        self
    }
    pub fn assert_fault(&self) -> &Self {
        assert!(self.is_fault(), "expected FAULT, got HALT ({:?})", self.return_stack);
        self
    }
    pub fn assert_returns_i64(&self, expected: i64) -> &Self {
        self.assert_halt();
        assert_eq!(
            self.top_i64(),
            Some(expected),
            "return mismatch: stack={:?}",
            self.return_stack
        );
        self
    }
    pub fn assert_returns_bool(&self, expected: bool) -> &Self {
        self.assert_halt();
        assert_eq!(self.top_bool(), Some(expected), "stack={:?}", self.return_stack);
        self
    }

    // ---- storage + events ----
    /// Final stored value for `key`, if present.
    pub fn storage(&self, key: &[u8]) -> Option<&[u8]> {
        self.storage_diff
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_slice())
    }
    pub fn assert_storage(&self, key: &[u8], expected: &[u8]) -> &Self {
        assert_eq!(
            self.storage(key),
            Some(expected),
            "storage[{}] mismatch (diff: {:?})",
            hex(key),
            self.storage_diff
                .iter()
                .map(|(k, v)| (hex(k), hex(v)))
                .collect::<Vec<_>>()
        );
        self
    }
    pub fn events(&self) -> &[Event] {
        &self.events
    }
    /// The first event named `name`, if any.
    pub fn event(&self, name: &str) -> Option<&Event> {
        self.events.iter().find(|e| e.name == name)
    }
    pub fn assert_event(&self, name: &str) -> &Self {
        assert!(
            self.event(name).is_some(),
            "expected event {name:?}, got {:?}",
            self.events.iter().map(|e| &e.name).collect::<Vec<_>>()
        );
        self
    }
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .filter_map(|i| u8::from_str_radix(s.get(i..i + 2)?, 16).ok())
        .collect()
}

/// A compiled contract (NEF + manifest) ready to invoke on the VM.
pub struct Contract {
    nef: PathBuf,
    manifest: PathBuf,
    _tmp: Option<tempdir::TempDir>,
}

impl Contract {
    /// Use already-built artifacts.
    pub fn from_files(nef: impl Into<PathBuf>, manifest: impl Into<PathBuf>) -> Self {
        Contract {
            nef: nef.into(),
            manifest: manifest.into(),
            _tmp: None,
        }
    }

    /// Build a contract crate to wasm32 and translate it to a NEF, returning a
    /// handle whose artifacts live in a temp dir for the test's lifetime.
    pub fn compile(crate_dir: impl AsRef<Path>) -> Result<Self, String> {
        let crate_dir = crate_dir.as_ref();
        let crate_dir = if crate_dir.is_absolute() {
            crate_dir.to_path_buf()
        } else {
            repo_root().join(crate_dir)
        };
        // build wasm32 (cdylib/lib)
        let out = Command::new("cargo")
            .args([
                "build",
                "--release",
                "--target",
                "wasm32-unknown-unknown",
                "--lib",
            ])
            .current_dir(&crate_dir)
            .output()
            .map_err(|e| format!("cargo build: {e}"))?;
        if !out.status.success() {
            return Err(format!(
                "wasm build failed for {}:\n{}",
                crate_dir.display(),
                String::from_utf8_lossy(&out.stderr)
            ));
        }
        let wasm = find_wasm(&crate_dir)?;
        let tmp = tempdir::TempDir::new().map_err(|e| format!("tempdir: {e}"))?;
        let nef = tmp.path().join("contract.nef");
        let manifest = tmp.path().join("contract.manifest.json");
        let out = Command::new(wasm_neovm_bin()?)
            .args([
                "--input",
                wasm.to_str().unwrap(),
                "--name",
                "TestContract",
                "--nef",
                nef.to_str().unwrap(),
                "--manifest",
                manifest.to_str().unwrap(),
            ])
            .output()
            .map_err(|e| format!("wasm-neovm: {e}"))?;
        if !out.status.success() {
            return Err(format!(
                "translation failed:\n{}",
                String::from_utf8_lossy(&out.stderr)
            ));
        }
        Ok(Contract {
            nef,
            manifest,
            _tmp: Some(tmp),
        })
    }

    /// Invoke `method` with `args` on a real NeoVM (no seeded state); panics
    /// only on harness failure (the VM's own FAULT is reported in [`VmOutcome`]).
    pub fn invoke(&self, method: &str, args: &[VmArg]) -> VmOutcome {
        self.call(method).args(args).run()
    }

    pub fn try_invoke(&self, method: &str, args: &[VmArg]) -> Result<VmOutcome, String> {
        self.call(method).args(args).try_run()
    }

    /// Start a stateful invocation builder: seed storage, add signers, set the
    /// runtime environment, then `.run()`.
    ///
    /// ```no_run
    /// # use neo_vm_test::Contract;
    /// # let c = Contract::compile("contracts/feature-storage-raw").unwrap();
    /// c.call("getKeyed")
    ///     .arg(7)
    ///     .storage(&[7], &999i64.to_le_bytes())
    ///     .signer([0u8; 20])
    ///     .run()
    ///     .assert_returns_i64(999);
    /// ```
    pub fn call<'a>(&'a self, method: &'a str) -> Invocation<'a> {
        Invocation {
            contract: self,
            method,
            args: Vec::new(),
            storage: Vec::new(),
            signers: Vec::new(),
            time: None,
            network: None,
        }
    }

    fn run_request(&self, req: serde_json::Value) -> Result<VmOutcome, String> {
        let tmp = tempdir::TempDir::new().map_err(|e| format!("tempdir: {e}"))?;
        let req_path = tmp.path().join("req.json");
        let res_path = tmp.path().join("res.json");
        std::fs::write(&req_path, serde_json::to_vec(&req).unwrap())
            .map_err(|e| format!("write req: {e}"))?;
        let status = Command::new(oracle_bin()?)
            .args([
                "-in",
                req_path.to_str().unwrap(),
                "-out",
                res_path.to_str().unwrap(),
            ])
            .output()
            .map_err(|e| format!("oracle: {e}"))?;
        if !status.status.success() {
            return Err(format!(
                "oracle exited non-zero:\n{}",
                String::from_utf8_lossy(&status.stderr)
            ));
        }
        let body = std::fs::read_to_string(&res_path).map_err(|e| format!("read res: {e}"))?;
        let v: serde_json::Value =
            serde_json::from_str(&body).map_err(|e| format!("parse res: {e}"))?;
        Ok(VmOutcome {
            state: v["state"].as_str().unwrap_or("FAULT").to_string(),
            return_stack: v["return_stack"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .map(|x| x.as_str().unwrap_or("").to_string())
                        .collect()
                })
                .unwrap_or_default(),
            gas_consumed: v["gas_consumed"].as_i64().unwrap_or(0),
            storage_diff: v["storage_diff"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .map(|s| {
                            (
                                unhex(s["key"].as_str().unwrap_or("")),
                                unhex(s["value"].as_str().unwrap_or("")),
                            )
                        })
                        .collect()
                })
                .unwrap_or_default(),
            events: v["events"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .map(|e| Event {
                            name: e["name"].as_str().unwrap_or("").to_string(),
                            state: e["state"]
                                .as_array()
                                .map(|st| {
                                    st.iter()
                                        .map(|x| x.as_str().unwrap_or("").to_string())
                                        .collect()
                                })
                                .unwrap_or_default(),
                        })
                        .collect()
                })
                .unwrap_or_default(),
            error_message: v["error_message"].as_str().map(|s| s.to_string()),
        })
    }

    pub fn nef_path(&self) -> &Path {
        &self.nef
    }
    pub fn manifest_path(&self) -> &Path {
        &self.manifest
    }

    // ---- debugging ----

    /// Disassemble the contract's NeoVM script to `IP: OPCODE operand` lines.
    pub fn disassemble(&self) -> Result<String, String> {
        let out = Command::new(oracle_bin()?)
            .args([
                "-disasm",
                self.nef.to_str().unwrap(),
                "-from",
                "0",
                "-count",
                "1000000",
            ])
            .output()
            .map_err(|e| format!("oracle disasm: {e}"))?;
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }

    /// Single-step trace of a method: each executed instruction with the
    /// post-step evaluation stack (top first). Invaluable for debugging a
    /// FAULT or a wrong return value.
    pub fn trace(&self, method: &str, args: &[VmArg]) -> Result<String, String> {
        use serde_json::json;
        let req = json!({
            "nef_path": self.nef, "manifest_path": self.manifest, "method": method,
            "arguments": args.iter().map(|a| a.to_json()).collect::<Vec<_>>(),
            "signers": [], "initial_storage": [], "gas_limit": 2_000_000_000i64,
        });
        let tmp = tempdir::TempDir::new().map_err(|e| format!("tempdir: {e}"))?;
        let req_path = tmp.path().join("req.json");
        std::fs::write(&req_path, serde_json::to_vec(&req).unwrap())
            .map_err(|e| format!("write req: {e}"))?;
        let out = Command::new(oracle_bin()?)
            .args([
                "-trace",
                "-in",
                req_path.to_str().unwrap(),
                "-from",
                "0",
                "-to",
                "1000000000",
            ])
            .output()
            .map_err(|e| format!("oracle trace: {e}"))?;
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }
}

/// Builder for a stateful invocation. Create via [`Contract::call`].
pub struct Invocation<'a> {
    contract: &'a Contract,
    method: &'a str,
    args: Vec<VmArg>,
    storage: Vec<(Vec<u8>, Vec<u8>)>,
    signers: Vec<[u8; 20]>,
    time: Option<u64>,
    network: Option<u32>,
}

impl<'a> Invocation<'a> {
    /// Append one argument.
    pub fn arg(mut self, a: impl Into<VmArg>) -> Self {
        self.args.push(a.into());
        self
    }
    /// Set all arguments at once.
    pub fn args(mut self, a: &[VmArg]) -> Self {
        self.args.extend_from_slice(a);
        self
    }
    /// Seed a storage entry the contract will see (raw key/value bytes).
    pub fn storage(mut self, key: &[u8], value: &[u8]) -> Self {
        self.storage.push((key.to_vec(), value.to_vec()));
        self
    }
    /// Add a witnessing signer (a 20-byte account); `check_witness` returns
    /// true for it.
    pub fn signer(mut self, account: [u8; 20]) -> Self {
        self.signers.push(account);
        self
    }
    /// Set `System.Runtime.GetTime` (ms since epoch).
    pub fn time(mut self, ms: u64) -> Self {
        self.time = Some(ms);
        self
    }
    /// Set `System.Runtime.GetNetwork` magic.
    pub fn network(mut self, magic: u32) -> Self {
        self.network = Some(magic);
        self
    }

    /// Run on the VM; panics only on harness failure (VM FAULT is in the result).
    pub fn run(self) -> VmOutcome {
        let method = self.method.to_string();
        self.try_run()
            .unwrap_or_else(|e| panic!("invoke {method}: {e}"))
    }

    pub fn try_run(self) -> Result<VmOutcome, String> {
        use serde_json::json;
        let mut req = json!({
            "nef_path": self.contract.nef,
            "manifest_path": self.contract.manifest,
            "method": self.method,
            "arguments": self.args.iter().map(|a| a.to_json()).collect::<Vec<_>>(),
            "signers": self.signers.iter()
                .map(|s| json!({"account": hex(s), "scopes": "CalledByEntry"}))
                .collect::<Vec<_>>(),
            "initial_storage": self.storage.iter()
                .map(|(k, v)| json!({"contract": "", "key": hex(k), "value": hex(v)}))
                .collect::<Vec<_>>(),
            "gas_limit": 2_000_000_000i64,
        });
        if let Some(t) = self.time {
            req["time"] = json!(t);
        }
        if let Some(n) = self.network {
            req["network"] = json!(n);
        }
        self.contract.run_request(req)
    }
}

fn find_wasm(crate_dir: &Path) -> Result<PathBuf, String> {
    let dir = crate_dir.join("target/wasm32-unknown-unknown/release");
    let entries = std::fs::read_dir(&dir).map_err(|e| format!("read {}: {e}", dir.display()))?;
    for e in entries.flatten() {
        let p = e.path();
        if p.extension().and_then(|s| s.to_str()) == Some("wasm") {
            return Ok(p);
        }
    }
    Err(format!("no .wasm in {}", dir.display()))
}

/// Resolve (and build on first use) the wasm-neovm translator binary.
fn wasm_neovm_bin() -> Result<PathBuf, String> {
    static CELL: OnceLock<Result<PathBuf, String>> = OnceLock::new();
    CELL.get_or_init(|| {
        if let Ok(p) = std::env::var("NEO_WASM_NEOVM") {
            return Ok(PathBuf::from(p));
        }
        let bin = repo_root().join("target/release/wasm-neovm");
        if !bin.exists() {
            let out = Command::new("cargo")
                .args(["build", "-p", "wasm-neovm", "--release"])
                .current_dir(repo_root())
                .output()
                .map_err(|e| format!("build wasm-neovm: {e}"))?;
            if !out.status.success() {
                return Err(format!(
                    "build wasm-neovm failed:\n{}",
                    String::from_utf8_lossy(&out.stderr)
                ));
            }
        }
        Ok(bin)
    })
    .clone()
}

/// Resolve (and build on first use) the conformance oracle binary.
fn oracle_bin() -> Result<PathBuf, String> {
    static CELL: OnceLock<Result<PathBuf, String>> = OnceLock::new();
    CELL.get_or_init(|| {
        if let Ok(p) = std::env::var("NEO_VM_ORACLE") {
            return Ok(PathBuf::from(p));
        }
        let bin = repo_root().join("target/neo-oracle");
        if !bin.exists() {
            // The oracle's Go module lives in conformance/ (its own go.mod), so
            // build from there with an absolute output path.
            let out = Command::new("go")
                .args(["build", "-o", bin.to_str().unwrap(), "./oracle"])
                .current_dir(repo_root().join("conformance"))
                .output()
                .map_err(|e| format!("go build oracle: {e} (is Go installed?)"))?;
            if !out.status.success() {
                return Err(format!(
                    "go build oracle failed:\n{}",
                    String::from_utf8_lossy(&out.stderr)
                ));
            }
        }
        Ok(bin)
    })
    .clone()
}

// Minimal embedded temp-dir helper (avoids an external dep for a dev crate).
mod tempdir {
    use std::path::{Path, PathBuf};
    pub struct TempDir(PathBuf);
    impl TempDir {
        pub fn new() -> std::io::Result<Self> {
            // Unique-ish dir under the OS temp; good enough for serial-ish tests.
            let base = std::env::temp_dir();
            let mut n = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            n ^= std::process::id() as u128;
            n = n.wrapping_mul(6364136223846793005).wrapping_add(1);
            let p = base.join(format!("neo-vm-test-{n:x}"));
            std::fs::create_dir_all(&p)?;
            Ok(TempDir(p))
        }
        pub fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}
