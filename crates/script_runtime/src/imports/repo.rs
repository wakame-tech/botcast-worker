use super::as_string;
use crate::runtime::{evaluate_args, ScriptRuntime};
use anyhow::Result;
use json_e::{
    value::{AsyncCallable, Value},
    Context,
};
use repos::{
    entity::{CommentId, EpisodeId, PodcastId, ScriptId},
    repo::{CommentRepo, EpisodeRepo, PodcastRepo, ScriptRepo},
};
use std::sync::Arc;
use tracing::instrument;

#[derive(Clone)]
pub struct GetPodcast {
    podcast_repo: Arc<dyn PodcastRepo>,
    episode_repo: Arc<dyn EpisodeRepo>,
}

#[async_trait::async_trait]
impl AsyncCallable for GetPodcast {
    #[instrument(skip(self, ctx))]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let id = PodcastId(as_string(&evaluated[0])?.parse()?);
        let podcast = self.podcast_repo.find_by_id(&id).await?;
        let mut podcast = serde_json::to_value(podcast)?;
        let episodes = self.episode_repo.find_all_by_podcast_id(&id).await?;
        podcast["episodes"] = serde_json::to_value(episodes)?;
        Ok(podcast.into())
    }
}

#[derive(Clone)]
pub struct GetEpisode {
    episode_repo: Arc<dyn EpisodeRepo>,
}

#[async_trait::async_trait]
impl AsyncCallable for GetEpisode {
    #[instrument(skip(self, ctx))]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let id = EpisodeId(as_string(&evaluated[0])?.parse()?);
        let (episode, comments) = self.episode_repo.find_by_id(&id).await?;
        let mut episode = serde_json::to_value(episode)?;
        episode["comments"] = serde_json::to_value(comments)?;
        Ok(episode.into())
    }
}

#[derive(Clone)]
pub struct GetComment {
    comment_repo: Arc<dyn CommentRepo>,
}

#[async_trait::async_trait]
impl AsyncCallable for GetComment {
    #[instrument(skip(self, ctx))]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let id = CommentId(as_string(&evaluated[0])?.parse()?);
        let res = self.comment_repo.find_by_id(&id).await?;
        let res = serde_json::to_value(res)?;
        Ok(res.into())
    }
}

#[derive(Clone)]
pub struct GetScript {
    script_repo: Arc<dyn ScriptRepo>,
}

#[async_trait::async_trait]
impl AsyncCallable for GetScript {
    #[instrument(skip(self, ctx))]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let id = ScriptId(as_string(&evaluated[0])?.parse()?);
        let res = self.script_repo.find_by_id(&id).await?;
        let res = serde_json::to_value(res)?;
        Ok(res.into())
    }
}

pub fn register_repo_functions(
    runtime: &mut ScriptRuntime,
    podcast_repo: Arc<dyn PodcastRepo>,
    episode_repo: Arc<dyn EpisodeRepo>,
    comment_repo: Arc<dyn CommentRepo>,
    script_repo: Arc<dyn ScriptRepo>,
) {
    runtime.register_function(
        "get_podcast",
        Box::new(GetPodcast {
            podcast_repo,
            episode_repo: episode_repo.clone(),
        }),
    );
    runtime.register_function("get_episode", Box::new(GetEpisode { episode_repo }));
    runtime.register_function("get_comment", Box::new(GetComment { comment_repo }));
    runtime.register_function("get_script", Box::new(GetScript { script_repo }));
}
