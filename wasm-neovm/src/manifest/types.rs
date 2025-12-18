use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ManifestParameter {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ManifestMethod {
    pub name: String,
    pub parameters: Vec<ManifestParameter>,
    #[serde(rename = "returntype")]
    pub return_type: String,
    pub offset: u32,
    pub safe: bool,
}

#[derive(Debug, Clone)]
pub struct RenderedManifest {
    pub value: serde_json::Value,
}

impl RenderedManifest {
    pub fn to_string(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(&self.value)
    }
}
