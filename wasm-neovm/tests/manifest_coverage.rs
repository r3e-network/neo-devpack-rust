use wasm_neovm::{
    metadata::{extract_nef_metadata, method_tokens_to_json, update_manifest_metadata},
    write_nef_with_metadata,
    translate_module_with_safe,
};

#[test]
fn manifest_and_nef_generation_round_trip() -> anyhow::Result<()> {
    let wasm = wat::parse_str(
        r#"(module
              (memory 1)
              (func (export "foo") (result i32)
                i32.const 42)
              (func (export "bar") (param i32) (result i32)
                local.get 0
                i32.const 1
                i32.add)
            )"#,
    )?;

    let translation = translate_module_with_safe(&wasm, "MyContract", &["foo"])?;
    let methods = translation.manifest.value["abi"]["methods"].clone();
    assert!(methods.as_array().unwrap().len() >= 2, "expected exported methods");
    assert!(translation.manifest.to_string()?.contains("MyContract"));

    // Extract metadata from manifest and ensure it feeds into NEF generation.
    let mut manifest_value = translation.manifest.value.clone();
    let metadata = extract_nef_metadata(&manifest_value)?;
    update_manifest_metadata(
        &mut manifest_value,
        metadata.source.as_deref(),
        &metadata.method_tokens,
    )?;

    // Ensure method tokens serialize as JSON array.
    let tokens_json = method_tokens_to_json(&metadata.method_tokens);
    assert!(tokens_json.is_array());

    // Write NEF and ensure checksum is appended.
    let tmp = tempfile::NamedTempFile::new()?;
    write_nef_with_metadata(
        &translation.script,
        metadata.source.as_deref(),
        &metadata.method_tokens,
        tmp.path(),
    )?;
    let bytes = std::fs::read(tmp.path())?;
    assert!(bytes.len() > translation.script.len(), "NEF should include header/footer");
    Ok(())
}
