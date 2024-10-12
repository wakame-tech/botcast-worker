use super::EpisodeRepo;
use anyhow::anyhow;
use script_runtime::{Manuscript, Section};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub(crate) struct ScriptService {
    pub(crate) episode_repo: Arc<dyn EpisodeRepo>,
}

impl ScriptService {
    async fn evaluate(&self, template: &serde_json::Value) -> anyhow::Result<Manuscript> {
        let manuscript = script_runtime::runtime::run(template).await?;
        Ok(manuscript)
    }

    pub(crate) async fn evaluate_to_manuscript(
        &self,
        _task_id: Uuid,
        episode_id: Uuid,
    ) -> anyhow::Result<Manuscript> {
        // let work_dir = use_work_dir(&task_id)?;
        let episode = self
            .episode_repo
            .find_by_id(&episode_id)
            .await?
            .ok_or_else(|| anyhow!("Episode not found"))?;

        let template = serde_json::json!({
            "title": episode.title,
            "sections": Vec::<Section>::new(),
        });
        let manuscript = self.evaluate(&template).await?;
        Ok(manuscript)
    }
}
