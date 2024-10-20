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
use std::sync::Arc;
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

async fn eval_script(
    State(state): State<Arc<AppState>>,
    Json(template): Json<Value>,
) -> Result<impl IntoResponse, Error> {
    let evaluated = ScriptService::new(*state.0)
        .evaluate_once(&template)
        .await?;
    Ok(Json(evaluated))
}

async fn insert_task(
    State(state): State<Arc<AppState>>,
    Json(args): Json<Value>,
) -> Result<impl IntoResponse, Error> {
    let args: Args = serde_json::from_value(args).map_err(|e| Error::InvalidInput(e.into()))?;
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
        .route("/evalScript", post(eval_script))
        .route("/updateEpisodeScript/:episode_id", post(update_script))
        .route("/insertTask", post(insert_task))
}
