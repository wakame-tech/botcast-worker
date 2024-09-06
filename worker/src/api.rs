use crate::{episode_repo::InMemoryEpisodeRepo, tasks::run};
use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::Value;
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
    let mut episode_repo = InMemoryEpisodeRepo::new();
    run(&mut episode_repo, task_id, &args).await?;
    Ok("")
}

pub fn create_router(router: Router) -> Router {
    router
        .route("/version", get(|| async { env!("CARGO_PKG_VERSION") }))
        .route("/run", post(run_task))
}
