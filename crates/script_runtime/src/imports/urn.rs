use crate::{parse_urn, Urn};
use anyhow::Result;
use json_e::{
    value::{AsyncCallable, Value},
    Context,
};
use repos::{comment_repo, episode_repo, script_repo};
use uuid::Uuid;

#[derive(Clone)]
pub(crate) struct UrnGet;

#[async_trait::async_trait]
impl AsyncCallable for UrnGet {
    async fn call(&self, context: &Context<'_>, args: &[Value]) -> Result<Value> {
        match args {
            [Value::String(urn)] => {
                let urn = Urn(urn.clone());
                let (resource, id) = parse_urn(&urn)?;

                let value = match resource.as_str() {
                    "episode" => {
                        let episode_repo = episode_repo();
                        let id: Uuid = id.parse()?;
                        let Some((episode, comments)) = episode_repo.find_by_id(&id).await? else {
                            return Err(anyhow::anyhow!("ResourceId:{} Not found", id));
                        };
                        let mut episode = serde_json::to_value(episode)?;
                        episode
                            .as_object_mut()
                            .unwrap()
                            .insert("comments".to_string(), serde_json::to_value(comments)?);
                        Ok(episode)
                    }
                    "comment" => {
                        let comment_repo = comment_repo();
                        let id = id.parse()?;
                        let Some(res) = comment_repo.find_by_id(&id).await? else {
                            return Err(anyhow::anyhow!("ResourceId:{} Not found", id));
                        };
                        serde_json::to_value(res)
                    }
                    "script" => {
                        let script_repo = script_repo();
                        let id = id.parse()?;
                        let Some(res) = script_repo.find_by_id(&id).await? else {
                            return Err(anyhow::anyhow!("Resource:{} Not found", id));
                        };
                        let template = serde_json::to_value(res)?;
                        let evaluated = json_e::render_with_context(&template, context).await?;
                        Ok(evaluated.into())
                    }
                    resource => return Err(anyhow::anyhow!("Resource:{} Not found", resource)),
                }?;
                Ok(value.into())
            }
            _ => Err(anyhow::anyhow!("invalid args".to_string())),
        }
    }
}
