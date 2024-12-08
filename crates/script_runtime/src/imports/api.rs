use super::as_string;
use crate::runtime::{evaluate_args, ScriptRuntime};
use anyhow::Result;
use api::{
    client::ApiClient,
    episode::{NewEpisode as NewEpisodeReq, Section},
};
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
pub struct NewComment {
    client: Arc<ApiClient>,
}

#[async_trait::async_trait]
impl AsyncCallable for NewComment {
    #[instrument(skip(self, ctx))]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let episode_id = as_string(&evaluated[0])?;
        let content = as_string(&evaluated[1])?;
        let res = self.client.new_comment(&episode_id, &content).await?;
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

#[derive(Clone)]
pub struct NewEpisode {
    client: Arc<ApiClient>,
}

#[async_trait::async_trait]
impl AsyncCallable for NewEpisode {
    #[instrument(skip(self, ctx))]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let podcast_id = as_string(&evaluated[0])?;
        let title = as_string(&evaluated[1])?;
        let sections: Vec<Section> = serde_json::from_value(evaluated[2].clone())?;
        let description = evaluated.get(3).map(|v| as_string(v)).transpose()?;
        self.client
            .new_episode(NewEpisodeReq {
                podcast_id,
                title,
                description,
                sections,
            })
            .await?;
        Ok(Value::Null)
    }
}

pub fn register_api_functions(runtime: &mut ScriptRuntime, client: Arc<ApiClient>) {
    let api_functions = vec![
        (
            "get_podcast",
            Box::new(GetPodcast {
                client: client.clone(),
            }) as Box<dyn AsyncCallable>,
        ),
        (
            "get_episode",
            Box::new(GetEpisode {
                client: client.clone(),
            }),
        ),
        (
            "get_comment",
            Box::new(GetComment {
                client: client.clone(),
            }),
        ),
        (
            "new_comment",
            Box::new(NewComment {
                client: client.clone(),
            }),
        ),
        (
            "get_script",
            Box::new(GetScript {
                client: client.clone(),
            }),
        ),
        (
            "new_episode",
            Box::new(NewEpisode {
                client: client.clone(),
            }),
        ),
    ];
    for (name, func) in api_functions {
        runtime.register_function(name, func);
    }
}
