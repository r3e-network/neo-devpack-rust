use super::*;
use crate::ManifestOverlay;

use exports::ExportedFunction;

pub(super) struct DriverState<'a> {
    pub(super) contract_name: &'a str,
    pub(super) adapter: Box<dyn ChainAdapter>,
    pub(super) extra_manifest_overlay: Option<ManifestOverlay>,
    pub(super) frontend: ModuleFrontend,
    pub(super) exported_funcs: BTreeMap<u32, ExportedFunction>,
    pub(super) import_export_indices: BTreeSet<usize>,
    pub(super) tables: Vec<TableInfo>,
    pub(super) script: Vec<u8>,
    pub(super) runtime: RuntimeHelpers,
    pub(super) feature_tracker: FeatureTracker,
    pub(super) methods: Vec<ManifestMethod>,
    pub(super) overlay_safe_methods: HashSet<String>,
    pub(super) manifest_overlay: Option<Value>,
    pub(super) section_method_tokens: Vec<MethodToken>,
    pub(super) section_source: Option<String>,
    pub(super) saw_code_section: bool,
    pub(super) next_defined_index: usize,
    pub(super) function_registry: Option<FunctionRegistry>,
    pub(super) start_function: Option<u32>,
    pub(super) start_defined_offset: Option<usize>,
}

impl<'a> DriverState<'a> {
    pub(super) fn new(config: TranslationConfig<'a>) -> Self {
        Self {
            contract_name: config.contract_name,
            adapter: get_adapter(config.source_chain),
            extra_manifest_overlay: config.extra_manifest_overlay,
            frontend: ModuleFrontend::new(),
            exported_funcs: BTreeMap::new(),
            import_export_indices: BTreeSet::new(),
            tables: Vec::new(),
            script: Vec::new(),
            runtime: RuntimeHelpers::default(),
            feature_tracker: FeatureTracker::default(),
            methods: Vec::new(),
            overlay_safe_methods: HashSet::new(),
            manifest_overlay: None,
            section_method_tokens: Vec::new(),
            section_source: None,
            saw_code_section: false,
            next_defined_index: 0,
            function_registry: None,
            start_function: None,
            start_defined_offset: None,
        }
    }

    pub(super) fn translate(mut self, bytes: &[u8]) -> Result<Translation> {
        self.parse_payloads(bytes)?;
        self.finalize()
    }
}

