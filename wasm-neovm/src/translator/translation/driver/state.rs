use super::*;
use crate::config::ManifestOverlay;

use exports::ExportedFunction;

pub(super) struct DriverState {
    pub(super) contract_name: String,
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

impl DriverState {
    pub(super) fn new(config: TranslationConfig) -> Self {
        // Pre-size collections based on typical contract sizes (Round 62, 63 optimizations)
        const TYPICAL_SCRIPT_CAPACITY: usize = 4096;
        const TYPICAL_METHODS_CAPACITY: usize = 32;

        Self {
            contract_name: config.contract_name.into_inner(),
            adapter: get_adapter(config.source_chain),
            extra_manifest_overlay: config.extra_manifest_overlay,
            frontend: ModuleFrontend::new(),
            exported_funcs: BTreeMap::new(),
            import_export_indices: BTreeSet::new(),
            tables: Vec::with_capacity(4),
            // Pre-allocate script buffer with typical capacity (Round 62 optimization)
            script: Vec::with_capacity(TYPICAL_SCRIPT_CAPACITY),
            // Use pre-allocated RuntimeHelpers (Round 62, 63 optimizations)
            runtime: RuntimeHelpers::with_capacity(8, 8, 16),
            feature_tracker: FeatureTracker::default(),
            methods: Vec::with_capacity(TYPICAL_METHODS_CAPACITY),
            // Pre-size HashSet for faster lookups (Round 63 optimization)
            overlay_safe_methods: HashSet::with_capacity(8),
            manifest_overlay: None,
            section_method_tokens: Vec::with_capacity(16),
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
