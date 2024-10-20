use super::AppState;
use crate::{
    error::Error,
    model::Args,
    usecase::{script_service::ScriptService, task_service::TaskService},
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use repos::entity::ScriptId;
use serde_json::{json, Value};
use std::{collections::BTreeMap, sync::Arc};
use uuid::Uuid;

async fn update_script(
    State(state): State<Arc<AppState>>,
    Path(script_id): Path<Uuid>,
    Json(template): Json<Value>,
) -> Result<impl IntoResponse, Error> {
    ScriptService::new(*state.0)
        .update_template(&ScriptId(script_id), template)
        .await?;
    Ok(StatusCode::CREATED)
}

async fn run_script(
    State(state): State<Arc<AppState>>,
    Path(script_id): Path<Uuid>,
) -> Result<impl IntoResponse, Error> {
    let evaluated = ScriptService::new(*state.0)
        .evaluate_script(&ScriptId(script_id), BTreeMap::new())
        .await?;
    Ok(Json(evaluated))
}

async fn eval_template(
    State(state): State<Arc<AppState>>,
    Json(template): Json<Value>,
) -> Result<impl IntoResponse, Error> {
    let evaluated = ScriptService::new(*state.0)
        .evaluate_once(&template)
        .await?;
    Ok(Json(evaluated))
}

async fn create_task(
    State(state): State<Arc<AppState>>,
    Json(args): Json<Args>,
) -> Result<impl IntoResponse, Error> {
    TaskService::new(*state.0).create_task(args).await?;
    Ok(StatusCode::CREATED)
}

async fn version() -> Result<impl IntoResponse, Error> {
    let worker_version = env!("CARGO_PKG_VERSION");
    Ok(Json(json!({
        "worker": worker_version,
    })))
}

pub(crate) fn routers() -> Router<Arc<AppState>> {
    Router::new()
        .route("/version", get(version))
        .route("/scripts/:script_id", post(update_script))
        .route("/scripts/:script_id/run", post(run_script))
        .route("/createTask", post(create_task))
        .route("/evalTemplate", post(eval_template))
}
