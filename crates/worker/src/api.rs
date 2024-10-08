use crate::{app_module::AppModule, infra::voicevox_client::VoiceVoxClient};
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
    pub(crate) episode_id: Uuid,
    pub(crate) url: String,
}

async fn run_task(Json(body): Json<Value>) -> Result<impl IntoResponse, AppError> {
    let args: Args = serde_json::from_value(body)?;
    let task_id = Uuid::new_v4();
    // episode_service
    //     .run(task_id, args.episode_id, args.url.parse()?)
    //     .await?;
    Ok("")
}

async fn version() -> Result<impl IntoResponse, AppError> {
    let worker_version = env!("CARGO_PKG_VERSION");
    let voicevox = VoiceVoxClient::new();
    let voicevox_version = voicevox.version().await.map_err(AppError)?;
    Ok(Json(json!({
        "worker": worker_version,
        "voicevox": voicevox_version,
    })))
}

fn create_router(router: Router<AppModule>) -> Router<AppModule> {
    router
        .route("/version", get(version))
        .route("/run", post(run_task))
}

pub async fn start_api(app_module: AppModule) -> anyhow::Result<()> {
    let router = Router::<AppModule>::new();
    let app = create_router(router).with_state(app_module);
    let port = std::env::var("PORT").unwrap_or("9001".to_string());
    log::info!("Listen port: {}", port);
    let listener = tokio::net::TcpListener::bind(&format!("0.0.0.0:{}", port)).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
