use super::{ctx::Ctx, AppError};
use crate::worker::{
    scrape::ScrapeEpisode,
    task::{Task, TaskRepo},
};
use axum::{extract::State, Json};
use reqwest::StatusCode;

#[derive(Debug, serde::Deserialize)]
pub(crate) struct CreateTaskBody {
    url: String,
}

pub(crate) async fn list_task(State(ctx): State<Ctx>) -> Result<Json<Vec<Task>>, AppError> {
    let repo = ctx.task_repo();
    let tasks = repo.list().await?;
    Ok(Json(tasks))
}

pub(crate) async fn create_task(
    State(ctx): State<Ctx>,
    Json(body): Json<CreateTaskBody>,
) -> Result<StatusCode, AppError> {
    let repo = ctx.task_repo();
    let task_id = TaskRepo::new_id();
    let scrape = ScrapeEpisode::new(task_id, body.url);
    repo.create(Task::Scrape(scrape)).await?;
    Ok(StatusCode::CREATED)
}
