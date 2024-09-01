use super::{ctx::Ctx, AppError};
use crate::{model::Task, repo::TaskRepo, worker::Args};
use axum::{extract::State, Json};
use reqwest::StatusCode;
use serde_json::Value;

pub(crate) async fn list_task(State(ctx): State<Ctx>) -> Result<Json<Vec<Task>>, AppError> {
    let repo = TaskRepo::new(ctx.pool);
    let tasks = repo.list().await?;
    Ok(Json(tasks))
}

pub(crate) async fn create_task(
    State(ctx): State<Ctx>,
    Json(body): Json<Value>,
) -> Result<StatusCode, AppError> {
    let repo = TaskRepo::new(ctx.pool);
    let args: Args = serde_json::from_value(body)?;
    let task = Task::new(args)?;
    repo.create(task).await?;
    Ok(StatusCode::CREATED)
}
