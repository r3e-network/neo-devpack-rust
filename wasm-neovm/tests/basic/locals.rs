use wasm_neovm::translate_module;

#[test]
fn translate_param_local_get() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "id") (param i32) (result i32)
                local.get 0)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Id").expect("translation succeeds");
    let ldarg0 = wasm_neovm::opcodes::lookup("LDARG0").unwrap().byte;
    let starg0 = wasm_neovm::opcodes::lookup("STARG0").unwrap().byte;
    assert!(translation.script.iter().filter(|&&b| b == ldarg0).count() >= 2);
    assert!(translation.script.contains(&starg0));
    assert_eq!(translation.script.last().copied(), Some(0x40));
}

#[test]
fn translate_local_set_and_get() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "mirror") (param i32) (result i32)
                (local i32)
                local.get 0
                local.set 1
                local.get 1)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Mirror").expect("translation succeeds");
    let ldarg0 = wasm_neovm::opcodes::lookup("LDARG0").unwrap().byte;
    let stloc0 = wasm_neovm::opcodes::lookup("STLOC0").unwrap().byte;
    let ldloc0 = wasm_neovm::opcodes::lookup("LDLOC0").unwrap().byte;
    assert!(translation.script.contains(&ldarg0));
    assert!(translation.script.contains(&stloc0));
    assert!(translation.script.contains(&ldloc0));
    assert_eq!(translation.script.last().copied(), Some(0x40));
}

#[test]
fn translate_local_tee() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "tee") (param i32) (result i32)
                (local i32)
                local.get 0
                local.tee 1)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Tee").expect("translation succeeds");
    let ldarg0 = wasm_neovm::opcodes::lookup("LDARG0").unwrap().byte;
    let stloc0 = wasm_neovm::opcodes::lookup("STLOC0").unwrap().byte;
    let ldloc0 = wasm_neovm::opcodes::lookup("LDLOC0").unwrap().byte;
    assert!(translation.script.contains(&ldarg0));
    assert!(translation.script.contains(&stloc0));
    assert!(translation.script.contains(&ldloc0));
    assert_eq!(translation.script.last().copied(), Some(0x40));
}

#[test]
fn translate_i64_parameter() {
    let wasm = wat::parse_str(
        r#"(module
              (func (export "id64") (param i64) (result i64)
                local.get 0)
            )"#,
    )
    .expect("valid wat");

    let translation = translate_module(&wasm, "Id64").expect("translation succeeds");
    let ldarg0 = wasm_neovm::opcodes::lookup("LDARG0").unwrap().byte;
    let starg0 = wasm_neovm::opcodes::lookup("STARG0").unwrap().byte;
    assert!(translation.script.iter().filter(|&&b| b == ldarg0).count() >= 2);
    assert!(translation.script.contains(&starg0));
    assert_eq!(translation.script.last().copied(), Some(0x40));
}
