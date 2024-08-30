use super::{ctx::Ctx, AppError};
use crate::episode::Episode;
use axum::{extract::State, Json};

pub(crate) async fn list_episodes(State(ctx): State<Ctx>) -> Result<Json<Vec<Episode>>, AppError> {
    let repo = ctx.episode_repo();
    let tasks = repo.list().await?;
    Ok(Json(tasks))
}
