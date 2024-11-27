use crate::error::Error;
use repos::{
    entity::ScriptId,
    repo::{CommentRepo, EpisodeRepo, PodcastRepo, ScriptRepo},
};
use script_runtime::{
    imports::{
        llm::{create_thread, delete_thread, register_llm_functions},
        urn::UrnGet,
    },
    runtime::ScriptRuntime,
};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone)]
pub(crate) struct ScriptService {
    podcast_repo: Arc<dyn PodcastRepo>,
    episode_repo: Arc<dyn EpisodeRepo>,
    comment_repo: Arc<dyn CommentRepo>,
    script_repo: Arc<dyn ScriptRepo>,
}

impl ScriptService {
    pub(crate) fn new(
        podcast_repo: Arc<dyn PodcastRepo>,
        episode_repo: Arc<dyn EpisodeRepo>,
        comment_repo: Arc<dyn CommentRepo>,
        script_repo: Arc<dyn ScriptRepo>,
    ) -> Self {
        Self {
            podcast_repo,
            episode_repo,
            comment_repo,
            script_repo,
        }
    }

    pub(crate) async fn run_template(
        &self,
        template: &serde_json::Value,
        values: BTreeMap<String, serde_json::Value>,
    ) -> anyhow::Result<serde_json::Value, Error> {
        let mut runtime = ScriptRuntime::default();
        runtime.register_function(
            "get",
            Box::new(UrnGet::new(
                self.podcast_repo.clone(),
                self.episode_repo.clone(),
                self.comment_repo.clone(),
                self.script_repo.clone(),
            )),
        );
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
