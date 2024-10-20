use chrono::{DateTime, Utc};
use uuid::Uuid;

// NOTE: #[derive(sqlx::Type)] + #[sqlx(transparent)] cannot useable in query_as! macro
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct PodcastId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct EpisodeId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct CommentId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct ScriptId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct TaskId(pub Uuid);

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct Podcast {
    pub id: Uuid,
    pub title: String,
    pub icon: String,
    pub script_id: Uuid,
    pub cron: Option<String>,
    pub user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

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

#[derive(Debug, sqlx::Type, serde::Serialize, serde::Deserialize)]
#[sqlx(type_name = "task_status", rename_all = "UPPERCASE")]
pub enum TaskStatus {
    None,
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct Task {
    pub id: Uuid,
    pub status: TaskStatus,
    pub args: serde_json::Value,
    pub execute_after: DateTime<Utc>,
    pub executed_at: Option<DateTime<Utc>>,
}
