use crate::error::Error;
use api::client::ApiClient;
use repos::{entity::ScriptId, repo::ScriptRepo};
use script_runtime::{
    imports::{
        llm::{create_thread, delete_thread, register_llm_functions},
        repo::register_repo_functions,
    },
    runtime::ScriptRuntime,
};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone)]
pub(crate) struct ScriptService {
    script_repo: Arc<dyn ScriptRepo>,
}

impl ScriptService {
    pub(crate) fn new(script_repo: Arc<dyn ScriptRepo>) -> Self {
        Self { script_repo }
    }

    pub(crate) async fn run_template(
        &self,
        token: String,
        template: &serde_json::Value,
        values: BTreeMap<String, serde_json::Value>,
    ) -> anyhow::Result<serde_json::Value, Error> {
        let api_endpoint = std::env::var("API_ENDPOINT")
            .map_err(|_| Error::InvalidInput(anyhow::anyhow!("API_ENDPOINT is not set")))?;
        let client = Arc::new(ApiClient::new(&api_endpoint, &token));
        let mut runtime = ScriptRuntime::default();
        register_repo_functions(&mut runtime, client);
        let open_ai_api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY is not set");
        let thread_id = create_thread(open_ai_api_key.clone())
            .await
            .map_err(Error::Script)?;
        register_llm_functions(&mut runtime, open_ai_api_key.clone(), thread_id.clone());
        let res = runtime.run(template, values).await.map_err(Error::Script)?;
        delete_thread(open_ai_api_key.clone(), thread_id)
            .await
            .map_err(Error::Script)?;
        Ok(res)
    }

    pub(crate) async fn update_template(
        &self,
        script_id: &ScriptId,
        template: serde_json::Value,
    ) -> anyhow::Result<(), Error> {
        let mut script = self.script_repo.find_by_id(script_id).await?;

        script.template = template;
        self.script_repo.update(&script).await?;
        Ok(())
    }
}
