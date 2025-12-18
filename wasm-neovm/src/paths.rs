use std::path::{Path, PathBuf};

pub(crate) fn derive_output_path(input: &Path, extension: &str) -> PathBuf {
    if input.file_name().is_some() {
        input.with_extension(extension)
    } else {
        let mut fallback = input.to_path_buf();
        fallback.push("contract");
        fallback.set_extension(extension);
        fallback
    }
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
