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
pub struct ScriptId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct CornerId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct MailId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct TaskId(pub Uuid);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Podcast {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub icon: String,
    pub user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Episode {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub audio_url: Option<String>,
    pub duration_sec: Option<i32>,
    pub sections: serde_json::Value,
    pub srt_url: Option<String>,
    pub podcast_id: Uuid,
    pub user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Script {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub arguments: serde_json::Value,
    pub template: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Corner {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub requesting_mail: bool,
    pub user_id: Uuid,
    pub mail_schema: serde_json::Value,
    pub podcast_id: Uuid,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Mail {
    pub id: Uuid,
    pub body: serde_json::Value,
    pub user_id: Uuid,
    pub corner_id: Uuid,
    pub created_at: DateTime<Utc>,
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
    pub user_id: Option<Uuid>,
    pub status: TaskStatus,
    pub cron: Option<String>,
    pub args: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub execute_after: DateTime<Utc>,
    pub executed_at: Option<DateTime<Utc>>,
    pub executed_finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct Secret {
    pub name: Option<String>,
    pub decrypted_secret: Option<String>,
}
