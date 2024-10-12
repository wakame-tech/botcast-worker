use crate::app_module::AppModule;
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

async fn eval_script(Json(template): Json<Value>) -> Result<impl IntoResponse, AppError> {
    let app_module = AppModule::new().await;
    let manuscript = app_module
        .script_service
        .evaluate_to_manuscript(template)
        .await?;
    Ok(Json(serde_json::to_value(manuscript)?))
}

async fn version() -> Result<impl IntoResponse, AppError> {
    let worker_version = env!("CARGO_PKG_VERSION");
    Ok(Json(json!({
        "worker": worker_version,
    })))
}

fn create_router(router: Router<AppModule>) -> Router<AppModule> {
    router
        .route("/version", get(version))
        .route("/evalScript", post(eval_script))
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
