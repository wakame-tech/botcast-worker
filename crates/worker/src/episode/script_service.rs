use repos::{repo::ScriptRepo, script_repo};
use std::sync::Arc;
use uuid::Uuid;

pub(crate) fn script_service() -> ScriptService {
    ScriptService {
        script_repo: script_repo(),
    }
}

#[derive(Clone)]
pub(crate) struct ScriptService {
    script_repo: Arc<dyn ScriptRepo>,
}

impl ScriptService {
    pub(crate) async fn evaluate_once(
        &self,
        template: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        script_runtime::runtime::run(&template).await
    }

    pub(crate) async fn evaluate_script(
        &self,
        script_id: Uuid,
    ) -> anyhow::Result<serde_json::Value> {
        let mut script = self
            .script_repo
            .find_by_id(&script_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Script not found"))?;
        let result = script_runtime::runtime::run(&script.template).await?;
        script.result = Some(result.clone());
        self.script_repo.update(&script).await?;
        Ok(result)
    }

    pub(crate) async fn update_script(
        &self,
        script_id: Uuid,
        template: serde_json::Value,
    ) -> anyhow::Result<()> {
        let mut script = self
            .script_repo
            .find_by_id(&script_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Script not found"))?;
        script.template = template;
        self.script_repo.update(&script).await?;
        Ok(())
    }
}
