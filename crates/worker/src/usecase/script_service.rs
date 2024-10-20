use repos::{
    entity::ScriptId,
    provider::{ProvideScriptRepo, Provider},
    repo::ScriptRepo,
};
use script_runtime::runtime;
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct ScriptService {
    script_repo: Arc<dyn ScriptRepo>,
}

impl ScriptService {
    pub(crate) fn new(provider: Provider) -> Self {
        Self {
            script_repo: provider.script_repo(),
        }
    }

    pub(crate) async fn evaluate_once(
        &self,
        template: &serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        runtime::run(template).await
    }

    pub(crate) async fn evaluate_script(
        &self,
        script_id: &ScriptId,
    ) -> anyhow::Result<serde_json::Value> {
        let mut script = self.script_repo.find_by_id(&script_id).await?;

        let result = runtime::run(&script.template).await?;

        script.result = Some(result.clone());
        self.script_repo.update(&script).await?;
        Ok(result)
    }

    pub(crate) async fn update_template(
        &self,
        script_id: &ScriptId,
        template: serde_json::Value,
    ) -> anyhow::Result<()> {
        let mut script = self.script_repo.find_by_id(&script_id).await?;

        script.template = template;
        self.script_repo.update(&script).await?;
        Ok(())
    }
}
