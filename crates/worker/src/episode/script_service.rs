use super::Manuscript;
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
    pub(crate) async fn evaluate_to_manuscript(
        &self,
        template: serde_json::Value,
    ) -> anyhow::Result<Manuscript> {
        let evaluated = script_runtime::runtime::run(&template).await?;
        Ok(serde_json::from_value(evaluated)?)
    }

    pub(crate) async fn update_script(
        &self,
        id: Uuid,
        template: serde_json::Value,
    ) -> anyhow::Result<()> {
        let mut script = self
            .script_repo
            .find_by_id(&id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Script not found"))?;
        script.template = template;
        self.script_repo.update(&script).await?;
        Ok(())
    }
}
