use std::fmt::Display;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub(crate) struct Episode {
    pub id: Uuid,
    pub title: String,
    pub content: Option<String>,
    pub audio_url: Option<String>,
    pub user_id: Option<Uuid>,
}

impl Display for Episode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.title)?;
        if let Some(content) = &self.content {
            for line in content.split("\n") {
                writeln!(f, "{}", line)?;
            }
        }
        Ok(())
    }
}
