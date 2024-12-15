use super::AppState;
use crate::{
    error::Error,
    usecase::{task_service::Args, Provider, UserApiClientProvider},
};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use repos::entity::ScriptId;
use serde_json::{json, Value};
use std::{collections::BTreeMap, sync::Arc};
use tracing::instrument;
use uuid::Uuid;

fn with_user_api_client(provider: &Provider, token: &str) -> Provider {
    Provider {
        provide_api_client: Arc::new(UserApiClientProvider::new(Some(token.to_string()))),
        ..provider.clone()
    }
}

#[instrument(skip(state))]
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

#[derive(Debug, serde::Deserialize)]
struct EvalTemplateRequest {
    template: Value,
    arguments: BTreeMap<String, Value>,
}

#[instrument(skip(state))]
async fn eval_template(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(EvalTemplateRequest {
        template,
        arguments,
    }): Json<EvalTemplateRequest>,
) -> Result<impl IntoResponse, Error> {
    let token = headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| Error::UnAuthorized)?;
    let provider = with_user_api_client(&state.0, token);

    let evaluated = provider
        .script_service()
        .run_template(&template, arguments)
        .await?;
    Ok(Json(evaluated))
}

#[instrument(skip(state))]
async fn create_task(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(args): Json<Args>,
) -> Result<impl IntoResponse, Error> {
    let token = headers
        .get("Authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| Error::UnAuthorized)?;
    let provider = with_user_api_client(&state.0, token);

    provider.task_service().create_task(args).await?;
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
        .route("/createTask", post(create_task))
        .route("/evalTemplate", post(eval_template))
}
