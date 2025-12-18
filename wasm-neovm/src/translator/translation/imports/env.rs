use super::*;

pub(in super::super) fn try_handle_env_import(
    import: &FunctionImport,
    func_type: &FuncType,
    params: &[StackValue],
    runtime: &mut RuntimeHelpers,
    script: &mut Vec<u8>,
    value_stack: &mut Vec<StackValue>,
) -> Result<bool> {
    if !import.module.eq_ignore_ascii_case("env") {
        return Ok(false);
    }

    let name = import.name.to_ascii_lowercase();
    let requires_return = !func_type.results().is_empty();
    if requires_return {
        if func_type.results().len() != 1 {
            bail!(
                "env import '{}::{}' must not return multiple values",
                import.module,
                import.name
            );
        }
        if func_type.results()[0] != ValType::I32 {
            bail!(
                "env import '{}::{}' returns unsupported type {:?}",
                import.module,
                import.name,
                func_type.results()[0]
            );
        }
    }

    let expect_params = || -> Result<()> {
        if func_type.params().len() != 3 || params.len() != 3 {
            bail!(
                "env import '{}::{}' expects three i32 parameters (dest, src/value, len)",
                import.module,
                import.name
            );
        }
        for ty in func_type.params() {
            if *ty != ValType::I32 {
                bail!(
                    "env import '{}::{}' parameter type {:?} is unsupported (expected i32)",
                    import.module,
                    import.name,
                    ty
                );
            }
        }
        Ok(())
    };

    match name.as_str() {
        "memcpy" | "__builtin_memcpy" => {
            expect_params()?;
            ensure_memory_access(runtime, 0)?;
            runtime.emit_memory_init_call(script)?;
            runtime.emit_env_memcpy_call(script)?;
        }
        "memmove" | "__builtin_memmove" => {
            expect_params()?;
            ensure_memory_access(runtime, 0)?;
            runtime.emit_memory_init_call(script)?;
            runtime.emit_env_memmove_call(script)?;
        }
        "memset" | "__builtin_memset" => {
            expect_params()?;
            ensure_memory_access(runtime, 0)?;
            runtime.emit_memory_init_call(script)?;
            runtime.emit_env_memset_call(script)?;
        }
        other => bail!(
            "env import '{}::{}' is not supported – compile with -nostdlib/-fno-builtin or provide a custom implementation",
            import.module,
            other
        ),
    }

    if requires_return {
        let dest_value = params.first().and_then(|value| value.const_value);
        value_stack.push(StackValue {
            const_value: dest_value,
            bytecode_start: None,
        });
    }

    Ok(true)
}
