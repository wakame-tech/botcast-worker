use crate::worker::voicevox_client::VoiceVox;
use sqlx::Pool;
use std::sync::LazyLock;
use surrealdb::{engine::local::Db, Surreal};

#[derive(Debug, Clone)]
pub(crate) struct Ctx {
    pub(crate) queue_db: Surreal<Db>,
    pub(crate) pool: Pool<sqlx::Postgres>,
    pub(crate) voicevox: VoiceVox,
}

impl Ctx {
    pub(crate) async fn new() -> anyhow::Result<Self> {
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
        Ok(ctx)
    }
}

static QUEUE_DB: LazyLock<Surreal<Db>> = LazyLock::new(|| Surreal::init());
