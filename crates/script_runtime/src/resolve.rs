use crate::provider::DefaultProvider;
use anyhow::Result;
use repos::{
    error::Error,
    provider::{ProvideCommentRepo, ProvideEpisodeRepo, ProvidePodcastRepo, ProvideScriptRepo},
    urn::Urn,
};

pub(crate) async fn resolve_urn(provider: DefaultProvider, urn: Urn) -> Result<serde_json::Value> {
    let value = match urn {
        Urn::Podcast(id) => {
            let podcast_repo = provider.podcast_repo();
            let podcast = podcast_repo.find_by_id(&id).await?;
            serde_json::to_value(podcast)?
        }
        Urn::Episode(id) => {
            let episode_repo = provider.episode_repo();
            let (episode, comments) = episode_repo.find_by_id(&id).await?;
            let mut episode = serde_json::to_value(episode)?;
            episode
                .as_object_mut()
                .unwrap()
                .insert("comments".to_string(), serde_json::to_value(comments)?);
            episode
        }
        Urn::Comment(id) => {
            let comment_repo = provider.comment_repo();
            let res = comment_repo.find_by_id(&id).await?;
            serde_json::to_value(res)?
        }
        Urn::Script(id) => {
            let script_repo = provider.script_repo();
            let res = script_repo.find_by_id(&id).await?;
            serde_json::to_value(res)?
        }
        Urn::Other(resource, id) => return Err(Error::NotFound(resource, id).into()),
    };
    Ok(value)
}
