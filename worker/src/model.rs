use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub(crate) struct Episode {
    pub id: Uuid,
    pub title: String,
    pub content: Option<String>,
    pub audio_url: Option<String>,
}

impl Episode {
    pub(crate) fn new(title: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            content: None,
            audio_url: None,
        }
    }
}

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
}

impl Task {
    pub(crate) fn new<T: Serialize>(args: T) -> anyhow::Result<Self> {
        Ok(Self {
            id: Uuid::new_v4(),
            status: TaskStatus::Pending,
            args: serde_json::to_value(args)?,
        })
    }
}
