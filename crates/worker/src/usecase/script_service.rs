use crate::error::Error;
use api::client::ApiClient;
use futures::future::try_join_all;
use repos::{
    entity::ScriptId,
    repo::{ScriptRepo, SecretRepo},
};
use script_runtime::{
    imports::{
        api::register_api_functions,
        llm::{create_thread, delete_thread, register_llm_functions},
    },
    runtime::ScriptRuntime,
};
use std::{collections::BTreeMap, sync::Arc};
use tracing::instrument;
use uuid::Uuid;

#[derive(Clone)]
pub(crate) struct ScriptService {
    script_repo: Arc<dyn ScriptRepo>,
    secret_repo: Arc<dyn SecretRepo>,
}

impl ScriptService {
    pub(crate) fn new(script_repo: Arc<dyn ScriptRepo>, secret_repo: Arc<dyn SecretRepo>) -> Self {
        Self {
            script_repo,
            secret_repo,
        }
    }

    async fn replace_context_to_secrets(
        &self,
        user_id: Uuid,
        context: BTreeMap<String, serde_json::Value>,
    ) -> anyhow::Result<BTreeMap<String, serde_json::Value>, Error> {
        let context: Result<_, Error> =
            try_join_all(context.into_iter().map(|(key, value)| async {
                match (key, value) {
                    (key, serde_json::Value::String(value)) if value.starts_with("$") => {
                        let name = value.trim_start_matches("$");
                        let secret = self.secret_repo.find_by_name(&user_id, &name).await?;
                        let secret = secret.decrypted_secret.ok_or_else(|| {
                            Error::InvalidInput(anyhow::anyhow!(
                                "Secret with name {} is not found",
                                name
                            ))
                        })?;
                        Ok((key, serde_json::Value::String(secret)))
                    }
                    e => Ok(e),
                }
            }))
            .await;
        Ok(context?.into_iter().collect())
    }

    #[instrument(skip(self, token), ret)]
    pub(crate) async fn run_template(
        &self,
        token: String,
        template: &serde_json::Value,
        context: BTreeMap<String, serde_json::Value>,
    ) -> anyhow::Result<serde_json::Value, Error> {
        let api_endpoint = std::env::var("API_ENDPOINT")
            .map_err(|_| Error::InvalidInput(anyhow::anyhow!("API_ENDPOINT is not set")))?;
        let client = Arc::new(ApiClient::new(&api_endpoint, &token));
        let me = client.me().await.map_err(Error::Other)?;
        let user_id: Uuid = me
            .id
            .parse()
            .map_err(|_| Error::InvalidInput(anyhow::anyhow!("invalid user id")))?;

        let context = self.replace_context_to_secrets(user_id, context).await?;

        let open_ai_api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY is not set");
        let thread_id = create_thread(open_ai_api_key.clone())
            .await
            .map_err(Error::Script)?;

        let mut runtime = ScriptRuntime::default();
        register_api_functions(&mut runtime, client);
        register_llm_functions(&mut runtime, open_ai_api_key.clone(), thread_id.clone());

        let res = runtime
            .run(template, context)
            .await
            .map_err(Error::Script)?;
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
