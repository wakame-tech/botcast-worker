use crate::usecase::{script_service::ScriptService, task_service::TaskService};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use repos::{entity::ScriptId, provider::Provider};
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
struct AppState(Provider);

#[derive(Debug)]
struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", self.0)).into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

async fn update_script(
    State(state): State<Arc<AppState>>,
    Path(script_id): Path<Uuid>,
    Json(template): Json<Value>,
) -> Result<impl IntoResponse, AppError> {
    ScriptService::new(state.0)
        .update_script(&ScriptId(script_id), template)
        .await?;
    Ok(StatusCode::CREATED)
}

async fn eval_script(
    State(state): State<Arc<AppState>>,
    Json(template): Json<Value>,
) -> Result<impl IntoResponse, AppError> {
    let evaluated = ScriptService::new(state.0).evaluate_once(&template).await?;
    Ok(Json(evaluated))
}

async fn insert_task(
    State(state): State<Arc<AppState>>,
    Json(args): Json<Value>,
) -> Result<impl IntoResponse, AppError> {
    TaskService::new(state.0).insert_task(args).await?;
    Ok(StatusCode::CREATED)
}

async fn version() -> Result<impl IntoResponse, AppError> {
    let worker_version = env!("CARGO_PKG_VERSION");
    Ok(Json(json!({
        "worker": worker_version,
    })))
}

pub async fn start_api(provider: Provider) -> anyhow::Result<()> {
    let state = Arc::new(AppState(provider));
    let router = Router::new()
        .route("/version", get(version))
        .route("/evalScript", post(eval_script))
        .route("/updateEpisodeScript/:episode_id", post(update_script))
        .route("/insertTask", post(insert_task))
        .with_state(state);
    let port = std::env::var("PORT").unwrap_or("9001".to_string());
    log::info!("Listen port: {}", port);
    let listener = tokio::net::TcpListener::bind(&format!("0.0.0.0:{}", port)).await?;
    axum::serve(listener, router).await?;
    Ok(())
}
