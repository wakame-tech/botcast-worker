use super::ScriptRepo;
use anyhow::anyhow;
use script_runtime::Manuscript;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub(crate) struct ScriptService {
    pub(crate) script_repo: Arc<dyn ScriptRepo>,
}

impl ScriptService {
    pub(crate) async fn evaluate_to_manuscript(
        &self,
        script_id: Uuid,
    ) -> anyhow::Result<Manuscript> {
        // let work_dir = use_work_dir(&task_id)?;
        let script = self
            .script_repo
            .find_by_id(&script_id)
            .await?
            .ok_or_else(|| anyhow!("Script not found"))?;
        let manuscript = script_runtime::runtime::run(&script.template).await?;
        Ok(manuscript)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::script_repo::DummyScriptRepo;
    use anyhow::Result;
    use script_runtime::Section;

    #[tokio::test]
    async fn test_evaluate_to_manuscript() -> Result<()> {
        let template = serde_json::json!({
            "title": { "$eval": "today('%Y')" },
            "sections": Vec::<Section>::new(),
        });
        let service = ScriptService {
            script_repo: Arc::new(DummyScriptRepo {
                template: template.clone(),
            }),
        };
        let manuscript = service.evaluate_to_manuscript(Uuid::new_v4()).await?;
        assert!(!manuscript.title.is_empty());
        Ok(())
    }
}
