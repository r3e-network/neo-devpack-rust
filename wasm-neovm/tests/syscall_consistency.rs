use regex::Regex;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SyscallEntry {
    name: String,
    hash: u32,
}

fn parse_neovm_syscalls() -> anyhow::Result<Vec<SyscallEntry>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .expect("wasm-neovm crate expected one directory below repo root");
    let neo_dir = repo_root.join("neo/src/Neo/SmartContract");

    let mut names: BTreeSet<String> = BTreeSet::new();
    let regex = Regex::new(r#"Register\("([^"]+)""#)?;
    for entry in WalkDir::new(&neo_dir) {
        let entry = entry?;
        if entry.file_type().is_file()
            && entry
                .path()
                .extension()
                .map(|ext| ext == "cs")
                .unwrap_or(false)
        {
            let contents = fs::read_to_string(entry.path())?;
            for caps in regex.captures_iter(&contents) {
                names.insert(caps[1].to_string());
            }
        }
    }

    let mut entries = Vec::new();
    for name in names.into_iter() {
        let hash = wasm_neovm::syscalls::lookup(&name)
            .map(|info| info.hash)
            .ok_or_else(|| anyhow::anyhow!("missing generated syscall for {name}"))?;
        entries.push(SyscallEntry { name, hash });
    }
    Ok(entries)
}

#[test]
fn neo_syscalls_match_reference() -> anyhow::Result<()> {
    let mut expected = parse_neovm_syscalls()?;
    expected.sort();

    let mut actual: Vec<SyscallEntry> = wasm_neovm::syscalls::all()
        .iter()
        .map(|info| SyscallEntry {
            name: info.name.to_string(),
            hash: info.hash,
        })
        .collect();
    actual.sort();

    assert_eq!(expected.len(), actual.len(), "syscall count mismatch");
    for (exp, act) in expected.iter().zip(actual.iter()) {
        assert_eq!(
            exp, act,
            "syscall mismatch: expected {:?}, got {:?}",
            exp, act
        );
    }

    Ok(())
}
