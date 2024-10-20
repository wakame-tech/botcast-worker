use crate::Urn;
use anyhow::Result;
use json_e::{
    value::{AsyncCallable, Value},
    Context,
};
use repos::{comment_repo, episode_repo, error::Error, podcast_repo, repo::PodcastId, script_repo};
use uuid::Uuid;

#[derive(Clone)]
pub(crate) struct Eval;

#[async_trait::async_trait]
impl AsyncCallable for Eval {
    async fn call(&self, context: &Context<'_>, args: &[Value]) -> Result<Value> {
        match args {
            [template, Value::Object(values)] => {
                let mut context = context.child();
                for (k, v) in values.iter() {
                    context.insert(k, v.clone());
                }
                let template = template.try_into()?;
                let evaluated = json_e::render_with_context(&template, &context).await?;
                Ok(evaluated.into())
            }
            _ => Err(anyhow::anyhow!("invalid args".to_string())),
        }
    }
}

#[derive(Clone)]
pub(crate) struct UrnGet;

#[async_trait::async_trait]
impl AsyncCallable for UrnGet {
    async fn call(&self, _: &Context<'_>, args: &[Value]) -> Result<Value> {
        let urn = match args {
            [Value::String(urn)] => urn.parse::<Urn>(),
            _ => return Err(anyhow::anyhow!("invalid args".to_string())),
        }?;

        let value = match urn {
            Urn::Podcast(id) => {
                let podcast_repo = podcast_repo();
                let podcast = podcast_repo.find_by_id(&id).await?;
                serde_json::to_value(podcast)
            }
            Urn::Episode(id) => {
                let episode_repo = episode_repo();
                let (episode, comments) = episode_repo.find_by_id(&id).await?;
                let mut episode = serde_json::to_value(episode)?;
                episode
                    .as_object_mut()
                    .unwrap()
                    .insert("comments".to_string(), serde_json::to_value(comments)?);
                Ok(episode)
            }
            Urn::Comment(id) => {
                let comment_repo = comment_repo();
                let res = comment_repo.find_by_id(&id).await?;
                serde_json::to_value(res)
            }
            Urn::Script(id) => {
                let script_repo = script_repo();
                let res = script_repo.find_by_id(&id).await?;
                serde_json::to_value(res)
            }
            Urn::Other(resource, id) => return Err(Error::NotFound(resource, id).into()),
        }?;
        Ok(value.into())
    }
}
