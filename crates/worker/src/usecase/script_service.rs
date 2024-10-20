use crate::error::Error;
use repos::{entity::ScriptId, provider::ProvideScriptRepo, repo::ScriptRepo};
use script_runtime::runtime;
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct ScriptService {
    script_repo: Arc<dyn ScriptRepo>,
}

impl ScriptService {
    pub(crate) fn new<P>(provider: P) -> Self
    where
        P: ProvideScriptRepo,
    {
        Self {
            script_repo: provider.script_repo(),
        }
    }

    pub(crate) async fn evaluate_once(
        &self,
        template: &serde_json::Value,
    ) -> anyhow::Result<serde_json::Value, Error> {
        runtime::run(template).await.map_err(Error::Other)
    }

    pub(crate) async fn evaluate_script(
        &self,
        script_id: &ScriptId,
    ) -> anyhow::Result<serde_json::Value, Error> {
        let mut script = self.script_repo.find_by_id(&script_id).await?;

        log::info!("Evaluating script: {:?}", script);
        let result = runtime::run(&script.template)
            .await
            .map_err(Error::Script)?;

        script.result = Some(result.clone());
        self.script_repo.update(&script).await?;
        Ok(result)
    }

    pub(crate) async fn update_template(
        &self,
        script_id: &ScriptId,
        template: serde_json::Value,
    ) -> anyhow::Result<(), Error> {
        let mut script = self.script_repo.find_by_id(&script_id).await?;

        script.template = template;
        self.script_repo.update(&script).await?;
        Ok(())
    }
}
