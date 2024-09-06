use crate::tasks::{
    episode_repo::DummyEpisodeRepo, run, storage::DummyStorage, voicevox_client::VoiceVox,
};
use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use uuid::Uuid;

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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct Args {
    pub(crate) episode_id: String,
    pub(crate) url: String,
}

async fn run_task(Json(body): Json<Value>) -> Result<impl IntoResponse, AppError> {
    let args: Args = serde_json::from_value(body)?;
    let task_id = Uuid::new_v4();
    let episode_repo = DummyEpisodeRepo;
    let storage = DummyStorage;
    run(&episode_repo, &storage, task_id, &args).await?;
    Ok("")
}

async fn version() -> Result<impl IntoResponse, AppError> {
    let worker_version = env!("CARGO_PKG_VERSION");
    let voicevox_endpoint =
        std::env::var("VOICEVOX_ENDPOINT").unwrap_or("http://localhost:50021".to_string());
    log::info!("VoiceVox endpoint: {}", voicevox_endpoint);
    let voicevox = VoiceVox::new(voicevox_endpoint);
    let voicevox_version = voicevox.version().await.map_err(AppError)?;
    Ok(Json(json!({
        "worker": worker_version,
        "voicevox": voicevox_version,
    })))
}

pub fn create_router(router: Router) -> Router {
    router
        .route("/version", get(version))
        .route("/run", post(run_task))
}
