use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use ctx::Ctx;
use episode::list_episodes;
use task::{create_task, list_task};

pub(crate) mod ctx;
mod episode;
mod task;

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

pub(crate) fn create_router(router: Router<Ctx>) -> Router<Ctx> {
    router
        .route(
            "/version/voicevox",
            get(|State(ctx): State<Ctx>| async move {
                let version = ctx.voicevox.version().await?;
                Result::<_, AppError>::Ok(version)
            }),
        )
        .route("/version", get(|| async { env!("CARGO_PKG_VERSION") }))
        .route("/", get(list_task))
        .route("/scripts", post(create_task))
        .route("/episodes", get(list_episodes))
}
