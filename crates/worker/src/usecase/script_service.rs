use crate::error::Error;
use api::client::ApiClient;
use futures::future::try_join_all;
use repos::{
    entity::ScriptId,
    repo::{ScriptRepo, SecretRepo},
};
use script_runtime::{plugins::botcast_api::BotCastApiPlugin, runtime::ScriptRuntime};
use std::{collections::BTreeMap, sync::Arc};
use tracing::instrument;
use uuid::Uuid;

#[derive(Clone)]
pub(crate) struct ScriptService {
    script_repo: Arc<dyn ScriptRepo>,
    secret_repo: Arc<dyn SecretRepo>,
    api_client: Arc<ApiClient>,
}

impl ScriptService {
    pub(crate) fn new(
        script_repo: Arc<dyn ScriptRepo>,
        secret_repo: Arc<dyn SecretRepo>,
        api_client: Arc<ApiClient>,
    ) -> Self {
        Self {
            script_repo,
            secret_repo,
            api_client,
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

    #[instrument(skip(self), ret)]
    pub(crate) async fn run_template(
        &self,
        template: &serde_json::Value,
        parameters: BTreeMap<String, serde_json::Value>,
    ) -> anyhow::Result<serde_json::Value, Error> {
        let me = self.api_client.me().await.map_err(Error::Other)?;
        let user_id: Uuid = me
            .id
            .parse()
            .map_err(|_| Error::InvalidInput(anyhow::anyhow!("invalid user id")))?;

        let context = self.replace_context_to_secrets(user_id, parameters).await?;

        let mut runtime = ScriptRuntime::default();
        runtime.install_plugin(BotCastApiPlugin::new(self.api_client.clone()));

        let res = runtime
            .run(template, context)
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
