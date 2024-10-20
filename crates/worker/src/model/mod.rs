use repos::entity::{EpisodeId, ScriptId};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub(crate) enum Args {
    GenerateAudio { episode_id: EpisodeId },
    EvaluateScript { script_id: ScriptId },
    NewEpisode { pre_episode_id: EpisodeId },
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