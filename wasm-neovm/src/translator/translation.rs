// Required imports
use anyhow::{anyhow, bail, Context, Result};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use wasmparser::{
    CompositeInnerType, DataKind, ExternalKind, FuncType, HeapType, Operator, Parser, Payload,
    RefType, TypeRef, ValType,
};

use crate::manifest::{merge_manifest, ManifestBuilder, ManifestMethod, ManifestParameter};
use crate::metadata::{
    dedup_method_tokens, extract_nef_metadata, parse_method_token_section, update_manifest_metadata,
};
use crate::nef::MethodToken;
use crate::neo_syscalls;
use crate::numeric;
use crate::opcodes;
use crate::syscalls;

use super::constants::*;
use super::helpers::*;
use super::runtime::{
    emit_bit_count, emit_select, emit_sign_extend, emit_zero_extend, ensure_memory_access,
    evaluate_global_init, evaluate_offset_expr, infer_contract_tokens, translate_data_drop,
    translate_memory_copy, translate_memory_fill, translate_memory_init, translate_memory_load,
    translate_memory_store, BitHelperKind, CallTarget, FunctionRegistry, RuntimeHelpers,
    StartDescriptor, StartKind, TableHelperKind, TableInfo,
};
use super::types::{StackValue, Translation, TranslationConfig};
use super::{FunctionImport, ModuleFrontend};

// ============================================================================
// Control Flow Types
// ============================================================================

/// Represents different kinds of control flow constructs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ControlKind {
    Block,
    Loop,
    If,
    Function,
}

/// Represents a control flow frame on the control stack
#[derive(Debug, Clone)]
struct ControlFrame {
    kind: ControlKind,
    stack_height: usize,
    result_count: usize, // Expected number of results (for Function and Block types)
    start_offset: usize,
    end_fixups: Vec<usize>,
    if_false_fixup: Option<usize>,
    has_else: bool,
}

#[derive(Debug, Default)]
struct ExportedFunction {
    names: Vec<ExportAlias>,
}

#[derive(Debug)]
struct ExportAlias {
    name: String,
    processed: bool,
}

pub fn translate_module(bytes: &[u8], contract_name: &str) -> Result<Translation> {
    translate_with_config(bytes, TranslationConfig::new(contract_name))
}

pub fn translate_with_config(bytes: &[u8], config: TranslationConfig) -> Result<Translation> {
    translate_module_internal(bytes, config)
}

fn translate_module_internal(bytes: &[u8], config: TranslationConfig) -> Result<Translation> {
    let contract_name = config.contract_name;
    let parser = Parser::new(0);
    let mut frontend = ModuleFrontend::new();
    let mut exported_funcs: BTreeMap<u32, ExportedFunction> = BTreeMap::new();
    let mut import_export_indices: BTreeSet<usize> = BTreeSet::new();
    let mut tables: Vec<TableInfo> = Vec::new();
    let mut script: Vec<u8> = Vec::new();
    let mut runtime = RuntimeHelpers::default();
    let mut methods: Vec<ManifestMethod> = Vec::new();
    let mut overlay_safe_methods: HashSet<String> = HashSet::new();
    let mut manifest_overlay: Option<Value> = None;
    let mut section_method_tokens: Vec<MethodToken> = Vec::new();
    let mut section_source: Option<String> = None;
    let mut saw_code_section = false;
    let mut next_defined_index: usize = 0;
    let mut function_registry: Option<FunctionRegistry> = None;
    let mut start_function: Option<u32> = None;
    let mut start_defined_offset: Option<usize> = None;

    for payload in parser.parse_all(bytes) {
        match payload? {
            Payload::Version { .. } => {}
            Payload::TypeSection(reader) => {
                for group in reader {
                    let group = group?;
                    for (_, subtype) in group.into_types_and_offsets() {
                        match subtype.composite_type.inner {
                            CompositeInnerType::Func(func) => frontend.register_signature(func),
                            _ => {}
                        }
                    }
                }
            }
            Payload::ImportSection(reader) => {
                for import in reader {
                    let import = import?;
                    match import.ty {
                        TypeRef::Func(type_index) => {
                            frontend.register_import(import.module, import.name, type_index);
                        }
                        TypeRef::Global(_) => {
                            bail!(
                                "global imports are not supported ({}::{})",
                                import.module,
                                import.name
                            );
                        }
                        _ => {
                            bail!("only function imports are supported (found non-function import {})", import.name);
                        }
                    }
                }
            }
            Payload::FunctionSection(reader) => {
                for idx in reader {
                    frontend.register_defined_function(idx?);
                }
            }
            Payload::TableSection(reader) => {
                for table in reader {
                    let table = table?;
                    if table.ty.table64 {
                        bail!("table64 is not supported");
                    }
                    if table.ty.shared {
                        bail!("shared tables are not supported");
                    }
                    if table.ty.element_type != RefType::FUNCREF {
                        bail!(
                            "reference type {:?} tables are not supported (NeoVM only models funcref handles; see docs/wasm-pipeline.md#9-unsupported-wasm-features)",
                            table.ty.element_type
                        );
                    }
                    let initial_len = usize::try_from(table.ty.initial)
                        .context("table initial size exceeds host limits")?;
                    let maximum = match table.ty.maximum {
                        Some(max) => {
                            Some(u32::try_from(max).context("table maximum exceeds 32-bit range")?)
                        }
                        None => None,
                    };
                    runtime.register_table(initial_len, maximum);
                    tables.push(TableInfo {
                        entries: vec![None; initial_len],
                    });
                }
            }
            Payload::GlobalSection(reader) => {
                for entry in reader {
                    let entry = entry?;
                    let value_type = entry.ty.content_type;
                    match value_type {
                        ValType::I32 | ValType::I64 => {}
                        other => bail!("only i32/i64 globals are supported (found {:?})", other),
                    }
                    let initial = evaluate_global_init(entry.init_expr, value_type)
                        .context("failed to evaluate global initialiser")?;
                    runtime.register_global(entry.ty.mutable, initial);
                }
            }
            Payload::ExportSection(reader) => {
                for export in reader {
                    let export = export?;
                    if export.kind == ExternalKind::Func {
                        let entry = exported_funcs
                            .entry(export.index)
                            .or_insert_with(ExportedFunction::default);
                        entry.names.push(ExportAlias {
                            name: export.name.to_string(),
                            processed: false,
                        });
                        if (export.index as usize) < frontend.import_len() {
                            import_export_indices.insert(export.index as usize);
                        }
                    }
                }
            }
            Payload::MemorySection(reader) => {
                for mem in reader {
                    let mem = mem?;
                    if mem.memory64 {
                        bail!("memory64 is not supported");
                    }
                    if mem.shared {
                        bail!("shared memories are not supported");
                    }
                    let initial = u32::try_from(mem.initial)
                        .context("memory initial size exceeds 32-bit range")?;
                    let maximum = match mem.maximum {
                        Some(max) => Some(
                            u32::try_from(max).context("memory maximum exceeds 32-bit range")?,
                        ),
                        None => None,
                    };
                    runtime
                        .set_memory_config(initial, maximum)
                        .context("failed to register memory section")?;
                }
            }
            Payload::CodeSectionStart { .. } => {
                saw_code_section = true;
                next_defined_index = 0;
                let total_functions =
                    frontend.import_len() + frontend.module_types().defined_functions_len();
                function_registry = Some(FunctionRegistry::new(total_functions));
            }
            Payload::CodeSectionEntry(body) => {
                let functions = function_registry
                    .as_mut()
                    .ok_or_else(|| anyhow!("code section encountered without initialisation"))?;
                let defined_index = next_defined_index;
                next_defined_index += 1;

                let func_index = frontend.import_len() + defined_index;
                let func_index_u32 = func_index as u32;
                let maybe_export = exported_funcs.get_mut(&func_index_u32);

                let function_name_owned = maybe_export
                    .as_ref()
                    .and_then(|entry| entry.names.first().map(|alias| alias.name.clone()))
                    .unwrap_or_else(|| format!("<internal:{}>", func_index));
                let function_name = function_name_owned.as_str();

                let type_index = frontend
                    .module_types()
                    .defined_type_index(defined_index)
                    .ok_or_else(|| {
                        anyhow!(
                            "no type index recorded for function '{}' (defined index {})",
                            function_name,
                            defined_index
                        )
                    })?;

                let func_type = frontend
                    .module_types()
                    .signature(type_index as usize)
                    .ok_or_else(|| {
                    anyhow!(
                        "type index {} referenced by function '{}' out of bounds",
                        type_index,
                        function_name
                    )
                })?;

                let offset = script.len();
                functions
                    .register_offset(&mut script, func_index, offset)
                    .context("failed to register function offset")?;

                if start_function == Some(func_index as u32) {
                    start_defined_offset = Some(offset);
                }

                let return_kind = translate_function(
                    func_type,
                    &body,
                    &mut script,
                    frontend.imports(),
                    frontend.module_types().signatures(),
                    frontend.module_types().defined_type_indices(),
                    &mut runtime,
                    &tables,
                    functions,
                    func_index,
                    start_function,
                    function_name,
                )
                .with_context(|| format!("failed to translate function '{}'", function_name))?;

                if let Some(entry) = maybe_export {
                    let parameter_defs: Vec<ManifestParameter> = func_type
                        .params()
                        .iter()
                        .enumerate()
                        .map(|(idx, param)| ManifestParameter {
                            name: format!("arg{}", idx),
                            kind: wasm_val_type_to_manifest(param)
                                .unwrap_or_else(|_| "Any".to_string()),
                        })
                        .collect();

                    for alias in entry.names.iter_mut() {
                        let method = ManifestMethod {
                            name: alias.name.clone(),
                            parameters: parameter_defs.clone(),
                            return_type: return_kind.clone(),
                            offset: offset as u32,
                            safe: false,
                        };
                        methods.push(method);
                        alias.processed = true;
                    }
                }
            }
            Payload::ElementSection(reader) => {
                for element in reader {
                    let element = element?;

                    let (table_index_opt, offset_opt) = match element.kind {
                        wasmparser::ElementKind::Active {
                            table_index,
                            offset_expr,
                        } => {
                            let table_idx = table_index.unwrap_or(0) as usize;
                            let offset = evaluate_offset_expr(offset_expr)
                                .context("failed to evaluate element offset")?;
                            if offset < 0 {
                                bail!("element segment offset must be non-negative");
                            }
                            (Some(table_idx), Some(offset as usize))
                        }
                        wasmparser::ElementKind::Passive => (None, None),
                        wasmparser::ElementKind::Declared => {
                            bail!("declared element segments are not supported")
                        }
                    };

                    let mut func_refs: Vec<Option<u32>> = Vec::new();
                    match element.items {
                        wasmparser::ElementItems::Functions(funcs) => {
                            for func in funcs {
                                func_refs.push(Some(func?));
                            }
                        }
                        wasmparser::ElementItems::Expressions(ref_ty, exprs) => {
                            if ref_ty != RefType::FUNCREF {
                                bail!(
                                    "element expressions for reference type {:?} are not supported (NeoVM only models funcref handles; see docs/wasm-pipeline.md#9-unsupported-wasm-features)",
                                    ref_ty
                                );
                            }
                            for expr in exprs {
                                let expr = expr?;
                                let mut reader = expr.get_operators_reader();
                                let mut value: Option<Option<u32>> = None;
                                while !reader.eof() {
                                    let op = reader.read()?;
                                    match op {
                                        Operator::RefNull { hty } => {
                                            if hty != HeapType::FUNC {
                                                bail!(
                                                    "element expression uses unsupported heap type {:?}",
                                                    hty
                                                );
                                            }
                                            value = Some(None);
                                        }
                                        Operator::RefFunc { function_index } => {
                                            value = Some(Some(function_index));
                                        }
                                        Operator::End => break,
                                        other => {
                                            bail!(
                                                "unsupported instruction {:?} in element segment expression",
                                                other
                                            );
                                        }
                                    }
                                }
                                let parsed = value.ok_or_else(|| {
                                    anyhow!("element expression did not yield a value")
                                })?;
                                func_refs.push(parsed);
                            }
                        }
                    }

                    let values_i32: Vec<i32> = func_refs
                        .iter()
                        .map(|opt| opt.map(|v| v as i32).unwrap_or(FUNCREF_NULL as i32))
                        .collect();

                    if let Some(table_index) = table_index_opt {
                        let offset = offset_opt.expect("active element requires offset");
                        let table = tables.get_mut(table_index).ok_or_else(|| {
                            anyhow!("element references missing table index {}", table_index)
                        })?;

                        for (i, func_ref) in func_refs.iter().enumerate() {
                            let slot = offset
                                .checked_add(i)
                                .ok_or_else(|| anyhow!("element segment offset overflow"))?;
                            if slot >= table.entries.len() {
                                bail!(
                                    "element segment writes past table bounds (index {}, table length {})",
                                    slot,
                                    table.entries.len()
                                );
                            }
                            table.entries[slot] = *func_ref;
                        }

                        runtime
                            .register_active_element(table_index, offset, values_i32)
                            .context("failed to register active element segment")?;
                    } else {
                        runtime.register_passive_element(values_i32);
                    }
                }
            }
            Payload::DataSection(reader) => {
                for entry in reader {
                    let entry = entry?;
                    let data_bytes = entry.data.to_vec();
                    match entry.kind {
                        DataKind::Passive => {
                            runtime
                                .register_passive_segment(data_bytes)
                                .context("failed to register passive data segment")?;
                        }
                        DataKind::Active {
                            memory_index,
                            offset_expr,
                        } => {
                            if memory_index != 0 {
                                bail!("only default memory index 0 is supported for active data segments");
                            }
                            let offset = evaluate_offset_expr(offset_expr)
                                .context("failed to evaluate active data segment offset")?;
                            if offset < 0 {
                                bail!("active data segment offset must be non-negative");
                            }
                            runtime
                                .register_active_segment(memory_index, offset as u64, data_bytes)
                                .context("failed to register active data segment")?;
                        }
                    }
                }
            }
            Payload::CustomSection(section) => match classify_custom_section(section.name()) {
                Some(CustomSectionKind::Manifest) => {
                    for overlay in parse_concatenated_json(section.data(), "neo.manifest")? {
                        collect_safe_methods(&overlay, &mut overlay_safe_methods);
                        if let Some(existing) = manifest_overlay.as_mut() {
                            merge_manifest(existing, &overlay);
                        } else {
                            manifest_overlay = Some(overlay);
                        }
                    }
                }
                Some(CustomSectionKind::MethodTokens) => {
                    let fragments = parse_concatenated_json(section.data(), "neo.methodtokens")?;
                    for fragment in fragments {
                        let bytes = serde_json::to_vec(&fragment)
                            .context("failed to serialize neo.methodtokens fragment")?;
                        let metadata = parse_method_token_section(&bytes)
                            .context("failed to parse neo.methodtokens custom section fragment")?;
                        if section_source.is_none() {
                            section_source = metadata.source.clone();
                        }
                        section_method_tokens.extend(metadata.method_tokens);
                    }
                }
                None => {}
            },
            Payload::StartSection { func, .. } => {
                if start_function.is_some() {
                    bail!("module contains multiple start sections");
                }
                start_function = Some(func);
            }
            Payload::TagSection(_)
            | Payload::DataCountSection { .. }
            | Payload::UnknownSection { .. } => {}
            Payload::End(_) => break,
            _ => {}
        }
    }

    if !saw_code_section && import_export_indices.is_empty() {
        bail!("input module does not contain a code section");
    }

    for &import_idx in &import_export_indices {
        let Some(entry) = exported_funcs.get_mut(&(import_idx as u32)) else {
            continue;
        };
        if !entry.names.iter().any(|alias| !alias.processed) {
            continue;
        }

        let import = frontend
            .imports()
            .get(import_idx)
            .ok_or_else(|| {
            anyhow!(
                "export references missing import function index {}",
                import_idx
            )
        })?;
        let type_index = get_import_type_index(import)?;
        let func_type = frontend
            .module_types()
            .signature(type_index as usize)
            .ok_or_else(|| {
                anyhow!(
                "invalid type index {} for import {}::{}",
                type_index,
                import.module,
                import.name
            )
        })?;

        if func_type.results().len() > 1 {
            bail!(
                "multi-value returns are not supported for exported import {}::{}",
                import.module,
                import.name
            );
        }

        let parameter_defs: Vec<ManifestParameter> = func_type
            .params()
            .iter()
            .enumerate()
            .map(|(idx, param)| ManifestParameter {
                name: format!("arg{}", idx),
                kind: wasm_val_type_to_manifest(param).unwrap_or_else(|_| "Any".to_string()),
            })
            .collect();

        let offset = script.len();
        let return_kind = emit_import_export_stub(
            &mut script,
            &mut runtime,
            frontend.imports(),
            frontend.module_types().signatures(),
            import_idx,
        )
        .with_context(|| {
            format!(
                "failed to synthesise export stub for import {}::{}",
                import.module, import.name
            )
        })?;

        for alias in entry.names.iter_mut() {
            if alias.processed {
                continue;
            }
            methods.push(ManifestMethod {
                name: alias.name.clone(),
                parameters: parameter_defs.clone(),
                return_type: return_kind.clone(),
                offset: offset as u32,
                safe: false,
            });
            alias.processed = true;
        }
    }

    let mut missing: Vec<String> = exported_funcs
        .values()
        .flat_map(|entry| {
            entry
                .names
                .iter()
                .filter(|alias| !alias.processed)
                .map(|alias| alias.name.clone())
        })
        .collect();
    missing.sort_unstable();

    if !missing.is_empty() {
        bail!(
            "did not translate exported functions: {}",
            missing.join(", ")
        );
    }

    if methods.is_empty() {
        bail!(
            "no exportable functions were translated – ensure functions are exported and meet translation constraints"
        );
    }

    for method in &mut methods {
        if overlay_safe_methods.remove(&method.name) {
            method.safe = true;
        }
    }
    if !overlay_safe_methods.is_empty() {
        let mut missing: Vec<String> = overlay_safe_methods.into_iter().collect();
        missing.sort_unstable();
        bail!(
            "manifest overlays marked the following methods safe but they were not exported: {}",
            missing.join(", ")
        );
    }

    let start_descriptor = if let Some(start_idx) = start_function {
        if (start_idx as usize) < frontend.import_len() {
            let import = frontend
                .imports()
                .get(start_idx as usize)
                .ok_or_else(|| anyhow!("start section references missing import {}", start_idx))?;
            let type_index = get_import_type_index(import)?;
            let func_type = frontend
                .module_types()
                .signature(type_index as usize)
                .ok_or_else(|| {
                    anyhow!(
                        "invalid type index {} for start import {}::{}",
                        type_index,
                        import.module,
                        import.name
                    )
                })?;
            if !func_type.params().is_empty() {
                bail!("start function must not take parameters");
            }
            if !func_type.results().is_empty() {
                bail!("start function must not return values");
            }
            Some(StartDescriptor {
                function_index: start_idx,
                kind: StartKind::Import,
            })
        } else {
            let defined_index = (start_idx as usize)
                .checked_sub(frontend.import_len())
                .ok_or_else(|| anyhow!("start function index underflow"))?;
            let type_index = frontend
                .module_types()
                .defined_type_index(defined_index)
                .ok_or_else(|| anyhow!("no type index recorded for start function"))?;
            let func_type = frontend
                .module_types()
                .signature(type_index as usize)
                .ok_or_else(|| anyhow!("invalid type index {} for start function", type_index))?;
            if !func_type.params().is_empty() {
                bail!("start function must not take parameters");
            }
            if !func_type.results().is_empty() {
                bail!("start function must not return values");
            }
            let offset = start_defined_offset.ok_or_else(|| {
                anyhow!(
                    "failed to record offset for start function; ensure code section is present"
                )
            })?;
            Some(StartDescriptor {
                function_index: start_idx,
                kind: StartKind::Defined { offset },
            })
        }
    } else {
        None
    };

    runtime.finalize(
        &mut script,
        start_descriptor.as_ref(),
        frontend.imports(),
        frontend.module_types().signatures(),
    )?;

    let mut manifest_builder = ManifestBuilder::new(contract_name, &methods);
    if let Some(overlay) = manifest_overlay {
        manifest_builder.merge_overlay(
            &overlay,
            Some("embedded neo.manifest sections".to_string()),
        );
    }
    if let Some(extra) = config.extra_manifest_overlay {
        manifest_builder.merge_overlay(&extra.value, extra.label);
    }
    manifest_builder.propagate_safe_flags();
    manifest_builder.ensure_method_parity()?;

    let mut metadata = extract_nef_metadata(manifest_builder.manifest_value())?;
    metadata.method_tokens.extend(section_method_tokens);
    let inferred_tokens = infer_contract_tokens(&script);
    metadata.method_tokens.extend(inferred_tokens);
    dedup_method_tokens(&mut metadata.method_tokens);
    if metadata.source.is_none() {
        metadata.source = section_source;
    }

    update_manifest_metadata(
        manifest_builder.manifest_value_mut(),
        metadata.source.as_deref(),
        &metadata.method_tokens,
    )?;

    Ok(Translation {
        script,
        manifest: manifest_builder.into_rendered(),
        method_tokens: metadata.method_tokens.clone(),
        source_url: metadata.source.clone(),
    })
}

fn emit_import_export_stub(
    script: &mut Vec<u8>,
    runtime: &mut RuntimeHelpers,
    imports: &[FunctionImport],
    types: &[FuncType],
    import_index: usize,
) -> Result<String> {
    let import = imports
        .get(import_index)
        .ok_or_else(|| anyhow!("import index {} out of range", import_index))?;
    let type_index = get_import_type_index(import)?;
    let func_type = types.get(type_index as usize).ok_or_else(|| {
        anyhow!(
            "invalid type index {} for import {}::{}",
            type_index,
            import.module,
            import.name
        )
    })?;

    for ty in func_type.params() {
        match ty {
            ValType::I32 | ValType::I64 => {}
            other => bail!(
                "import '{}::{}' exported with unsupported parameter type {:?}",
                import.module,
                import.name,
                other
            ),
        }
    }

    if func_type.results().len() > 1 {
        bail!(
            "import '{}::{}' exported with unsupported multi-value return",
            import.module,
            import.name
        );
    }

    let mut params_stack: Vec<StackValue> = Vec::with_capacity(func_type.params().len());
    for (idx, _) in func_type.params().iter().enumerate() {
        emit_load_arg(script, idx as u32)?;
        params_stack.push(StackValue {
            const_value: None,
            bytecode_start: None,
        });
    }

    let mut synthetic_stack: Vec<StackValue> = Vec::new();
    if try_handle_env_import(
        import,
        func_type,
        &params_stack,
        runtime,
        script,
        &mut synthetic_stack,
    )? {
        script.push(RET);
        let return_kind = func_type
            .results()
            .first()
            .map(|ty| wasm_val_type_to_manifest(ty))
            .transpose()?
            .unwrap_or_else(|| "Void".to_string());
        return Ok(return_kind);
    }

    handle_import_call(import_index as u32, script, imports, types, &params_stack)?;

    script.push(RET);

    let return_kind = func_type
        .results()
        .first()
        .map(|ty| wasm_val_type_to_manifest(ty))
        .transpose()?
        .unwrap_or_else(|| "Void".to_string());

    Ok(return_kind)
}

fn collect_safe_methods(value: &Value, accumulator: &mut HashSet<String>) {
    match value {
        Value::Object(map) => {
            if let Some(methods) = map
                .get("abi")
                .and_then(Value::as_object)
                .and_then(|abi| abi.get("methods"))
                .and_then(Value::as_array)
            {
                for method in methods {
                    if method.get("safe").and_then(Value::as_bool).unwrap_or(false) {
                        if let Some(name) = method.get("name").and_then(Value::as_str) {
                            accumulator.insert(name.to_string());
                        }
                    }
                }
            }
            for child in map.values() {
                collect_safe_methods(child, accumulator);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_safe_methods(item, accumulator);
            }
        }
        _ => {}
    }
}

fn translate_function(
    func_type: &FuncType,
    body: &wasmparser::FunctionBody,
    script: &mut Vec<u8>,
    imports: &[FunctionImport],
    types: &[FuncType],
    func_type_indices: &[u32],
    runtime: &mut RuntimeHelpers,
    tables: &[TableInfo],
    functions: &mut FunctionRegistry,
    function_index: usize,
    start_function: Option<u32>,
    function_name: &str,
) -> Result<String> {
    let params = func_type.params();
    for ty in params {
        match ty {
            ValType::I32 | ValType::I64 => {}
            other => bail!("only i32/i64 parameters are supported (found {:?})", other),
        }
    }
    let param_count = params.len();

    let returns = func_type.results();
    if returns.len() > 1 {
        bail!("multi-value returns are not supported");
    }

    if let Some(start_idx) = start_function {
        if start_idx as usize != function_index {
            runtime.emit_memory_init_call(script)?;
        }
    }

    let return_kind = returns
        .first()
        .map(|ty| wasm_val_type_to_manifest(ty))
        .transpose()?;

    let locals_reader = body.get_locals_reader()?;
    let mut local_states: Vec<LocalState> = Vec::new();
    for i in 0..param_count {
        local_states.push(LocalState {
            kind: LocalKind::Param(i as u32),
            const_value: None,
        });
    }

    let mut next_local_slot: u32 = 0;
    for entry in locals_reader {
        let (count, ty) = entry?;
        if ty != ValType::I32 && ty != ValType::I64 {
            bail!("only i32/i64 locals are supported (found {:?})", ty);
        }
        for _ in 0..count {
            local_states.push(LocalState {
                kind: LocalKind::Local(next_local_slot),
                const_value: Some(0),
            });
            next_local_slot += 1;
        }
    }

    let mut emitted_ret = false;
    let op_reader = body.get_operators_reader()?;
    let mut value_stack: Vec<StackValue> = Vec::new();
    let mut control_stack: Vec<ControlFrame> = Vec::new();
    let mut is_unreachable = false;

    // Push implicit function-level control frame
    // In WASM, the function body itself is an implicit block that can be targeted by branches
    // stack_height is 0 because branches to the function can occur at any point
    // result_count tracks how many values must be on stack when branching to function exit
    control_stack.push(ControlFrame {
        kind: ControlKind::Function,
        stack_height: 0,
        result_count: returns.len(), // Function expects return values
        start_offset: script.len(),
        end_fixups: Vec::new(),
        if_false_fixup: None,
        has_else: false,
    });

    // Ensure the current function offset is known to callers (already registered before entry).
    // This assertion helps catch internal misuse during development.
    if !functions.contains_index(function_index) {
        bail!(
            "function index {} out of range for translation",
            function_index
        );
    }

    for op in op_reader {
        let op = op?;
        match op {
            Operator::Nop => {}
            Operator::I32Const { value } => {
                let entry = emit_push_int(script, value as i128);
                value_stack.push(entry);
            }
            Operator::I64Const { value } => {
                let entry = emit_push_int(script, value as i128);
                value_stack.push(entry);
            }
            Operator::I32Clz => {
                let value = pop_value(&mut value_stack, "i32.clz operand")?;
                let result = emit_bit_count(script, runtime, value, BitHelperKind::Clz(32))?;
                value_stack.push(result);
            }
            Operator::I32Ctz => {
                let value = pop_value(&mut value_stack, "i32.ctz operand")?;
                let result = emit_bit_count(script, runtime, value, BitHelperKind::Ctz(32))?;
                value_stack.push(result);
            }
            Operator::I32Popcnt => {
                let value = pop_value(&mut value_stack, "i32.popcnt operand")?;
                let result = emit_bit_count(script, runtime, value, BitHelperKind::Popcnt(32))?;
                value_stack.push(result);
            }
            Operator::I64Clz => {
                let value = pop_value(&mut value_stack, "i64.clz operand")?;
                let result = emit_bit_count(script, runtime, value, BitHelperKind::Clz(64))?;
                value_stack.push(result);
            }
            Operator::I64Ctz => {
                let value = pop_value(&mut value_stack, "i64.ctz operand")?;
                let result = emit_bit_count(script, runtime, value, BitHelperKind::Ctz(64))?;
                value_stack.push(result);
            }
            Operator::I64Popcnt => {
                let value = pop_value(&mut value_stack, "i64.popcnt operand")?;
                let result = emit_bit_count(script, runtime, value, BitHelperKind::Popcnt(64))?;
                value_stack.push(result);
            }
            Operator::I32Add => {
                let rhs = pop_value(&mut value_stack, "i32.add rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.add lhs")?;
                let result = emit_binary_op(script, "ADD", lhs, rhs, |a, b| {
                    let lhs = a as i32;
                    let rhs = b as i32;
                    Some(lhs.wrapping_add(rhs) as i128)
                })?;
                value_stack.push(result);
            }
            Operator::I64Add => {
                let rhs = pop_value(&mut value_stack, "i64.add rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.add lhs")?;
                let result = emit_binary_op(script, "ADD", lhs, rhs, |a, b| {
                    let lhs = a as i64;
                    let rhs = b as i64;
                    Some(lhs.wrapping_add(rhs) as i128)
                })?;
                value_stack.push(result);
            }
            Operator::I32Sub => {
                let rhs = pop_value(&mut value_stack, "i32.sub rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.sub lhs")?;
                let result = emit_binary_op(script, "SUB", lhs, rhs, |a, b| {
                    let lhs = a as i32;
                    let rhs = b as i32;
                    Some(lhs.wrapping_sub(rhs) as i128)
                })?;
                value_stack.push(result);
            }
            Operator::I64Sub => {
                let rhs = pop_value(&mut value_stack, "i64.sub rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.sub lhs")?;
                let result = emit_binary_op(script, "SUB", lhs, rhs, |a, b| {
                    let lhs = a as i64;
                    let rhs = b as i64;
                    Some(lhs.wrapping_sub(rhs) as i128)
                })?;
                value_stack.push(result);
            }
            Operator::I32Mul => {
                let rhs = pop_value(&mut value_stack, "i32.mul rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.mul lhs")?;
                let result = emit_binary_op(script, "MUL", lhs, rhs, |a, b| {
                    let lhs = a as i32;
                    let rhs = b as i32;
                    Some(lhs.wrapping_mul(rhs) as i128)
                })?;
                value_stack.push(result);
            }
            Operator::I32And => {
                let rhs = pop_value(&mut value_stack, "i32.and rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.and lhs")?;
                let result = emit_binary_op(script, "AND", lhs, rhs, |a, b| {
                    Some(((a as i32) & (b as i32)) as i128)
                })?;
                value_stack.push(result);
            }
            Operator::I32Or => {
                let rhs = pop_value(&mut value_stack, "i32.or rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.or lhs")?;
                let result = emit_binary_op(script, "OR", lhs, rhs, |a, b| {
                    Some(((a as i32) | (b as i32)) as i128)
                })?;
                value_stack.push(result);
            }
            Operator::I32Xor => {
                let rhs = pop_value(&mut value_stack, "i32.xor rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.xor lhs")?;
                let result = emit_binary_op(script, "XOR", lhs, rhs, |a, b| {
                    Some(((a as i32) ^ (b as i32)) as i128)
                })?;
                value_stack.push(result);
            }
            Operator::I32Shl => {
                let rhs = pop_value(&mut value_stack, "i32.shl rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.shl lhs")?;
                let result = emit_binary_op(script, "SHL", lhs, rhs, |a, b| {
                    let shift = (b as u32) & 31;
                    Some(((a as i32) << shift) as i128)
                })?;
                value_stack.push(result);
            }
            Operator::I32ShrS => {
                let rhs = pop_value(&mut value_stack, "i32.shr_s rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.shr_s lhs")?;
                let result = emit_shift_right(script, lhs, rhs, 32, ShiftKind::Arithmetic)?;
                value_stack.push(result);
            }
            Operator::I32ShrU => {
                let rhs = pop_value(&mut value_stack, "i32.shr_u rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.shr_u lhs")?;
                let result = emit_shift_right(script, lhs, rhs, 32, ShiftKind::Logical)?;
                value_stack.push(result);
            }
            Operator::I32Rotl => {
                let rhs = pop_value(&mut value_stack, "i32.rotl rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.rotl lhs")?;
                let result = emit_rotate(script, lhs, rhs, 32, true)?;
                value_stack.push(result);
            }
            Operator::I32Rotr => {
                let rhs = pop_value(&mut value_stack, "i32.rotr rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.rotr lhs")?;
                let result = emit_rotate(script, lhs, rhs, 32, false)?;
                value_stack.push(result);
            }
            Operator::I64Mul => {
                let rhs = pop_value(&mut value_stack, "i64.mul rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.mul lhs")?;
                let result = emit_binary_op(script, "MUL", lhs, rhs, |a, b| {
                    let lhs = a as i64;
                    let rhs = b as i64;
                    Some(lhs.wrapping_mul(rhs) as i128)
                })?;
                value_stack.push(result);
            }
            Operator::I64And => {
                let rhs = pop_value(&mut value_stack, "i64.and rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.and lhs")?;
                let result = emit_binary_op(script, "AND", lhs, rhs, |a, b| {
                    Some(((a as i64) & (b as i64)) as i128)
                })?;
                value_stack.push(result);
            }
            Operator::I64Or => {
                let rhs = pop_value(&mut value_stack, "i64.or rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.or lhs")?;
                let result = emit_binary_op(script, "OR", lhs, rhs, |a, b| {
                    Some(((a as i64) | (b as i64)) as i128)
                })?;
                value_stack.push(result);
            }
            Operator::I64Xor => {
                let rhs = pop_value(&mut value_stack, "i64.xor rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.xor lhs")?;
                let result = emit_binary_op(script, "XOR", lhs, rhs, |a, b| {
                    Some(((a as i64) ^ (b as i64)) as i128)
                })?;
                value_stack.push(result);
            }
            Operator::I64Shl => {
                let rhs = pop_value(&mut value_stack, "i64.shl rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.shl lhs")?;
                let result = emit_binary_op(script, "SHL", lhs, rhs, |a, b| {
                    let shift = (b as u32) & 63;
                    Some(((a as i64) << shift) as i128)
                })?;
                value_stack.push(result);
            }
            Operator::I64ShrS => {
                let rhs = pop_value(&mut value_stack, "i64.shr_s rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.shr_s lhs")?;
                let result = emit_shift_right(script, lhs, rhs, 64, ShiftKind::Arithmetic)?;
                value_stack.push(result);
            }
            Operator::I64ShrU => {
                let rhs = pop_value(&mut value_stack, "i64.shr_u rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.shr_u lhs")?;
                let result = emit_shift_right(script, lhs, rhs, 64, ShiftKind::Logical)?;
                value_stack.push(result);
            }
            Operator::I64Rotl => {
                let rhs = pop_value(&mut value_stack, "i64.rotl rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.rotl lhs")?;
                let result = emit_rotate(script, lhs, rhs, 64, true)?;
                value_stack.push(result);
            }
            Operator::I64Rotr => {
                let rhs = pop_value(&mut value_stack, "i64.rotr rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.rotr lhs")?;
                let result = emit_rotate(script, lhs, rhs, 64, false)?;
                value_stack.push(result);
            }
            Operator::I32DivS => {
                let rhs = pop_value(&mut value_stack, "i32.div_s rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.div_s lhs")?;
                let result = emit_binary_op(script, "DIV", lhs, rhs, |a, b| {
                    let dividend = a as i32;
                    let divisor = b as i32;
                    if divisor == 0 {
                        return None;
                    }
                    if dividend == i32::MIN && divisor == -1 {
                        return None;
                    }
                    Some((dividend / divisor) as i128)
                })?;
                value_stack.push(result);
            }
            Operator::I32DivU => {
                let rhs = pop_value(&mut value_stack, "i32.div_u rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.div_u lhs")?;
                let result = emit_unsigned_binary_op(script, UnsignedOp::Div, lhs, rhs, 32)?;
                value_stack.push(result);
            }
            Operator::I32RemS => {
                let rhs = pop_value(&mut value_stack, "i32.rem_s rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.rem_s lhs")?;
                let result = emit_binary_op(script, "MOD", lhs, rhs, |a, b| {
                    let dividend = a as i32;
                    let divisor = b as i32;
                    if divisor == 0 {
                        return None;
                    }
                    Some((dividend % divisor) as i128)
                })?;
                value_stack.push(result);
            }
            Operator::I32RemU => {
                let rhs = pop_value(&mut value_stack, "i32.rem_u rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.rem_u lhs")?;
                let result = emit_unsigned_binary_op(script, UnsignedOp::Rem, lhs, rhs, 32)?;
                value_stack.push(result);
            }
            Operator::I64DivS => {
                let rhs = pop_value(&mut value_stack, "i64.div_s rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.div_s lhs")?;
                let result = emit_binary_op(script, "DIV", lhs, rhs, |a, b| {
                    let dividend = a as i64;
                    let divisor = b as i64;
                    if divisor == 0 {
                        return None;
                    }
                    if dividend == i64::MIN && divisor == -1 {
                        return None;
                    }
                    Some((dividend / divisor) as i128)
                })?;
                value_stack.push(result);
            }
            Operator::I64DivU => {
                let rhs = pop_value(&mut value_stack, "i64.div_u rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.div_u lhs")?;
                let result = emit_unsigned_binary_op(script, UnsignedOp::Div, lhs, rhs, 64)?;
                value_stack.push(result);
            }
            Operator::I64RemS => {
                let rhs = pop_value(&mut value_stack, "i64.rem_s rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.rem_s lhs")?;
                let result = emit_binary_op(script, "MOD", lhs, rhs, |a, b| {
                    let dividend = a as i64;
                    let divisor = b as i64;
                    if divisor == 0 {
                        return None;
                    }
                    Some((dividend % divisor) as i128)
                })?;
                value_stack.push(result);
            }
            Operator::I64RemU => {
                let rhs = pop_value(&mut value_stack, "i64.rem_u rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.rem_u lhs")?;
                let result = emit_unsigned_binary_op(script, UnsignedOp::Rem, lhs, rhs, 64)?;
                value_stack.push(result);
            }
            Operator::I32WrapI64 => {
                let value = pop_value(&mut value_stack, "i32.wrap_i64 operand")?;
                let result = emit_sign_extend(script, value, 32, 32)?;
                value_stack.push(result);
            }
            Operator::I64ExtendI32U => {
                let value = pop_value(&mut value_stack, "i64.extend_i32_u operand")?;
                let result = emit_zero_extend(script, value, 32)?;
                value_stack.push(result);
            }
            Operator::I64ExtendI32S => {
                let value = pop_value(&mut value_stack, "i64.extend_i32_s operand")?;
                let result = emit_sign_extend(script, value, 32, 64)?;
                value_stack.push(result);
            }
            Operator::I32Extend8S => {
                let value = pop_value(&mut value_stack, "i32.extend8_s operand")?;
                let result = emit_sign_extend(script, value, 8, 32)?;
                value_stack.push(result);
            }
            Operator::I32Extend16S => {
                let value = pop_value(&mut value_stack, "i32.extend16_s operand")?;
                let result = emit_sign_extend(script, value, 16, 32)?;
                value_stack.push(result);
            }
            Operator::I64Extend8S => {
                let value = pop_value(&mut value_stack, "i64.extend8_s operand")?;
                let result = emit_sign_extend(script, value, 8, 64)?;
                value_stack.push(result);
            }
            Operator::I64Extend16S => {
                let value = pop_value(&mut value_stack, "i64.extend16_s operand")?;
                let result = emit_sign_extend(script, value, 16, 64)?;
                value_stack.push(result);
            }
            Operator::I64Extend32S => {
                let value = pop_value(&mut value_stack, "i64.extend32_s operand")?;
                let result = emit_sign_extend(script, value, 32, 64)?;
                value_stack.push(result);
            }
            Operator::Block { .. } => {
                control_stack.push(ControlFrame {
                    kind: ControlKind::Block,
                    stack_height: value_stack.len(),
                    result_count: 0,  // Blocks don't affect function return
                    start_offset: script.len(),
                    end_fixups: Vec::new(),
                    if_false_fixup: None,
                    has_else: false,
                });
                // Entering a new block resets unreachable state
                is_unreachable = false;
            }
            Operator::Loop { .. } => {
                control_stack.push(ControlFrame {
                    kind: ControlKind::Loop,
                    stack_height: value_stack.len(),
                    result_count: 0,  // Loops don't affect function return
                    start_offset: script.len(),
                    end_fixups: Vec::new(),
                    if_false_fixup: None,
                    has_else: false,
                });
                // Entering a new loop resets unreachable state
                is_unreachable = false;
            }
            Operator::If { .. } => {
                let _cond = pop_value(&mut value_stack, "if condition")?;
                // Condition already materialised on stack
                let jump_pos = emit_jump_placeholder(script, "JMPIFNOT_L")?;
                control_stack.push(ControlFrame {
                    kind: ControlKind::If,
                    stack_height: value_stack.len(),
                    result_count: 0,  // If blocks don't affect function return
                    start_offset: script.len(),
                    end_fixups: Vec::new(),
                    if_false_fixup: Some(jump_pos),
                    has_else: false,
                });
                // Entering IF resets unreachable state
                is_unreachable = false;
            }
            Operator::Else => {
                let frame = control_stack
                    .last_mut()
                    .ok_or_else(|| anyhow!("ELSE without matching IF"))?;
                if !matches!(frame.kind, ControlKind::If) {
                    bail!("ELSE can only appear within an IF block");
                }
                if let Some(pos) = frame.if_false_fixup.take() {
                    patch_jump(script, pos, script.len())?;
                }
                // Jump over else body when the THEN branch executes
                let jump_end = emit_jump_placeholder(script, "JMP_L")?;
                frame.end_fixups.push(jump_end);
                frame.has_else = true;
                frame.start_offset = script.len();
                value_stack.truncate(frame.stack_height);
                // Entering ELSE resets unreachable state
                is_unreachable = false;
            }
            Operator::End => {
                let frame = control_stack
                    .pop()
                    .ok_or_else(|| anyhow!("END without matching block"))?;

                match frame.kind {
                    ControlKind::If => {
                        if let Some(pos) = frame.if_false_fixup {
                            patch_jump(script, pos, script.len())?;
                        }
                    }
                    ControlKind::Loop | ControlKind::Block => {}
                    ControlKind::Function => {
                        // This is the final END of the function
                        // Don't truncate value stack or patch fixups for function-level frame
                        continue;
                    }
                }
                for fixup in frame.end_fixups {
                    patch_jump(script, fixup, script.len())?;
                }
                value_stack.truncate(frame.stack_height);
                // Code after END is reachable again
                is_unreachable = false;
            }
            Operator::Br { relative_depth } => {
                handle_branch(
                    script,
                    &mut value_stack,
                    &mut control_stack,
                    relative_depth as usize,
                    false,
                    &mut is_unreachable,
                )?;
            }
            Operator::BrIf { relative_depth } => {
                let _cond = pop_value(&mut value_stack, "br_if condition")?;
                handle_branch(
                    script,
                    &mut value_stack,
                    &mut control_stack,
                    relative_depth as usize,
                    true,
                    &mut is_unreachable,
                )?;
            }
            Operator::BrTable { targets } => {
                let index = pop_value(&mut value_stack, "br_table index")?;
                let mut target_depths: Vec<usize> = Vec::with_capacity(targets.len() as usize);
                for target in targets.targets() {
                    target_depths.push(target? as usize);
                }
                let default_depth = targets.default() as usize;
                handle_br_table(
                    script,
                    &mut value_stack,
                    &mut control_stack,
                    index,
                    &target_depths,
                    default_depth,
                    &mut is_unreachable,
                )?;
            }
            Operator::MemorySize { mem, .. } => {
                ensure_memory_access(runtime, mem)?;
                runtime.emit_memory_init_call(script)?;
                script.push(lookup_opcode("LDSFLD2")?.byte);
                value_stack.push(StackValue {
                    const_value: None,
                    bytecode_start: None,
                });
            }
            Operator::I32Load { memarg, .. } => {
                let mem = memarg.memory;
                let offset = memarg.offset;
                let addr = pop_value(&mut value_stack, "i32.load address")?;
                translate_memory_load(
                    script,
                    runtime,
                    &mut value_stack,
                    addr,
                    mem,
                    offset,
                    4,
                    None,
                    32,
                    "i32.load",
                )?;
            }
            Operator::I64Load { memarg, .. } => {
                let mem = memarg.memory;
                let offset = memarg.offset;
                let addr = pop_value(&mut value_stack, "i64.load address")?;
                translate_memory_load(
                    script,
                    runtime,
                    &mut value_stack,
                    addr,
                    mem,
                    offset,
                    8,
                    None,
                    64,
                    "i64.load",
                )?;
            }
            Operator::I32Load8S { memarg, .. } => {
                let mem = memarg.memory;
                let offset = memarg.offset;
                let addr = pop_value(&mut value_stack, "i32.load8_s address")?;
                translate_memory_load(
                    script,
                    runtime,
                    &mut value_stack,
                    addr,
                    mem,
                    offset,
                    1,
                    Some((8, 32)),
                    32,
                    "i32.load8_s",
                )?;
            }
            Operator::I32Load8U { memarg, .. } => {
                let mem = memarg.memory;
                let offset = memarg.offset;
                let addr = pop_value(&mut value_stack, "i32.load8_u address")?;
                translate_memory_load(
                    script,
                    runtime,
                    &mut value_stack,
                    addr,
                    mem,
                    offset,
                    1,
                    None,
                    32,
                    "i32.load8_u",
                )?;
            }
            Operator::I32Load16S { memarg, .. } => {
                let mem = memarg.memory;
                let offset = memarg.offset;
                let addr = pop_value(&mut value_stack, "i32.load16_s address")?;
                translate_memory_load(
                    script,
                    runtime,
                    &mut value_stack,
                    addr,
                    mem,
                    offset,
                    2,
                    Some((16, 32)),
                    32,
                    "i32.load16_s",
                )?;
            }
            Operator::I32Load16U { memarg, .. } => {
                let mem = memarg.memory;
                let offset = memarg.offset;
                let addr = pop_value(&mut value_stack, "i32.load16_u address")?;
                translate_memory_load(
                    script,
                    runtime,
                    &mut value_stack,
                    addr,
                    mem,
                    offset,
                    2,
                    None,
                    32,
                    "i32.load16_u",
                )?;
            }
            Operator::I64Load8S { memarg, .. } => {
                let mem = memarg.memory;
                let offset = memarg.offset;
                let addr = pop_value(&mut value_stack, "i64.load8_s address")?;
                translate_memory_load(
                    script,
                    runtime,
                    &mut value_stack,
                    addr,
                    mem,
                    offset,
                    1,
                    Some((8, 64)),
                    64,
                    "i64.load8_s",
                )?;
            }
            Operator::I64Load8U { memarg, .. } => {
                let mem = memarg.memory;
                let offset = memarg.offset;
                let addr = pop_value(&mut value_stack, "i64.load8_u address")?;
                translate_memory_load(
                    script,
                    runtime,
                    &mut value_stack,
                    addr,
                    mem,
                    offset,
                    1,
                    None,
                    64,
                    "i64.load8_u",
                )?;
            }
            Operator::I64Load16S { memarg, .. } => {
                let mem = memarg.memory;
                let offset = memarg.offset;
                let addr = pop_value(&mut value_stack, "i64.load16_s address")?;
                translate_memory_load(
                    script,
                    runtime,
                    &mut value_stack,
                    addr,
                    mem,
                    offset,
                    2,
                    Some((16, 64)),
                    64,
                    "i64.load16_s",
                )?;
            }
            Operator::I64Load16U { memarg, .. } => {
                let mem = memarg.memory;
                let offset = memarg.offset;
                let addr = pop_value(&mut value_stack, "i64.load16_u address")?;
                translate_memory_load(
                    script,
                    runtime,
                    &mut value_stack,
                    addr,
                    mem,
                    offset,
                    2,
                    None,
                    64,
                    "i64.load16_u",
                )?;
            }
            Operator::I64Load32S { memarg, .. } => {
                let mem = memarg.memory;
                let offset = memarg.offset;
                let addr = pop_value(&mut value_stack, "i64.load32_s address")?;
                translate_memory_load(
                    script,
                    runtime,
                    &mut value_stack,
                    addr,
                    mem,
                    offset,
                    4,
                    Some((32, 64)),
                    64,
                    "i64.load32_s",
                )?;
            }
            Operator::I64Load32U { memarg, .. } => {
                let mem = memarg.memory;
                let offset = memarg.offset;
                let addr = pop_value(&mut value_stack, "i64.load32_u address")?;
                translate_memory_load(
                    script,
                    runtime,
                    &mut value_stack,
                    addr,
                    mem,
                    offset,
                    4,
                    None,
                    64,
                    "i64.load32_u",
                )?;
            }
            Operator::I32Store { memarg, .. } => {
                let value = pop_value(&mut value_stack, "i32.store value")?;
                let addr = pop_value(&mut value_stack, "i32.store address")?;
                translate_memory_store(
                    script,
                    runtime,
                    value,
                    addr,
                    memarg.memory,
                    memarg.offset,
                    4,
                    "i32.store",
                )?;
            }
            Operator::I64Store { memarg, .. } => {
                let value = pop_value(&mut value_stack, "i64.store value")?;
                let addr = pop_value(&mut value_stack, "i64.store address")?;
                translate_memory_store(
                    script,
                    runtime,
                    value,
                    addr,
                    memarg.memory,
                    memarg.offset,
                    8,
                    "i64.store",
                )?;
            }
            Operator::I32Store8 { memarg, .. } => {
                let value = pop_value(&mut value_stack, "i32.store8 value")?;
                let addr = pop_value(&mut value_stack, "i32.store8 address")?;
                translate_memory_store(
                    script,
                    runtime,
                    value,
                    addr,
                    memarg.memory,
                    memarg.offset,
                    1,
                    "i32.store8",
                )?;
            }
            Operator::I32Store16 { memarg, .. } => {
                let value = pop_value(&mut value_stack, "i32.store16 value")?;
                let addr = pop_value(&mut value_stack, "i32.store16 address")?;
                translate_memory_store(
                    script,
                    runtime,
                    value,
                    addr,
                    memarg.memory,
                    memarg.offset,
                    2,
                    "i32.store16",
                )?;
            }
            Operator::I64Store8 { memarg, .. } => {
                let value = pop_value(&mut value_stack, "i64.store8 value")?;
                let addr = pop_value(&mut value_stack, "i64.store8 address")?;
                translate_memory_store(
                    script,
                    runtime,
                    value,
                    addr,
                    memarg.memory,
                    memarg.offset,
                    1,
                    "i64.store8",
                )?;
            }
            Operator::I64Store16 { memarg, .. } => {
                let value = pop_value(&mut value_stack, "i64.store16 value")?;
                let addr = pop_value(&mut value_stack, "i64.store16 address")?;
                translate_memory_store(
                    script,
                    runtime,
                    value,
                    addr,
                    memarg.memory,
                    memarg.offset,
                    2,
                    "i64.store16",
                )?;
            }
            Operator::I64Store32 { memarg, .. } => {
                let value = pop_value(&mut value_stack, "i64.store32 value")?;
                let addr = pop_value(&mut value_stack, "i64.store32 address")?;
                translate_memory_store(
                    script,
                    runtime,
                    value,
                    addr,
                    memarg.memory,
                    memarg.offset,
                    4,
                    "i64.store32",
                )?;
            }
            Operator::MemoryGrow { mem, .. } => {
                ensure_memory_access(runtime, mem)?;
                runtime.emit_memory_init_call(script)?;
                runtime.emit_memory_grow_call(script)?;
                value_stack.push(StackValue {
                    const_value: None,
                    bytecode_start: None,
                });
            }
            Operator::MemoryFill { mem, .. } => {
                let len = pop_value(&mut value_stack, "memory.fill len")?;
                let value = pop_value(&mut value_stack, "memory.fill value")?;
                let dest = pop_value(&mut value_stack, "memory.fill dest")?;
                translate_memory_fill(script, runtime, dest, value, len, mem)
                    .context("failed to translate memory.fill")?;
            }
            Operator::MemoryCopy {
                dst_mem, src_mem, ..
            } => {
                let len = pop_value(&mut value_stack, "memory.copy len")?;
                let src = pop_value(&mut value_stack, "memory.copy src")?;
                let dest = pop_value(&mut value_stack, "memory.copy dest")?;
                translate_memory_copy(script, runtime, dest, src, len, dst_mem, src_mem)
                    .context("failed to translate memory.copy")?;
            }
            Operator::MemoryInit {
                data_index, mem, ..
            } => {
                let len = pop_value(&mut value_stack, "memory.init len")?;
                let src = pop_value(&mut value_stack, "memory.init offset")?;
                let dest = pop_value(&mut value_stack, "memory.init dest")?;
                translate_memory_init(script, runtime, dest, src, len, data_index, mem)
                    .context("failed to translate memory.init")?;
            }
            Operator::DataDrop { data_index } => {
                translate_data_drop(script, runtime, data_index)
                    .context("failed to translate data.drop")?;
            }
            Operator::TableGet { table } => {
                let _ = pop_value(&mut value_stack, "table.get index")?;
                runtime.table_slot(table as usize)?;
                runtime.emit_memory_init_call(script)?;
                runtime.emit_table_helper(script, TableHelperKind::Get(table as usize))?;
                value_stack.push(StackValue {
                    const_value: None,
                    bytecode_start: None,
                });
            }
            Operator::TableSet { table } => {
                let _ = pop_value(&mut value_stack, "table.set value")?;
                let _ = pop_value(&mut value_stack, "table.set index")?;
                runtime.table_slot(table as usize)?;
                runtime.emit_memory_init_call(script)?;
                runtime.emit_table_helper(script, TableHelperKind::Set(table as usize))?;
            }
            Operator::TableSize { table } => {
                runtime.table_slot(table as usize)?;
                runtime.emit_memory_init_call(script)?;
                runtime.emit_table_helper(script, TableHelperKind::Size(table as usize))?;
                value_stack.push(StackValue {
                    const_value: None,
                    bytecode_start: None,
                });
            }
            Operator::TableGrow { table } => {
                let _delta = pop_value(&mut value_stack, "table.grow delta")?;
                let _value = pop_value(&mut value_stack, "table.grow value")?;
                runtime.table_slot(table as usize)?;
                runtime.emit_memory_init_call(script)?;
                runtime.emit_table_helper(script, TableHelperKind::Grow(table as usize))?;
                value_stack.push(StackValue {
                    const_value: None,
                    bytecode_start: None,
                });
            }
            Operator::TableFill { table } => {
                let _len = pop_value(&mut value_stack, "table.fill len")?;
                let _value = pop_value(&mut value_stack, "table.fill value")?;
                let _dst = pop_value(&mut value_stack, "table.fill dest")?;
                runtime.table_slot(table as usize)?;
                runtime.emit_memory_init_call(script)?;
                runtime.emit_table_helper(script, TableHelperKind::Fill(table as usize))?;
            }
            Operator::TableCopy {
                dst_table,
                src_table,
            } => {
                let _len = pop_value(&mut value_stack, "table.copy len")?;
                let _src = pop_value(&mut value_stack, "table.copy src")?;
                let _dst = pop_value(&mut value_stack, "table.copy dest")?;
                runtime.table_slot(dst_table as usize)?;
                runtime.table_slot(src_table as usize)?;
                runtime.emit_memory_init_call(script)?;
                runtime.emit_table_helper(
                    script,
                    TableHelperKind::Copy {
                        dst: dst_table as usize,
                        src: src_table as usize,
                    },
                )?;
            }
            Operator::TableInit { table, elem_index } => {
                let _len = pop_value(&mut value_stack, "table.init len")?;
                let _src = pop_value(&mut value_stack, "table.init offset")?;
                let _dst = pop_value(&mut value_stack, "table.init dest")?;
                runtime.ensure_passive_element(elem_index as usize)?;
                runtime.table_slot(table as usize)?;
                runtime.emit_memory_init_call(script)?;
                runtime.emit_table_helper(
                    script,
                    TableHelperKind::InitFromPassive {
                        table: table as usize,
                        segment: elem_index as usize,
                    },
                )?;
            }
            Operator::ElemDrop { elem_index } => {
                runtime.ensure_passive_element(elem_index as usize)?;
                runtime.emit_memory_init_call(script)?;
                runtime
                    .emit_table_helper(script, TableHelperKind::ElemDrop(elem_index as usize))?;
            }
            Operator::I32Eq => {
                let rhs = pop_value(&mut value_stack, "i32.eq rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.eq lhs")?;
                let result = emit_binary_op(script, "EQUAL", lhs, rhs, |a, b| {
                    Some(if a == b { 1 } else { 0 })
                })?;
                value_stack.push(result);
            }
            Operator::I32Ne => {
                let rhs = pop_value(&mut value_stack, "i32.ne rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.ne lhs")?;
                let result = emit_binary_op(script, "NOTEQUAL", lhs, rhs, |a, b| {
                    Some(if a != b { 1 } else { 0 })
                })?;
                value_stack.push(result);
            }
            Operator::I32LtS => {
                let rhs = pop_value(&mut value_stack, "i32.lt_s rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.lt_s lhs")?;
                let result = emit_signed_compare(script, lhs, rhs, 32, CompareOp::Lt)?;
                value_stack.push(result);
            }
            Operator::I32LtU => {
                let rhs = pop_value(&mut value_stack, "i32.lt_u rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.lt_u lhs")?;
                let result = emit_unsigned_compare(script, lhs, rhs, 32, CompareOp::Lt)?;
                value_stack.push(result);
            }
            Operator::I32LeS => {
                let rhs = pop_value(&mut value_stack, "i32.le_s rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.le_s lhs")?;
                let result = emit_signed_compare(script, lhs, rhs, 32, CompareOp::Le)?;
                value_stack.push(result);
            }
            Operator::I32LeU => {
                let rhs = pop_value(&mut value_stack, "i32.le_u rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.le_u lhs")?;
                let result = emit_unsigned_compare(script, lhs, rhs, 32, CompareOp::Le)?;
                value_stack.push(result);
            }
            Operator::I32GtS => {
                let rhs = pop_value(&mut value_stack, "i32.gt_s rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.gt_s lhs")?;
                let result = emit_signed_compare(script, lhs, rhs, 32, CompareOp::Gt)?;
                value_stack.push(result);
            }
            Operator::I32GtU => {
                let rhs = pop_value(&mut value_stack, "i32.gt_u rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.gt_u lhs")?;
                let result = emit_unsigned_compare(script, lhs, rhs, 32, CompareOp::Gt)?;
                value_stack.push(result);
            }
            Operator::I32GeS => {
                let rhs = pop_value(&mut value_stack, "i32.ge_s rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.ge_s lhs")?;
                let result = emit_signed_compare(script, lhs, rhs, 32, CompareOp::Ge)?;
                value_stack.push(result);
            }
            Operator::I32GeU => {
                let rhs = pop_value(&mut value_stack, "i32.ge_u rhs")?;
                let lhs = pop_value(&mut value_stack, "i32.ge_u lhs")?;
                let result = emit_unsigned_compare(script, lhs, rhs, 32, CompareOp::Ge)?;
                value_stack.push(result);
            }
            Operator::I64Eq => {
                let rhs = pop_value(&mut value_stack, "i64.eq rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.eq lhs")?;
                let result = emit_binary_op(script, "EQUAL", lhs, rhs, |a, b| {
                    Some(if a == b { 1 } else { 0 })
                })?;
                value_stack.push(result);
            }
            Operator::I64Ne => {
                let rhs = pop_value(&mut value_stack, "i64.ne rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.ne lhs")?;
                let result = emit_binary_op(script, "NOTEQUAL", lhs, rhs, |a, b| {
                    Some(if a != b { 1 } else { 0 })
                })?;
                value_stack.push(result);
            }
            Operator::I64LtS => {
                let rhs = pop_value(&mut value_stack, "i64.lt_s rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.lt_s lhs")?;
                let result = emit_signed_compare(script, lhs, rhs, 64, CompareOp::Lt)?;
                value_stack.push(result);
            }
            Operator::I64LtU => {
                let rhs = pop_value(&mut value_stack, "i64.lt_u rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.lt_u lhs")?;
                let result = emit_unsigned_compare(script, lhs, rhs, 64, CompareOp::Lt)?;
                value_stack.push(result);
            }
            Operator::I64LeS => {
                let rhs = pop_value(&mut value_stack, "i64.le_s rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.le_s lhs")?;
                let result = emit_signed_compare(script, lhs, rhs, 64, CompareOp::Le)?;
                value_stack.push(result);
            }
            Operator::I64LeU => {
                let rhs = pop_value(&mut value_stack, "i64.le_u rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.le_u lhs")?;
                let result = emit_unsigned_compare(script, lhs, rhs, 64, CompareOp::Le)?;
                value_stack.push(result);
            }
            Operator::I64GtS => {
                let rhs = pop_value(&mut value_stack, "i64.gt_s rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.gt_s lhs")?;
                let result = emit_signed_compare(script, lhs, rhs, 64, CompareOp::Gt)?;
                value_stack.push(result);
            }
            Operator::I64GtU => {
                let rhs = pop_value(&mut value_stack, "i64.gt_u rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.gt_u lhs")?;
                let result = emit_unsigned_compare(script, lhs, rhs, 64, CompareOp::Gt)?;
                value_stack.push(result);
            }
            Operator::I64GeS => {
                let rhs = pop_value(&mut value_stack, "i64.ge_s rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.ge_s lhs")?;
                let result = emit_signed_compare(script, lhs, rhs, 64, CompareOp::Ge)?;
                value_stack.push(result);
            }
            Operator::I64GeU => {
                let rhs = pop_value(&mut value_stack, "i64.ge_u rhs")?;
                let lhs = pop_value(&mut value_stack, "i64.ge_u lhs")?;
                let result = emit_unsigned_compare(script, lhs, rhs, 64, CompareOp::Ge)?;
                value_stack.push(result);
            }
            Operator::Select => {
                let condition = pop_value(&mut value_stack, "select condition")?;
                let false_value = pop_value(&mut value_stack, "select false value")?;
                let true_value = pop_value(&mut value_stack, "select true value")?;
                let result = emit_select(script, true_value, false_value, condition)?;
                value_stack.push(result);
            }
            Operator::TypedSelect { ty } => {
                ensure_select_type_supported(&[ty])?;
                let condition = pop_value(&mut value_stack, "typed select condition")?;
                let false_value = pop_value(&mut value_stack, "typed select false value")?;
                let true_value = pop_value(&mut value_stack, "typed select true value")?;
                let result = emit_select(script, true_value, false_value, condition)?;
                value_stack.push(result);
            }
            Operator::TypedSelectMulti { tys } => {
                ensure_select_type_supported(&tys)?;
                let condition = pop_value(&mut value_stack, "typed select condition")?;
                let false_value = pop_value(&mut value_stack, "typed select false value")?;
                let true_value = pop_value(&mut value_stack, "typed select true value")?;
                let result = emit_select(script, true_value, false_value, condition)?;
                value_stack.push(result);
            }
            Operator::I32Eqz => {
                let value = pop_value(&mut value_stack, "i32.eqz operand")?;
                let result = emit_eqz(script, value)?;
                value_stack.push(result);
            }
            Operator::I64Eqz => {
                let value = pop_value(&mut value_stack, "i64.eqz operand")?;
                let result = emit_eqz(script, value)?;
                value_stack.push(result);
            }
            Operator::LocalGet { local_index } => {
                let state = local_states
                    .get(local_index as usize)
                    .ok_or_else(|| anyhow!("local index {} out of bounds", local_index))?;
                let value = emit_local_get(script, state)?;
                value_stack.push(value);
            }
            Operator::LocalSet { local_index } => {
                let value = pop_value(&mut value_stack, "local.set operand")?;
                let state = local_states
                    .get_mut(local_index as usize)
                    .ok_or_else(|| anyhow!("local index {} out of bounds", local_index))?;
                emit_local_set(script, state, &value)?;
            }
            Operator::LocalTee { local_index } => {
                let value = pop_value(&mut value_stack, "local.tee operand")?;
                let state = local_states
                    .get_mut(local_index as usize)
                    .ok_or_else(|| anyhow!("local index {} out of bounds", local_index))?;
                emit_local_set(script, state, &value)?;
                let value = emit_local_get(script, state)?;
                value_stack.push(value);
            }
            Operator::GlobalGet { global_index } => {
                let idx = global_index as usize;
                let const_value = runtime.global_const_value(idx)?;
                if let Some(value) = const_value {
                    let entry = emit_push_int(script, value);
                    value_stack.push(entry);
                } else {
                    runtime.emit_memory_init_call(script)?;
                    let slot = runtime.global_slot(idx)?;
                    emit_load_static(script, slot)?;
                    value_stack.push(StackValue {
                        const_value: None,
                        bytecode_start: None,
                    });
                }
            }
            Operator::GlobalSet { global_index } => {
                let idx = global_index as usize;
                if !runtime.global_mutable(idx)? {
                    bail!("global {} is immutable", idx);
                }
                let _value = pop_value(&mut value_stack, "global.set operand")?;
                runtime.emit_memory_init_call(script)?;
                let slot = runtime.global_slot(idx)?;
                emit_store_static(script, slot)?;
                runtime.clear_global_const(idx)?;
            }
            Operator::Return => {
                script.push(RET);
                emitted_ret = true;
                value_stack.clear();
            }
            Operator::Call { function_index } => {
                let param_count = if let Some(import) = imports.get(function_index as usize) {
                    let type_index = get_import_type_index(import)?;
                    types
                        .get(type_index as usize)
                        .ok_or_else(|| {
                            anyhow!(
                                "invalid type index {} for import {}",
                                type_index,
                                import.name
                            )
                        })?
                        .params()
                        .len()
                } else {
                    let defined_index = (function_index as usize)
                        .checked_sub(imports.len())
                        .ok_or_else(|| anyhow!("function index underflow"))?;
                    let type_index =
                        func_type_indices
                            .get(defined_index)
                            .copied()
                            .ok_or_else(|| {
                                anyhow!("no type index recorded for function {}", function_index)
                            })?;
                    types
                        .get(type_index as usize)
                        .ok_or_else(|| {
                            anyhow!(
                                "invalid type index {} for function {}",
                                type_index,
                                function_index
                            )
                        })?
                        .params()
                        .len()
                };

                let mut params = Vec::with_capacity(param_count);
                for _ in 0..param_count {
                    params.push(pop_value(&mut value_stack, "call argument")?);
                }
                params.reverse();

                if let Some(import) = imports.get(function_index as usize) {
                    let type_index = get_import_type_index(import)?;
                    let func_sig = types.get(type_index as usize).ok_or_else(|| {
                        anyhow!(
                            "invalid type index {} for import {}",
                            type_index,
                            import.name
                        )
                    })?;
                    if try_handle_env_import(
                        import,
                        func_sig,
                        &params,
                        runtime,
                        script,
                        &mut value_stack,
                    )? {
                        continue;
                    }

                    handle_import_call(function_index, script, imports, types, &params)?;
                    if !func_sig.results().is_empty() {
                        value_stack.push(StackValue {
                            const_value: None,
                            bytecode_start: None,
                        });
                    }
                } else {
                    let defined_index = (function_index as usize) - imports.len();
                    let type_index =
                        func_type_indices
                            .get(defined_index)
                            .copied()
                            .ok_or_else(|| {
                                anyhow!("no type index recorded for function {}", function_index)
                            })?;
                    let func_sig = types.get(type_index as usize).ok_or_else(|| {
                        anyhow!(
                            "invalid type index {} for function {}",
                            type_index,
                            function_index
                        )
                    })?;
                    if func_sig.params().len() != params.len() {
                        bail!(
                            "function {} expects {} argument(s) but {} were provided",
                            function_index,
                            func_sig.params().len(),
                            params.len()
                        );
                    }
                    if func_sig.results().len() > 1 {
                        bail!(
                            "multi-value returns are not supported (function {} returns {} values)",
                            function_index,
                            func_sig.results().len()
                        );
                    }
                    functions.emit_call(script, function_index as usize)?;
                    if !func_sig.results().is_empty() {
                        value_stack.push(StackValue {
                            const_value: None,
                            bytecode_start: None,
                        });
                    }
                }
            }
            Operator::CallIndirect {
                table_index,
                type_index,
            } => {
                tables.get(table_index as usize).ok_or_else(|| {
                    anyhow!("call_indirect references missing table {}", table_index)
                })?;

                let func_sig = types.get(type_index as usize).ok_or_else(|| {
                    anyhow!("type index {} out of bounds for call_indirect", type_index)
                })?;

                for ty in func_sig.params() {
                    match ty {
                        ValType::I32 | ValType::I64 => {}
                        other => bail!(
                            "call_indirect with unsupported parameter type {:?}; only i32/i64 are supported",
                            other
                        ),
                    }
                }
                if func_sig.results().len() > 1 {
                    bail!("call_indirect returning multiple values is not supported");
                }

                let _table_index_value = pop_value(&mut value_stack, "call_indirect table index")?;

                let mut params = Vec::with_capacity(func_sig.params().len());
                for _ in 0..func_sig.params().len() {
                    params.push(pop_value(&mut value_stack, "call_indirect argument")?);
                }
                params.reverse();

                runtime.emit_memory_init_call(script)?;
                runtime.table_slot(table_index as usize)?;
                runtime.emit_table_helper(script, TableHelperKind::Get(table_index as usize))?;

                script.push(lookup_opcode("DUP")?.byte);
                let _ = emit_push_int(script, FUNCREF_NULL);
                script.push(lookup_opcode("EQUAL")?.byte);
                let trap_null = emit_jump_placeholder(script, "JMPIF_L")?;
                script.push(lookup_opcode("DROP")?.byte);

                let total_functions = imports.len() + func_type_indices.len();
                let mut case_fixups: Vec<(usize, CallTarget)> = Vec::new();
                for fn_index in 0..total_functions {
                    let candidate_type_index = if fn_index < imports.len() {
                        get_import_type_index(&imports[fn_index])?
                    } else {
                        let defined_index = fn_index - imports.len();
                        *func_type_indices.get(defined_index).ok_or_else(|| {
                            anyhow!(
                                "call_indirect target function {} missing type entry",
                                fn_index
                            )
                        })?
                    };

                    if candidate_type_index != type_index {
                        continue;
                    }

                    script.push(lookup_opcode("DUP")?.byte);
                    let _ = emit_push_int(script, fn_index as i128);
                    script.push(lookup_opcode("EQUAL")?.byte);
                    let jump = emit_jump_placeholder(script, "JMPIF_L")?;

                    let target = if fn_index < imports.len() {
                        CallTarget::Import(fn_index as u32)
                    } else {
                        CallTarget::Defined(fn_index)
                    };
                    case_fixups.push((jump, target));
                }

                let trap_label = script.len();
                script.push(lookup_opcode("DROP")?.byte);
                script.push(lookup_opcode("ABORT")?.byte);
                patch_jump(script, trap_null, trap_label)?;

                let mut end_fixups: Vec<usize> = Vec::new();
                for (jump, target) in case_fixups {
                    let label = script.len();
                    patch_jump(script, jump, label)?;
                    script.push(lookup_opcode("DROP")?.byte);
                    match target {
                        CallTarget::Import(idx) => {
                            handle_import_call(idx, script, imports, types, &params)?;
                        }
                        CallTarget::Defined(idx) => {
                            functions.emit_call(script, idx)?;
                        }
                    }
                    let end_jump = emit_jump_placeholder(script, "JMP_L")?;
                    end_fixups.push(end_jump);
                }

                let end_label = script.len();
                for fixup in end_fixups {
                    patch_jump(script, fixup, end_label)?;
                }

                if !func_sig.results().is_empty() {
                    value_stack.push(StackValue {
                        const_value: None,
                        bytecode_start: None,
                    });
                }
            }
            Operator::Drop => {
                let value = pop_value(&mut value_stack, "drop operand")?;
                if let Some(start) = value.bytecode_start {
                    script.truncate(start);
                } else {
                    let drop = lookup_opcode("DROP")?;
                    script.push(drop.byte);
                }
            }
            Operator::RefEq => {
                let rhs = pop_value(&mut value_stack, "ref.eq rhs")?;
                let lhs = pop_value(&mut value_stack, "ref.eq lhs")?;
                let result = emit_binary_op(script, "EQUAL", lhs, rhs, |a, b| {
                    Some(if a == b { 1 } else { 0 })
                })?;
                value_stack.push(result);
            }
            Operator::RefAsNonNull => {
                let value = pop_value(&mut value_stack, "ref.as_non_null operand")?;
                if let Some(constant) = value.const_value {
                    if constant == FUNCREF_NULL {
                        let abort = lookup_opcode("ABORT")?;
                        script.push(abort.byte);
                        value_stack.clear();
                    } else {
                        value_stack.push(value);
                    }
                } else {
                    let dup = lookup_opcode("DUP")?;
                    script.push(dup.byte);
                    let _ = emit_push_int(script, FUNCREF_NULL);
                    script.push(lookup_opcode("EQUAL")?.byte);
                    let skip_trap = emit_jump_placeholder(script, "JMPIFNOT_L")?;
                    script.push(lookup_opcode("DROP")?.byte);
                    script.push(lookup_opcode("ABORT")?.byte);
                    let continue_label = script.len();
                    patch_jump(script, skip_trap, continue_label)?;
                    value_stack.push(value);
                }
            }
            Operator::RefNull { hty } => match hty {
                HeapType::FUNC => {
                    let entry = emit_push_int(script, FUNCREF_NULL);
                    value_stack.push(entry);
                }
                other => bail!(
                    "ref.null with heap type {:?} is unsupported (NeoVM only models funcref handles; see docs/wasm-pipeline.md#9-unsupported-wasm-features)",
                    other
                ),
            },
            Operator::RefIsNull => {
                let value = pop_value(&mut value_stack, "ref.is_null operand")?;
                let sentinel = emit_push_int(script, FUNCREF_NULL);
                let result = emit_binary_op(script, "EQUAL", value, sentinel, |a, b| {
                    Some(if a == b { 1 } else { 0 })
                })?;
                value_stack.push(result);
            }
            Operator::RefFunc { function_index } => {
                let total_functions = imports.len() + func_type_indices.len();
                if function_index as usize >= total_functions {
                    bail!(
                        "ref.func references unknown function index {} (total functions: {})",
                        function_index,
                        total_functions
                    );
                }
                let entry = emit_push_int(script, function_index as i128);
                value_stack.push(entry);
            }
            Operator::Unreachable => {
                let abort = lookup_opcode("ABORT")?;
                script.push(abort.byte);
                value_stack.clear();
            }
            _ => {
                if let Some(desc) = describe_float_op(&op) {
                    let context = format!("{} in function {}", desc, function_name);
                    return numeric::unsupported_float(&context);
                }
                if let Some(desc) = describe_simd_op(&op) {
                    let context = format!("{} in function {}", desc, function_name);
                    return numeric::unsupported_simd(&context);
                }
                bail!(format!("unsupported Wasm operator {:?} ({}).", op, UNSUPPORTED_FEATURE_DOC));
            }
        }
    }

    if !emitted_ret {
        script.push(RET);
    }

    if let Some(frame) = control_stack.last() {
        bail!(
            "unclosed block detected at end of function (kind: {:?})",
            frame.kind
        );
    }

    Ok(return_kind.unwrap_or_else(|| "Void".to_string()))
}

fn pop_value(stack: &mut Vec<StackValue>, context: &str) -> Result<StackValue> {
    stack
        .pop()
        .ok_or_else(|| anyhow!("stack underflow while processing {}", context))
}

pub(crate) fn emit_binary_op(
    script: &mut Vec<u8>,
    opcode_name: &str,
    lhs: StackValue,
    rhs: StackValue,
    combine: impl FnOnce(i128, i128) -> Option<i128>,
) -> Result<StackValue> {
    let opcode = lookup_opcode(opcode_name)?;
    script.push(opcode.byte);
    let const_value = match (lhs.const_value, rhs.const_value) {
        (Some(a), Some(b)) => combine(a, b),
        _ => None,
    };
    Ok(StackValue {
        const_value,
        bytecode_start: None,
    })
}

fn emit_eqz(script: &mut Vec<u8>, value: StackValue) -> Result<StackValue> {
    if let (Some(constant), Some(start)) = (value.const_value, value.bytecode_start) {
        script.truncate(start);
        let result = if constant == 0 { 1 } else { 0 };
        return Ok(emit_push_int(script, result));
    }

    let push0 = lookup_opcode("PUSH0")?;
    script.push(push0.byte);
    let equal = lookup_opcode("EQUAL")?;
    script.push(equal.byte);
    Ok(StackValue {
        const_value: None,
        bytecode_start: None,
    })
}

#[derive(Clone, Copy)]
enum ShiftKind {
    Arithmetic,
    Logical,
}

fn emit_shift_right(
    script: &mut Vec<u8>,
    lhs: StackValue,
    rhs: StackValue,
    bits: u32,
    kind: ShiftKind,
) -> Result<StackValue> {
    mask_shift_amount(script, bits)?;
    match kind {
        ShiftKind::Arithmetic => {
            script.push(lookup_opcode("SHR")?.byte);
        }
        ShiftKind::Logical => {
            let swap = lookup_opcode("SWAP")?;
            script.push(swap.byte);
            let mask = ((1u128 << bits) - 1) as i128;
            let _ = emit_push_int(script, mask);
            script.push(lookup_opcode("AND")?.byte);
            script.push(swap.byte);
            script.push(lookup_opcode("SHR")?.byte);
        }
    }

    let const_value = match (lhs.const_value, rhs.const_value) {
        (Some(a), Some(b)) => {
            let shift = (b as u32) & (bits - 1);
            match kind {
                ShiftKind::Arithmetic => {
                    if bits == 32 {
                        Some(((a as i32) >> shift) as i128)
                    } else {
                        Some(((a as i64) >> shift) as i128)
                    }
                }
                ShiftKind::Logical => {
                    let mask = (1u128 << bits) - 1;
                    let unsigned = (a as u128) & mask;
                    Some((unsigned >> shift) as i128)
                }
            }
        }
        _ => None,
    };

    Ok(StackValue {
        const_value,
        bytecode_start: None,
    })
}

fn emit_rotate(
    script: &mut Vec<u8>,
    value: StackValue,
    shift: StackValue,
    bits: u32,
    left: bool,
) -> Result<StackValue> {
    match (
        value.const_value,
        shift.const_value,
        value.bytecode_start,
        shift.bytecode_start,
    ) {
        (Some(v), Some(s), Some(value_start), Some(shift_start)) => {
            let mask = match bits {
                32 => 31,
                64 => 63,
                _ => unreachable!(),
            };
            let rotate = match bits {
                32 => {
                    let val = v as i32;
                    let amt = (s as u32) & mask;
                    if left {
                        val.rotate_left(amt) as i128
                    } else {
                        val.rotate_right(amt) as i128
                    }
                }
                64 => {
                    let val = v as i64;
                    let amt = (s as u32) & mask;
                    if left {
                        val.rotate_left(amt) as i128
                    } else {
                        val.rotate_right(amt) as i128
                    }
                }
                _ => unreachable!(),
            };
            script.truncate(value_start.min(shift_start));
            Ok(emit_push_int(script, rotate))
        }
        _ => emit_rotate_dynamic(script, value, shift, bits, left),
    }
}

fn emit_rotate_dynamic(
    script: &mut Vec<u8>,
    value: StackValue,
    shift: StackValue,
    bits: u32,
    left: bool,
) -> Result<StackValue> {
    let mut stack = vec![value, shift];

    let mask_sv = emit_push_int(script, (bits - 1) as i128);
    stack.push(mask_sv);
    apply_binary(script, &mut stack, "AND", |a, b| Some(a & b))?;

    stack_pick(script, &mut stack, 1)?;
    stack_pick(script, &mut stack, 1)?;

    if left {
        apply_binary(script, &mut stack, "SHL", |a, b| {
            let shift = (b as u32) & (bits - 1);
            if bits == 32 {
                Some(((a as i32) << shift) as i128)
            } else {
                Some(((a as i64) << shift) as i128)
            }
        })?
    } else {
        apply_shift_right(script, &mut stack, bits, ShiftKind::Logical)?
    };

    stack_pick(script, &mut stack, 2)?;
    stack_pick(script, &mut stack, 2)?;

    let bits_sv = emit_push_int(script, bits as i128);
    stack.push(bits_sv);
    stack_swap(script, &mut stack)?;
    apply_binary(script, &mut stack, "SUB", |a, b| Some(a - b))?;

    if left {
        apply_shift_right(script, &mut stack, bits, ShiftKind::Logical)?
    } else {
        apply_binary(script, &mut stack, "SHL", |a, b| {
            let shift = (b as u32) & (bits - 1);
            if bits == 32 {
                Some(((a as i32) << shift) as i128)
            } else {
                Some(((a as i64) << shift) as i128)
            }
        })?
    };

    let result = apply_binary(script, &mut stack, "OR", |a, b| Some(a | b))?;

    stack_swap(script, &mut stack)?;
    stack_drop(script, &mut stack)?;
    stack_swap(script, &mut stack)?;
    stack_drop(script, &mut stack)?;

    Ok(result)
}

fn apply_binary(
    script: &mut Vec<u8>,
    stack: &mut Vec<StackValue>,
    opcode: &str,
    combine: impl FnOnce(i128, i128) -> Option<i128>,
) -> Result<StackValue> {
    let rhs = stack
        .pop()
        .ok_or_else(|| anyhow!("stack underflow for {} rhs", opcode))?;
    let lhs = stack
        .pop()
        .ok_or_else(|| anyhow!("stack underflow for {} lhs", opcode))?;
    let result = emit_binary_op(script, opcode, lhs, rhs, combine)?;
    let clone = result.clone();
    stack.push(result);
    Ok(clone)
}

fn apply_shift_right(
    script: &mut Vec<u8>,
    stack: &mut Vec<StackValue>,
    bits: u32,
    kind: ShiftKind,
) -> Result<StackValue> {
    let rhs = stack
        .pop()
        .ok_or_else(|| anyhow!("stack underflow for shift rhs"))?;
    let lhs = stack
        .pop()
        .ok_or_else(|| anyhow!("stack underflow for shift lhs"))?;
    let result = emit_shift_right(script, lhs, rhs, bits, kind)?;
    let clone = result.clone();
    stack.push(result);
    Ok(clone)
}

fn stack_pick(script: &mut Vec<u8>, stack: &mut Vec<StackValue>, index: usize) -> Result<()> {
    let idx_sv = emit_push_int(script, index as i128);
    stack.push(idx_sv);
    script.push(lookup_opcode("PICK")?.byte);
    let len = stack.len();
    if index >= len - 1 {
        bail!("PICK index {} out of range", index);
    }
    let picked = stack[len - 2 - index].clone();
    stack.pop();
    stack.push(StackValue {
        const_value: picked.const_value,
        bytecode_start: None,
    });
    Ok(())
}

fn stack_swap(script: &mut Vec<u8>, stack: &mut Vec<StackValue>) -> Result<()> {
    if stack.len() < 2 {
        bail!("SWAP requires at least two stack values");
    }
    script.push(lookup_opcode("SWAP")?.byte);
    let len = stack.len();
    stack.swap(len - 1, len - 2);
    Ok(())
}

fn stack_drop(script: &mut Vec<u8>, stack: &mut Vec<StackValue>) -> Result<()> {
    if stack.is_empty() {
        bail!("DROP requires at least one stack value");
    }
    script.push(lookup_opcode("DROP")?.byte);
    stack.pop();
    Ok(())
}

fn mask_shift_amount(script: &mut Vec<u8>, bits: u32) -> Result<()> {
    if bits == 0 {
        return Ok(());
    }
    let mask = (bits - 1) as i128;
    let _ = emit_push_int(script, mask);
    script.push(lookup_opcode("AND")?.byte);
    Ok(())
}

fn describe_float_op(op: &Operator) -> Option<String> {
    let name = format!("{:?}", op);
    if name.starts_with("F32") || name.starts_with("F64") {
        return Some(name.to_lowercase());
    }
    None
}

fn describe_simd_op(op: &Operator) -> Option<String> {
    const PREFIXES: &[&str] = &["I8x16", "I16x8", "I32x4", "I64x2", "F32x4", "F64x2", "V128"];
    let name = format!("{:?}", op);
    if PREFIXES.iter().any(|prefix| name.starts_with(prefix)) {
        return Some(name.to_lowercase());
    }
    None
}

fn emit_local_get(script: &mut Vec<u8>, state: &LocalState) -> Result<StackValue> {
    if let Some(value) = state.const_value {
        return Ok(emit_push_int(script, value));
    }

    match state.kind {
        LocalKind::Param(index) => emit_load_arg(script, index)?,
        LocalKind::Local(slot) => emit_load_local_slot(script, slot)?,
    }

    Ok(StackValue {
        const_value: None,
        bytecode_start: None,
    })
}

fn emit_local_set(script: &mut Vec<u8>, state: &mut LocalState, value: &StackValue) -> Result<()> {
    match state.kind {
        LocalKind::Param(index) => emit_store_arg(script, index)?,
        LocalKind::Local(slot) => emit_store_local_slot(script, slot)?,
    }
    state.const_value = value.const_value;
    Ok(())
}

/// Extract type index from Import's TypeRef
fn get_import_type_index(import: &FunctionImport) -> Result<u32> {
    Ok(import.type_index)
}

/// Helper to emit indexed opcodes (LDARG, STARG, LDLOC, STLOC)
/// NeoVM has optimized opcodes for indices 0-6 (e.g., LDARG0-LDARG6)
fn emit_indexed_opcode(
    script: &mut Vec<u8>,
    base_opcode: &str,
    _indexed_base: &str,
    index: u32,
) -> Result<()> {
    // Try optimized indexed opcode first (0-6)
    if index <= 6 {
        let indexed_name = format!("{}{}", base_opcode, index);
        if let Some(opcode) = opcodes::lookup(&indexed_name) {
            script.push(opcode.byte);
            return Ok(());
        }
    }

    // Fall back to base opcode with explicit index
    let opcode =
        opcodes::lookup(base_opcode).ok_or_else(|| anyhow!("unknown opcode: {}", base_opcode))?;

    if index > u8::MAX as u32 {
        bail!(
            "{} index {} exceeds NeoVM operand limit (0-255)",
            base_opcode,
            index
        );
    }

    script.push(opcode.byte);
    script.push(index as u8);
    Ok(())
}

fn emit_load_arg(script: &mut Vec<u8>, index: u32) -> Result<()> {
    emit_indexed_opcode(script, "LDARG", "LDARG", index)
}

fn emit_store_arg(script: &mut Vec<u8>, index: u32) -> Result<()> {
    emit_indexed_opcode(script, "STARG", "STARG", index)
}

fn emit_load_local_slot(script: &mut Vec<u8>, slot: u32) -> Result<()> {
    emit_indexed_opcode(script, "LDLOC", "LDLOC", slot)
}

fn emit_store_local_slot(script: &mut Vec<u8>, slot: u32) -> Result<()> {
    emit_indexed_opcode(script, "STLOC", "STLOC", slot)
}

fn wasm_val_type_to_manifest(ty: &ValType) -> Result<String> {
    let repr = match ty {
        ValType::I32 => "Integer",
        ValType::I64 => "Integer",
        ValType::F32 | ValType::F64 => return numeric::unsupported_float("manifest numeric type"),
        ValType::V128 => return numeric::unsupported_simd("manifest v128"),
        ValType::Ref(_) => return numeric::unsupported_reference_type("manifest reference type"),
    };
    Ok(repr.to_string())
}

fn handle_branch(
    script: &mut Vec<u8>,
    value_stack: &mut Vec<StackValue>,
    control_stack: &mut Vec<ControlFrame>,
    relative_depth: usize,
    conditional: bool,
    is_unreachable: &mut bool,
) -> Result<()> {
    if relative_depth >= control_stack.len() {
        bail!(
            "branch depth {} exceeds current control stack",
            relative_depth
        );
    }
    let target_index = control_stack.len() - 1 - relative_depth;
    let (prefix, _) = control_stack.split_at_mut(target_index + 1);
    let frame = &mut prefix[target_index];

    // Only validate stack height if we're not already in unreachable code
    // For Function frames: branching means providing return values, validate against result_count
    // For other frames: branching means jumping to end of block, validate against stack_height
    if !*is_unreachable {
        let expected = if frame.kind == ControlKind::Function {
            frame.result_count
        } else {
            frame.stack_height
        };
        if value_stack.len() != expected {
            bail!(
                "branch requires {} values but current stack has {}",
                expected,
                value_stack.len()
            );
        }
    }

    match frame.kind {
        ControlKind::Loop => {
            let opcode = if conditional { "JMPIF_L" } else { "JMP_L" };
            emit_jump_to(script, opcode, frame.start_offset)?;
        }
        _ => {
            let opcode = if conditional { "JMPIF_L" } else { "JMP_L" };
            let pos = emit_jump_placeholder(script, opcode)?;
            frame.end_fixups.push(pos);
        }
    }

    if !conditional {
        // For Function frames, keep result_count values on stack for return
        // For other frames, truncate to stack_height
        let target_size = if frame.kind == ControlKind::Function {
            frame.result_count
        } else {
            frame.stack_height
        };
        value_stack.truncate(target_size);
        // Unconditional branch makes subsequent code unreachable
        *is_unreachable = true;
    }

    Ok(())
}

fn handle_br_table(
    script: &mut Vec<u8>,
    value_stack: &mut Vec<StackValue>,
    control_stack: &mut Vec<ControlFrame>,
    index: StackValue,
    targets: &[usize],
    default_depth: usize,
    is_unreachable: &mut bool,
) -> Result<()> {
    if let Some(const_idx) = index.const_value {
        if let Some(start) = index.bytecode_start {
            script.truncate(start);
        } else {
            script.push(lookup_opcode("DROP")?.byte);
        }
        let idx = if const_idx < 0 || const_idx > usize::MAX as i128 {
            usize::MAX
        } else {
            const_idx as usize
        };
        let depth = targets.get(idx).copied().unwrap_or(default_depth);
        handle_branch(
            script,
            value_stack,
            control_stack,
            depth,
            false,
            is_unreachable,
        )?;
        return Ok(());
    }

    emit_br_table_dynamic(
        script,
        value_stack,
        control_stack,
        targets,
        default_depth,
        is_unreachable,
    )
}

fn emit_br_table_dynamic(
    script: &mut Vec<u8>,
    value_stack: &mut Vec<StackValue>,
    control_stack: &mut Vec<ControlFrame>,
    targets: &[usize],
    default_depth: usize,
    is_unreachable: &mut bool,
) -> Result<()> {
    let dup = lookup_opcode("DUP")?.byte;
    let equal = lookup_opcode("EQUAL")?.byte;
    let drop = lookup_opcode("DROP")?.byte;

    let mut case_fixups: Vec<(usize, usize)> = Vec::with_capacity(targets.len());

    for (case_index, &depth) in targets.iter().enumerate() {
        script.push(dup);
        let _ = emit_push_int(script, case_index as i128);
        script.push(equal);
        let fixup = emit_jump_placeholder(script, "JMPIF_L")?;
        case_fixups.push((fixup, depth));
    }

    script.push(drop);
    handle_branch(
        script,
        value_stack,
        control_stack,
        default_depth,
        false,
        is_unreachable,
    )?;

    for (fixup, depth) in case_fixups {
        let label_pos = script.len();
        patch_jump(script, fixup, label_pos)?;
        script.push(drop);
        handle_branch(
            script,
            value_stack,
            control_stack,
            depth,
            false,
            is_unreachable,
        )?;
    }

    Ok(())
}

pub(crate) fn handle_import_call(
    function_index: u32,
    script: &mut Vec<u8>,
    imports: &[FunctionImport],
    types: &[FuncType],
    params: &[StackValue],
) -> Result<()> {
    let import = imports
        .get(function_index as usize)
        .ok_or_else(|| anyhow!("calls to user-defined functions are not supported"))?;
    let type_index = get_import_type_index(import)?;
    let func_type = types.get(type_index as usize).ok_or_else(|| {
        anyhow!(
            "invalid type index {} for import {}",
            type_index,
            import.name
        )
    })?;

    let module = import.module.to_ascii_lowercase();
    match module.as_str() {
        "opcode" => emit_opcode_call(import, func_type, params, script),
        "syscall" => {
            emit_syscall_call(import, script)?;
            Ok(())
        }
        "neo" => {
            emit_neo_syscall(import, script)?;
            Ok(())
        }
        other => bail!("unsupported import module '{}::{}'", other, import.name),
    }
}

fn try_handle_env_import(
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
        let dest_value = params.get(0).and_then(|value| value.const_value);
        value_stack.push(StackValue {
            const_value: dest_value,
            bytecode_start: None,
        });
    }

    Ok(true)
}

fn emit_opcode_call(
    import: &FunctionImport,
    func_type: &FuncType,
    params: &[StackValue],
    script: &mut Vec<u8>,
) -> Result<()> {
    if import.name.eq_ignore_ascii_case("raw") {
        ensure_param_count(import, params, 1)?;
        let value = truncate_literal(params.last().unwrap(), script, 1)? as u8;
        script.push(value);
        return Ok(());
    }

    if import.name.eq_ignore_ascii_case("raw4") {
        ensure_param_count(import, params, 1)?;
        let value = truncate_literal(params.last().unwrap(), script, 4)? as i64;
        script.extend_from_slice(&(value as u32).to_le_bytes());
        return Ok(());
    }

    let info = opcodes::lookup(&import.name)
        .ok_or_else(|| anyhow!("unknown NeoVM opcode '{}'", import.name))?;

    if info.operand_size_prefix != 0 {
        bail!(
            "opcode '{}' has a variable-size operand; emit it manually via opcode.raw/raw4",
            import.name
        );
    }

    if !func_type.results().is_empty() {
        bail!(
            "imported opcode '{}' must have signature (param ..., result void)",
            import.name
        );
    }

    let expected_params = func_type.params().len();
    if expected_params != params.len() {
        bail!(
            "imported opcode '{}' expects {} parameter(s) but {} were provided",
            import.name,
            expected_params,
            params.len()
        );
    }

    if info.operand_size == 0 {
        if !params.is_empty() {
            bail!("opcode '{}' does not take immediate operands", import.name);
        }
        script.push(info.byte);
        return Ok(());
    }

    ensure_param_count(import, params, 1)?;
    let immediate = truncate_literal(params.last().unwrap(), script, info.operand_size as usize)?;

    script.push(info.byte);
    match info.operand_size {
        1 => script.push(immediate as u8),
        2 => script.extend_from_slice(&(immediate as i16).to_le_bytes()),
        4 => script.extend_from_slice(&(immediate as i32).to_le_bytes()),
        8 => script.extend_from_slice(&(immediate as i64).to_le_bytes()),
        other => {
            bail!(
                "unsupported operand size {} for opcode '{}'; use opcode.raw/raw4",
                other,
                import.name
            );
        }
    }

    Ok(())
}

fn emit_syscall_call(import: &FunctionImport, script: &mut Vec<u8>) -> Result<()> {
    let syscall = syscalls::lookup(&import.name)
        .ok_or_else(|| anyhow!("unknown syscall '{}'", import.name))?;
    let opcode =
        opcodes::lookup("SYSCALL").ok_or_else(|| anyhow!("SYSCALL opcode metadata missing"))?;

    // SYSCALL has a 4-byte immediate hash.
    if opcode.operand_size != 4 || opcode.operand_size_prefix != 0 {
        bail!("unexpected SYSCALL operand metadata");
    }

    script.push(opcode.byte);
    script.extend_from_slice(&syscall.hash.to_le_bytes());
    Ok(())
}

fn emit_neo_syscall(import: &FunctionImport, script: &mut Vec<u8>) -> Result<()> {
    let syscall_name = neo_syscalls::lookup_neo_syscall(&import.name)
        .ok_or_else(|| anyhow!("unknown Neo syscall import '{}'", import.name))?;
    let syscall = syscalls::lookup(syscall_name)
        .ok_or_else(|| anyhow!("syscall '{}' not found", syscall_name))?;
    let opcode =
        opcodes::lookup("SYSCALL").ok_or_else(|| anyhow!("SYSCALL opcode metadata missing"))?;

    if opcode.operand_size != 4 || opcode.operand_size_prefix != 0 {
        bail!("unexpected SYSCALL operand metadata");
    }

    script.push(opcode.byte);
    script.extend_from_slice(&syscall.hash.to_le_bytes());
    Ok(())
}

fn ensure_param_count(
    import: &FunctionImport,
    params: &[StackValue],
    expected: usize,
) -> Result<()> {
    if params.len() != expected {
        bail!(
            "import '{}' expects {} parameter(s) but {} were provided",
            import.name,
            expected,
            params.len()
        );
    }
    Ok(())
}

fn literal_instruction_len(script: &[u8], start: usize) -> Result<usize> {
    if start >= script.len() {
        bail!(
            "invalid literal start {} for script of length {}",
            start,
            script.len()
        );
    }

    let opcode = script[start];
    let len = if opcode == PUSHM1 || opcode == PUSH0 {
        1usize
    } else if (PUSH_BASE + 1..=PUSH_BASE + 16).contains(&opcode) {
        1usize
    } else if opcode == PUSHINT8 {
        1usize + 1
    } else if opcode == PUSHINT16 {
        1usize + 2
    } else if opcode == PUSHINT32 {
        1usize + 4
    } else if opcode == PUSHINT64 {
        1usize + 8
    } else if opcode == PUSHINT128 {
        1usize + 16
    } else {
        bail!(
            "unable to determine literal length for opcode 0x{:02X}",
            opcode
        );
    };

    if start + len > script.len() {
        bail!("literal extends beyond script bounds");
    }

    Ok(len)
}

fn truncate_literal(param: &StackValue, script: &mut Vec<u8>, max_bytes: usize) -> Result<i128> {
    let value = param
        .const_value
        .ok_or_else(|| anyhow!("import argument must be a compile-time constant"))?;
    // Treat the literal as signed; validate it fits within the requested bytes.
    let bits = max_bytes * 8;
    if bits == 0 || bits > 64 {
        bail!("unsupported immediate width {} bytes", max_bytes);
    }
    let min = -(1i128 << (bits - 1));
    let max_signed = (1i128 << (bits - 1)) - 1;
    let max_unsigned = (1i128 << bits) - 1;
    if !(min <= value && value <= max_signed) && !(0 <= value && value <= max_unsigned) {
        bail!(
            "literal value {} does not fit in {} byte(s) for opcode immediate",
            value,
            max_bytes
        );
    }

    let Some(start) = param.bytecode_start else {
        bail!("import argument cannot be materialised as an immediate; ensure it is a literal");
    };

    if start >= script.len() {
        bail!("internal error: literal start beyond current script length");
    }

    let literal_len = literal_instruction_len(script, start)?;
    let literal_end = start + literal_len;
    if literal_end == script.len() {
        script.truncate(start);
    } else {
        let drop = lookup_opcode("DROP")?;
        script.push(drop.byte);
    }

    Ok(value)
}

#[derive(Debug, Clone)]
struct LocalState {
    kind: LocalKind,
    const_value: Option<i128>,
}

#[derive(Debug, Clone)]
enum LocalKind {
    Param(u32),
    Local(u32),
}

enum UnsignedOp {
    Div,
    Rem,
}

#[derive(Clone, Copy)]
enum CompareOp {
    Lt,
    Le,
    Gt,
    Ge,
}

impl CompareOp {
    fn opcode_name(self) -> &'static str {
        match self {
            CompareOp::Lt => "LT",
            CompareOp::Le => "LE",
            CompareOp::Gt => "GT",
            CompareOp::Ge => "GE",
        }
    }

    fn evaluate_signed(self, lhs: i128, rhs: i128, bits: u32) -> bool {
        match bits {
            32 => {
                let lhs = lhs as i32;
                let rhs = rhs as i32;
                self.evaluate_order(lhs, rhs)
            }
            64 => {
                let lhs = lhs as i64;
                let rhs = rhs as i64;
                self.evaluate_order(lhs, rhs)
            }
            other => unreachable!("unsupported signed comparison width {}", other),
        }
    }

    fn evaluate_unsigned(self, lhs: u128, rhs: u128) -> bool {
        self.evaluate_order(lhs, rhs)
    }

    fn evaluate_order<T: PartialOrd>(self, lhs: T, rhs: T) -> bool {
        match self {
            CompareOp::Lt => lhs < rhs,
            CompareOp::Le => lhs <= rhs,
            CompareOp::Gt => lhs > rhs,
            CompareOp::Ge => lhs >= rhs,
        }
    }
}

fn emit_unsigned_binary_op(
    script: &mut Vec<u8>,
    op: UnsignedOp,
    lhs: StackValue,
    rhs: StackValue,
    bits: u32,
) -> Result<StackValue> {
    mask_unsigned_operands(script, bits)?;

    let opcode_name = match op {
        UnsignedOp::Div => "DIV",
        UnsignedOp::Rem => "MOD",
    };
    script.push(lookup_opcode(opcode_name)?.byte);

    let mask = (1u128 << bits) - 1;
    let const_value = match (lhs.const_value, rhs.const_value) {
        (Some(a), Some(b)) => {
            let dividend = (a as u128) & mask;
            let divisor = (b as u128) & mask;
            if divisor == 0 {
                None
            } else {
                let value = match op {
                    UnsignedOp::Div => dividend / divisor,
                    UnsignedOp::Rem => dividend % divisor,
                };
                Some(value as i128)
            }
        }
        _ => None,
    };

    Ok(StackValue {
        const_value,
        bytecode_start: None,
    })
}

fn emit_signed_compare(
    script: &mut Vec<u8>,
    lhs: StackValue,
    rhs: StackValue,
    bits: u32,
    kind: CompareOp,
) -> Result<StackValue> {
    emit_binary_op(script, kind.opcode_name(), lhs, rhs, |a, b| {
        let cmp = kind.evaluate_signed(a, b, bits);
        Some(if cmp { 1 } else { 0 })
    })
}

fn emit_unsigned_compare(
    script: &mut Vec<u8>,
    lhs: StackValue,
    rhs: StackValue,
    bits: u32,
    kind: CompareOp,
) -> Result<StackValue> {
    mask_unsigned_operands(script, bits)?;
    let mask = (1u128 << bits) - 1;
    emit_binary_op(script, kind.opcode_name(), lhs, rhs, |a, b| {
        let lhs = (a as u128) & mask;
        let rhs = (b as u128) & mask;
        let cmp = kind.evaluate_unsigned(lhs, rhs);
        Some(if cmp { 1 } else { 0 })
    })
}

fn mask_unsigned_operands(script: &mut Vec<u8>, bits: u32) -> Result<()> {
    let mask_value = ((1u128 << bits) - 1) as i128;
    let and = lookup_opcode("AND")?;
    let swap = lookup_opcode("SWAP")?;

    let _ = emit_push_int(script, mask_value);
    script.push(and.byte);
    script.push(swap.byte);
    let _ = emit_push_int(script, mask_value);
    script.push(and.byte);
    script.push(swap.byte);

    Ok(())
}

fn ensure_select_type_supported(tys: &[ValType]) -> Result<()> {
    if tys.len() > 1 {
        bail!("typed select results with more than one value are not supported");
    }
    for ty in tys {
        match ty {
            ValType::I32 | ValType::I64 => {}
            other => bail!(
                "typed select with unsupported value type {:?}; only i32/i64 are supported",
                other
            ),
        }
    }
    Ok(())
}
