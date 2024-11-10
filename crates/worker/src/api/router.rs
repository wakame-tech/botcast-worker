use super::AppState;
use crate::{error::Error, model::Args};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use repos::entity::{PodcastId, ScriptId};
use serde_json::{json, Value};
use std::{collections::BTreeMap, sync::Arc};
use uuid::Uuid;

async fn update_script(
    State(state): State<Arc<AppState>>,
    Path(script_id): Path<Uuid>,
    Json(template): Json<Value>,
) -> Result<impl IntoResponse, Error> {
    state
        .0
        .script_service()
        .update_template(&ScriptId(script_id), template)
        .await?;
    Ok(StatusCode::CREATED)
}

async fn run_podcast_template(
    State(state): State<Arc<AppState>>,
    Path(podcast_id): Path<Uuid>,
) -> Result<impl IntoResponse, Error> {
    let manuscript = state
        .0
        .episode_service()
        .generate_manuscript(&PodcastId(podcast_id))
        .await?;
    Ok(Json(manuscript))
}

#[derive(Debug, serde::Deserialize)]
struct EvalTemplateRequest {
    template: Value,
    context: BTreeMap<String, Value>,
}

async fn eval_template(
    State(state): State<Arc<AppState>>,
    Json(EvalTemplateRequest { template, context }): Json<EvalTemplateRequest>,
) -> Result<impl IntoResponse, Error> {
    let evaluated = state
        .0
        .script_service()
        .evaluate_template(&template, context)
        .await?;
    Ok(Json(evaluated))
}

async fn create_task(
    State(state): State<Arc<AppState>>,
    Json(args): Json<Args>,
) -> Result<impl IntoResponse, Error> {
    state.0.task_service().create_task(args).await?;
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
        .route(
            "/podcasts/:podcast_id/runTemplate",
            post(run_podcast_template),
        )
        .route("/scripts/:script_id", post(update_script))
        .route("/createTask", post(create_task))
        .route("/evalTemplate", post(eval_template))
}
