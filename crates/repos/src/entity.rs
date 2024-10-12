use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct Episode {
    pub id: Uuid,
    pub title: String,
    pub audio_url: Option<String>,
    pub script_id: Uuid,
    pub manuscript: Option<serde_json::Value>,
    pub podcast_id: Uuid,
    pub user_id: Option<Uuid>,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct Script {
    pub id: Uuid,
    pub user_id: Uuid,
    pub template: serde_json::Value,
}
