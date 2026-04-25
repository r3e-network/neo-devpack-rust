// Copyright (c) 2025-2026 R3E Network
// SPDX-License-Identifier: MIT

use std::collections::{BTreeSet, HashSet, VecDeque};

use super::*;

pub(super) struct FunctionDependencyGraph {
    import_function_count: usize,
    direct_edges: Vec<Vec<u32>>,
    indirect_edges: Vec<bool>,
    indirect_roots: BTreeSet<u32>,
}

impl FunctionDependencyGraph {
    pub(super) fn function_requires_runtime_init(
        &self,
        root: u32,
        direct_init_functions: &HashSet<u32>,
    ) -> bool {
        if direct_init_functions.contains(&root) {
            return true;
        }
        if (root as usize) < self.import_function_count {
            return false;
        }

        let mut seen = HashSet::new();
        let mut queue = VecDeque::from([root]);

        while let Some(function_index) = queue.pop_front() {
            if !seen.insert(function_index) {
                continue;
            }
            if direct_init_functions.contains(&function_index) {
                return true;
            }

            let Some(defined_idx) =
                (function_index as usize).checked_sub(self.import_function_count)
            else {
                continue;
            };
            if defined_idx >= self.direct_edges.len() {
                continue;
            }

            for &callee in &self.direct_edges[defined_idx] {
                if (callee as usize) >= self.import_function_count {
                    queue.push_back(callee);
                }
            }

            if self.indirect_edges[defined_idx] {
                for &callee in &self.indirect_roots {
                    if (callee as usize) >= self.import_function_count {
                        queue.push_back(callee);
                    }
                }
            }
        }

        false
    }
}

pub(super) fn analyze_function_dependency_graph(bytes: &[u8]) -> Result<FunctionDependencyGraph> {
    let mut import_function_count: usize = 0;
    let mut element_function_refs: BTreeSet<u32> = BTreeSet::new();
    let mut ref_func_refs: BTreeSet<u32> = BTreeSet::new();
    let mut direct_edges: Vec<Vec<u32>> = Vec::new();
    let mut indirect_edges: Vec<bool> = Vec::new();

    for payload in Parser::new(0).parse_all(bytes) {
        match payload? {
            Payload::ImportSection(reader) => {
                for import in reader {
                    if matches!(import?.ty, TypeRef::Func(_)) {
                        import_function_count += 1;
                    }
                }
            }
            Payload::ElementSection(reader) => {
                for element in reader {
                    let element = element?;
                    match element.items {
                        wasmparser::ElementItems::Functions(functions) => {
                            for function_index in functions {
                                element_function_refs.insert(function_index?);
                            }
                        }
                        wasmparser::ElementItems::Expressions(_, exprs) => {
                            for expr in exprs {
                                let expr = expr?;
                                let mut operators = expr.get_operators_reader();
                                while !operators.eof() {
                                    match operators.read()? {
                                        Operator::RefFunc { function_index } => {
                                            element_function_refs.insert(function_index);
                                        }
                                        Operator::End => break,
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Payload::CodeSectionEntry(body) => {
                let mut callees = Vec::new();
                let mut uses_indirect = false;
                let mut ops = body.get_operators_reader()?;

                while !ops.eof() {
                    match ops.read()? {
                        Operator::Call { function_index } => callees.push(function_index),
                        Operator::CallIndirect { .. } => uses_indirect = true,
                        Operator::RefFunc { function_index } => {
                            ref_func_refs.insert(function_index);
                        }
                        _ => {}
                    }
                }

                direct_edges.push(callees);
                indirect_edges.push(uses_indirect);
            }
            Payload::End(_) => break,
            _ => {}
        }
    }

    element_function_refs.extend(ref_func_refs);

    Ok(FunctionDependencyGraph {
        import_function_count,
        direct_edges,
        indirect_edges,
        indirect_roots: element_function_refs,
    })
}

/// Computes the set of reachable **defined** function indices (absolute Wasm function indices).
///
/// Reachability roots are exported functions and start function. We follow direct `call`
/// edges through function bodies. If any reachable function uses `call_indirect`, all
/// functions referenced by element segments are treated as additional roots.
pub(super) fn analyze_reachable_defined_functions(bytes: &[u8]) -> Result<HashSet<u32>> {
    let mut import_function_count: usize = 0;
    let mut roots: BTreeSet<u32> = BTreeSet::new();
    let mut element_function_refs: BTreeSet<u32> = BTreeSet::new();
    let mut ref_func_refs: BTreeSet<u32> = BTreeSet::new();
    let mut call_edges: Vec<Vec<u32>> = Vec::new();
    let mut has_call_indirect: Vec<bool> = Vec::new();

    for payload in Parser::new(0).parse_all(bytes) {
        match payload? {
            Payload::ImportSection(reader) => {
                for import in reader {
                    if matches!(import?.ty, TypeRef::Func(_)) {
                        import_function_count += 1;
                    }
                }
            }
            Payload::ExportSection(reader) => {
                for export in reader {
                    let export = export?;
                    if export.kind == ExternalKind::Func {
                        roots.insert(export.index);
                    }
                }
            }
            Payload::StartSection { func, .. } => {
                roots.insert(func);
            }
            Payload::ElementSection(reader) => {
                for element in reader {
                    let element = element?;
                    match element.items {
                        wasmparser::ElementItems::Functions(functions) => {
                            for function_index in functions {
                                element_function_refs.insert(function_index?);
                            }
                        }
                        wasmparser::ElementItems::Expressions(_, exprs) => {
                            for expr in exprs {
                                let expr = expr?;
                                let mut operators = expr.get_operators_reader();
                                while !operators.eof() {
                                    match operators.read()? {
                                        Operator::RefFunc { function_index } => {
                                            element_function_refs.insert(function_index);
                                        }
                                        Operator::End => break,
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Payload::CodeSectionEntry(body) => {
                let mut direct_callees: Vec<u32> = Vec::new();
                let mut uses_indirect = false;
                let mut ops = body.get_operators_reader()?;

                while !ops.eof() {
                    match ops.read()? {
                        Operator::Call { function_index } => direct_callees.push(function_index),
                        Operator::CallIndirect { .. } => uses_indirect = true,
                        Operator::RefFunc { function_index } => {
                            ref_func_refs.insert(function_index);
                        }
                        _ => {}
                    }
                }

                call_edges.push(direct_callees);
                has_call_indirect.push(uses_indirect);
            }
            Payload::End(_) => break,
            _ => {}
        }
    }

    let defined_count = call_edges.len();
    let mut reachable: HashSet<u32> = HashSet::with_capacity(defined_count);
    let mut queue: VecDeque<u32> = VecDeque::new();

    for &root in &roots {
        if (root as usize) >= import_function_count {
            let defined_idx = (root as usize) - import_function_count;
            if defined_idx < defined_count {
                queue.push_back(root);
            }
        }
    }

    let mut included_table_roots = false;

    while let Some(function_index) = queue.pop_front() {
        if !reachable.insert(function_index) {
            continue;
        }

        let defined_idx = (function_index as usize) - import_function_count;
        if defined_idx >= defined_count {
            continue;
        }

        for &callee in &call_edges[defined_idx] {
            if (callee as usize) < import_function_count {
                continue;
            }
            let callee_defined_idx = (callee as usize) - import_function_count;
            if callee_defined_idx < defined_count {
                queue.push_back(callee);
            }
        }

        if has_call_indirect[defined_idx] && !included_table_roots {
            included_table_roots = true;
            for &table_func in &element_function_refs {
                if (table_func as usize) < import_function_count {
                    continue;
                }
                let table_defined_idx = (table_func as usize) - import_function_count;
                if table_defined_idx < defined_count {
                    queue.push_back(table_func);
                }
            }
            for &func in &ref_func_refs {
                if (func as usize) < import_function_count {
                    continue;
                }
                let defined_idx = (func as usize) - import_function_count;
                if defined_idx < defined_count {
                    queue.push_back(func);
                }
            }
        }
    }

    Ok(reachable)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reachability_skips_unreferenced_defined_functions() {
        let wasm = wat::parse_str(
            r#"(module
                  (func $dead (result i32)
                    i32.const 999)
                  (func $leaf (result i32)
                    i32.const 7)
                  (func $mid (result i32)
                    call $leaf)
                  (func (export "main") (result i32)
                    call $mid))"#,
        )
        .expect("valid wat");

        let reachable = analyze_reachable_defined_functions(&wasm).expect("analysis succeeds");

        assert!(
            !reachable.contains(&0),
            "dead function should be unreachable"
        );
        assert!(reachable.contains(&1));
        assert!(reachable.contains(&2));
        assert!(reachable.contains(&3));
        assert_eq!(reachable.len(), 3);
    }

    #[test]
    fn reachability_includes_table_functions_when_indirect_calls_are_used() {
        let wasm = wat::parse_str(
            r#"(module
                  (type $t (func (result i32)))
                  (table 2 funcref)
                  (func $f0 (type $t) (result i32)
                    i32.const 10)
                  (func $f1 (type $t) (result i32)
                    i32.const 20)
                  (func (export "main") (type $t) (result i32)
                    i32.const 0
                    call_indirect (type $t))
                  (elem (i32.const 0) $f0 $f1))"#,
        )
        .expect("valid wat");

        let reachable = analyze_reachable_defined_functions(&wasm).expect("analysis succeeds");

        assert!(reachable.contains(&0));
        assert!(reachable.contains(&1));
        assert!(reachable.contains(&2));
        assert_eq!(reachable.len(), 3);
    }
}
