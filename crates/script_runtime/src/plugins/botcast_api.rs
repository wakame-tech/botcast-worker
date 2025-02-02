use super::{as_string, evaluate_args, Plugin};
use anyhow::Result;
use api::{
    client::ApiClient,
    episode::{NewEpisode as NewEpisodeReq, Section, UpdateEpisode as UpdateEpisodeReq},
};
use json_e::{
    value::{AsyncCallable, Function, Value},
    Context,
};
use std::sync::Arc;
use tracing::instrument;

#[derive(Clone)]
struct Me(Arc<ApiClient>);

#[async_trait::async_trait]
impl AsyncCallable for Me {
    #[instrument(skip(self))]
    async fn call(&self, _: &Context<'_>, _: &[Value]) -> Result<Value> {
        let me = self.0.me().await?;
        let me = serde_json::to_value(me)?;
        Ok(me.into())
    }
}

#[derive(Clone)]
struct GetPodcast(Arc<ApiClient>);

#[async_trait::async_trait]
impl AsyncCallable for GetPodcast {
    #[instrument(skip(self, ctx))]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let id = as_string(&evaluated[0])?;
        let podcast = self.0.podcast(&id).await?;
        let podcast = serde_json::to_value(podcast)?;
        Ok(podcast.into())
    }
}

#[derive(Clone)]
struct GetEpisode(Arc<ApiClient>);

#[async_trait::async_trait]
impl AsyncCallable for GetEpisode {
    #[instrument(skip(self, ctx))]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let id = as_string(&evaluated[0])?;
        let episode = self.0.episode(&id).await?;
        let episode = serde_json::to_value(episode)?;
        Ok(episode.into())
    }
}

#[derive(Clone)]
struct GetScript(Arc<ApiClient>);

#[async_trait::async_trait]
impl AsyncCallable for GetScript {
    #[instrument(skip(self, ctx))]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let id = as_string(&evaluated[0])?;
        let res = self.0.script(&id).await?;
        let res = serde_json::to_value(res)?;
        Ok(res.into())
    }
}

#[derive(Clone)]
struct NewEpisode(Arc<ApiClient>);

#[async_trait::async_trait]
impl AsyncCallable for NewEpisode {
    #[instrument(skip(self, ctx))]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let podcast_id = as_string(&evaluated[0])?;
        let title = as_string(&evaluated[1])?;
        let sections: Vec<Section> = serde_json::from_value(evaluated[2].clone())?;
        let description = evaluated.get(3).map(as_string).transpose()?;
        self.0
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

#[derive(Clone)]
struct UpdateEpisode(Arc<ApiClient>);

#[async_trait::async_trait]
impl AsyncCallable for UpdateEpisode {
    #[instrument(skip(self, ctx))]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let args = evaluate_args(ctx, args).await?;
        let episode_id = as_string(&args[0])?;
        let title = as_string(&args[1])?;
        let sections: Option<Vec<Section>> =
            (!args[2].is_null()).then_some(serde_json::from_value(args[2].clone())?);
        let description = (!args[3].is_null()).then_some(as_string(&args[3])?);
        self.0
            .update_episode(UpdateEpisodeReq {
                id: episode_id,
                title,
                description,
                sections,
            })
            .await?;
        Ok(Value::Null)
    }
}

#[derive(Clone)]
struct GetPodcastMails(Arc<ApiClient>);

#[async_trait::async_trait]
impl AsyncCallable for GetPodcastMails {
    #[instrument(skip(self, ctx))]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let args = evaluate_args(ctx, args).await?;
        let corner_id = as_string(&args[0])?;
        let mails = self.0.mails(&corner_id).await?;
        Ok(serde_json::to_value(mails)?.into())
    }
}

pub struct BotCastApiPlugin {
    client: Arc<ApiClient>,
}

impl BotCastApiPlugin {
    pub fn new(client: Arc<ApiClient>) -> Self {
        Self { client }
    }
}

impl Plugin for BotCastApiPlugin {
    fn register_functions(&self, context: &mut Context<'_>) {
        let functions = vec![
            (
                "me",
                Box::new(Me(self.client.clone())) as Box<dyn AsyncCallable>,
            ),
            ("get_podcast", Box::new(GetPodcast(self.client.clone()))),
            ("get_episode", Box::new(GetEpisode(self.client.clone()))),
            ("get_script", Box::new(GetScript(self.client.clone()))),
            ("new_episode", Box::new(NewEpisode(self.client.clone()))),
            (
                "update_episode",
                Box::new(UpdateEpisode(self.client.clone())),
            ),
            (
                "get_podcast_mails",
                Box::new(GetPodcastMails(self.client.clone())),
            ),
        ];
        for (name, func) in functions {
            context.insert(name, Value::Function(Function::new(name, func)));
        }
    }
}
