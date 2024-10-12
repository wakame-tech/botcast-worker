use repos::repo::ScriptRepo;
use script_runtime::Manuscript;
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct ScriptService {
    pub(crate) script_repo: Arc<dyn ScriptRepo>,
}

impl ScriptService {
    pub(crate) async fn evaluate_to_manuscript(
        &self,
        template: serde_json::Value,
    ) -> anyhow::Result<Manuscript> {
        let manuscript = script_runtime::runtime::run(&template).await?;
        Ok(manuscript)
    }
}
