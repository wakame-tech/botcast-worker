use super::{ctx::Ctx, AppError};
use crate::{model::Episode, repo::EpisodeRepo};
use axum::{extract::State, Json};

pub(crate) async fn list_episodes(State(ctx): State<Ctx>) -> Result<Json<Vec<Episode>>, AppError> {
    let repo = EpisodeRepo::new(ctx.pool);
    let tasks = repo.list().await?;
    Ok(Json(tasks))
}
