pub mod imports;
pub mod runtime;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Urn(pub String);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Section {
    Serif { speaker: Urn, text: String },
}

/// evaluated script
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Manuscript {
    pub title: String,
    pub sections: Vec<Section>,
}
