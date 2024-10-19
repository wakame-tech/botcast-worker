use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct Episode {
    pub id: Uuid,
    pub title: String,
    pub audio_url: Option<String>,
    pub script_id: Uuid,
    pub srt_url: Option<String>,
    pub podcast_id: Uuid,
    pub user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct Comment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub episode_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct Script {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub template: serde_json::Value,
    pub result: Option<serde_json::Value>,
}
