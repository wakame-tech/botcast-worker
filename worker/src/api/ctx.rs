use crate::worker::{r2_client::R2Client, voicevox_client::VoiceVox};
use sqlx::Pool;

#[derive(Debug, Clone)]
pub struct Ctx {
    pub(crate) pool: Pool<sqlx::Postgres>,
    pub(crate) voicevox: VoiceVox,
    pub(crate) r2_client: R2Client,
}

impl Ctx {
    pub async fn new() -> anyhow::Result<Self> {
        let voicevox_endpoint =
            std::env::var("VOICEVOX_ENDPOINT").unwrap_or("http://localhost:50021".to_string());

        log::info!("VoiceVox endpoint: {}", voicevox_endpoint);

        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is required");
        let pool = sqlx::PgPool::connect(&database_url).await?;

        let ctx = Ctx {
            pool: pool.clone(),
            voicevox: VoiceVox::default(),
            r2_client: R2Client::new()?,
        };
        Ok(ctx)
    }
}
