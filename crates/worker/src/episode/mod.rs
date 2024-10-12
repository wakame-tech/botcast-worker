pub mod episode_service;
mod generate_audio;
mod resolve_urn;
pub mod script_service;

use script_runtime::Urn;

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
