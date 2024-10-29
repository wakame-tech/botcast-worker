use crate::runtime::{display_fn_io, evaluate_args, insert_values};
use anyhow::Result;
use json_e::{
    value::{AsyncCallable, Value},
    Context,
};
use repos::{
    repo::{CommentRepo, EpisodeRepo, PodcastRepo, ScriptRepo},
    urn::Urn,
};
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct Eval;

#[async_trait::async_trait]
impl AsyncCallable for Eval {
    async fn call(&self, context: &Context<'_>, args: &[Value]) -> Result<Value> {
        let (template, values) = match args {
            [template, Value::Object(values)] => (template, values),
            _ => return Err(anyhow::anyhow!("invalid args".to_string())),
        };

        let mut context = context.child();
        insert_values(&mut context, values.clone());
        let template = template.try_into()?;
        let ret = json_e::render_with_context(&template, &context)
            .await
            .map(Value::from)
            .map_err(|e| anyhow::anyhow!("eval error: {}", e))
            .and_then(|v| v.try_into());
        log::info!("{}", display_fn_io("eval", args, &ret)?);
        Ok(ret?.into())
    }
}

#[derive(Clone)]
pub struct UrnGet {
    podcast_repo: Arc<dyn PodcastRepo>,
    episode_repo: Arc<dyn EpisodeRepo>,
    comment_repo: Arc<dyn CommentRepo>,
    script_repo: Arc<dyn ScriptRepo>,
}

impl UrnGet {
    pub fn new(
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

    async fn resolve_urn(&self, urn: Urn) -> Result<serde_json::Value> {
        let value = match urn {
            Urn::Podcast(id) => {
                let podcast = self.podcast_repo.find_by_id(&id).await?;
                let mut podcast = serde_json::to_value(podcast)?;
                let episodes = self.episode_repo.find_all_by_podcast_id(&id).await?;
                podcast["episodes"] = serde_json::to_value(episodes)?;
                podcast
            }
            Urn::Episode(id) => {
                let (episode, comments) = self.episode_repo.find_by_id(&id).await?;
                let mut episode = serde_json::to_value(episode)?;
                episode["comments"] = serde_json::to_value(comments)?;
                episode
            }
            Urn::Comment(id) => {
                let res = self.comment_repo.find_by_id(&id).await?;
                serde_json::to_value(res)?
            }
            Urn::Script(id) => {
                let res = self.script_repo.find_by_id(&id).await?;
                serde_json::to_value(res)?
            }
            Urn::Other(resource, id) => {
                return Err(repos::error::Error::NotFound(resource, id).into())
            }
        };
        Ok(value)
    }
}

#[async_trait::async_trait]
impl AsyncCallable for UrnGet {
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let urn = match evaluated.as_slice() {
            [serde_json::Value::String(urn)] => urn.parse::<Urn>(),
            _ => return Err(anyhow::anyhow!("invalid args".to_string())),
        }?;
        let value = self.resolve_urn(urn).await;
        log::info!("{}", display_fn_io("get", args, &value)?);
        Ok(value?.into())
    }
}
