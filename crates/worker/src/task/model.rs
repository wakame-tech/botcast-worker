use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, sqlx::Type, serde::Serialize, serde::Deserialize)]
#[sqlx(type_name = "task_status", rename_all = "UPPERCASE")]
pub(crate) enum TaskStatus {
    None,
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub(crate) struct Task {
    pub id: Uuid,
    pub status: TaskStatus,
    pub args: serde_json::Value,
    pub execute_after: DateTime<Utc>,
    pub executed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub(crate) enum Args {
    GenerateAudio { episode_id: Uuid },
    EvaluateScript { script_id: Uuid },
    NewEpisode { pre_episode_id: Uuid },
}
