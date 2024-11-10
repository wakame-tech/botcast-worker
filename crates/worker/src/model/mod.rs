use repos::entity::{EpisodeId, PodcastId};
use std::collections::BTreeMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub(crate) enum Args {
    GenerateAudio {
        episode_id: EpisodeId,
    },
    EvaluateTemplate {
        template: serde_json::Value,
        context: BTreeMap<String, serde_json::Value>,
    },
    NewEpisode {
        podcast_id: PodcastId,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum Section {
    Serif { speaker: String, text: String },
}

/// evaluated script
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Manuscript {
    pub title: String,
    pub sections: Vec<Section>,
}
