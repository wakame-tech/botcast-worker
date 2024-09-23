use std::fmt::Display;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub(crate) struct Episode {
    pub id: Uuid,
    pub title: String,
    pub audio_url: Option<String>,
    pub script_url: Option<String>,
    pub podcast_id: Uuid,
    pub user_id: Option<Uuid>,
    pub created_at: String,
}

impl Display for Episode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self)?;
        Ok(())
    }
}
