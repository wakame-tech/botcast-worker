use super::as_string;
use crate::runtime::{evaluate_args, ScriptRuntime};
use anyhow::Result;
use api::client::ApiClient;
use json_e::{
    value::{AsyncCallable, Value},
    Context,
};
use std::sync::Arc;
use tracing::instrument;

#[derive(Clone)]
pub struct GetPodcast {
    client: Arc<ApiClient>,
}

#[async_trait::async_trait]
impl AsyncCallable for GetPodcast {
    #[instrument(skip(self, ctx))]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let id = as_string(&evaluated[0])?;
        let podcast = self.client.podcast(&id).await?;
        let podcast = serde_json::to_value(podcast)?;
        Ok(podcast.into())
    }
}

#[derive(Clone)]
pub struct GetEpisode {
    client: Arc<ApiClient>,
}

#[async_trait::async_trait]
impl AsyncCallable for GetEpisode {
    #[instrument(skip(self, ctx))]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let id = as_string(&evaluated[0])?;
        let episode = self.client.episode(&id).await?;
        let episode = serde_json::to_value(episode)?;
        Ok(episode.into())
    }
}

#[derive(Clone)]
pub struct GetComment {
    client: Arc<ApiClient>,
}

#[async_trait::async_trait]
impl AsyncCallable for GetComment {
    #[instrument(skip(self, ctx))]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let id = as_string(&evaluated[0])?;
        let res = self.client.comment(&id).await?;
        let res = serde_json::to_value(res)?;
        Ok(res.into())
    }
}

#[derive(Clone)]
pub struct GetScript {
    client: Arc<ApiClient>,
}

#[async_trait::async_trait]
impl AsyncCallable for GetScript {
    #[instrument(skip(self, ctx))]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let id = as_string(&evaluated[0])?;
        let res = self.client.script(&id).await?;
        let res = serde_json::to_value(res)?;
        Ok(res.into())
    }
}

pub fn register_repo_functions(runtime: &mut ScriptRuntime, client: Arc<ApiClient>) {
    runtime.register_function(
        "get_podcast",
        Box::new(GetPodcast {
            client: client.clone(),
        }),
    );
    runtime.register_function(
        "get_episode",
        Box::new(GetEpisode {
            client: client.clone(),
        }),
    );
    runtime.register_function(
        "get_comment",
        Box::new(GetComment {
            client: client.clone(),
        }),
    );
    runtime.register_function(
        "get_script",
        Box::new(GetScript {
            client: client.clone(),
        }),
    );
}
