use super::{ctx::Ctx, AppError};
use crate::{
    model::Task,
    repo::TaskRepo,
    worker::{scrape::ScrapeEpisode, Args},
};
use axum::{extract::State, Json};
use reqwest::StatusCode;

#[derive(Debug, serde::Deserialize)]
pub(crate) struct CreateTaskBody {
    url: String,
}

pub(crate) async fn list_task(State(ctx): State<Ctx>) -> Result<Json<Vec<Task>>, AppError> {
    let repo = TaskRepo::new(ctx.pool);
    let tasks = repo.list().await?;
    Ok(Json(tasks))
}

pub(crate) async fn create_task(
    State(ctx): State<Ctx>,
    Json(body): Json<CreateTaskBody>,
) -> Result<StatusCode, AppError> {
    let repo = TaskRepo::new(ctx.pool);
    let args = Args::Scrape(ScrapeEpisode::new(body.url));
    let task = Task::new(args)?;
    repo.create(task).await?;
    Ok(StatusCode::CREATED)
}
