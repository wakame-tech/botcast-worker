use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use episode::{Episode, EpisodeRepo};
use scrape::ScrapeEpisode;
use sqlx::Pool;
use std::{sync::LazyLock, time::Duration};
use surrealdb::{
    engine::local::{Db, Mem},
    Surreal,
};
use task::{Task, TaskRepo};
use voicevox_client::VoiceVox;

mod episode;
mod scrape;
mod synthesis;
mod task;
mod voicevox_client;

#[derive(Debug, Clone)]
struct Ctx {
    queue_db: Surreal<Db>,
    pool: Pool<sqlx::Postgres>,
    voicevox: voicevox_client::VoiceVox,
}

impl Ctx {
    fn task_repo(&self) -> TaskRepo {
        TaskRepo {
            db: self.queue_db.clone(),
        }
    }

    fn episode_repo(&self) -> EpisodeRepo {
        EpisodeRepo {
            pool: self.pool.clone(),
        }
    }
}

static QUEUE_DB: LazyLock<Surreal<Db>> = LazyLock::new(|| Surreal::init());

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

#[derive(Debug, serde::Deserialize)]
struct CreateTaskBody {
    url: String,
}

async fn list_task(State(ctx): State<Ctx>) -> Result<Json<Vec<Task>>, AppError> {
    let repo = ctx.task_repo();
    let tasks = repo.list().await?;
    Ok(Json(tasks))
}

async fn create_task(
    State(ctx): State<Ctx>,
    Json(body): Json<CreateTaskBody>,
) -> Result<StatusCode, AppError> {
    let repo = ctx.task_repo();
    let task_id = TaskRepo::new_id();
    let scrape = ScrapeEpisode::new(task_id, body.url);
    repo.create(Task::Scrape(scrape)).await?;
    Ok(StatusCode::CREATED)
}

async fn list_episodes(State(ctx): State<Ctx>) -> Result<Json<Vec<Episode>>, AppError> {
    let repo = ctx.episode_repo();
    let tasks = repo.list().await?;
    Ok(Json(tasks))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    QUEUE_DB.connect::<Mem>(()).await?;
    QUEUE_DB.use_ns("default").use_db("database").await?;

    let voicevox_endpoint =
        std::env::var("VOICEVOX_ENDPOINT").unwrap_or("http://localhost:50021".to_string());

    log::info!("VoiceVox endpoint: {}", voicevox_endpoint);

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is required");
    let pool = sqlx::PgPool::connect(&database_url).await?;

    let ctx = Ctx {
        queue_db: QUEUE_DB.clone(),
        pool: pool.clone(),
        voicevox: VoiceVox::default(),
    };

    let ctx_ = ctx.clone();
    tokio::spawn(async move {
        let interval = Duration::from_secs(5);
        loop {
            log::info!("Watching tasks...");
            let repo = ctx_.task_repo();
            if let Err(e) = repo.watch(&ctx_).await {
                log::error!("Error: {:?}", e);
            }
            tokio::time::sleep(interval).await;
        }
    });

    let app = Router::new()
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
        .with_state(ctx);
    let port = std::env::var("PORT").unwrap_or("9001".to_string());
    log::info!("Listen port: {}", port);

    let listener = tokio::net::TcpListener::bind(&format!("0.0.0.0:{}", port)).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
