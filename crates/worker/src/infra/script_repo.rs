use crate::episode::{Script, ScriptRepo};
use axum::async_trait;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

pub(crate) struct PostgresScriptRepo {
    pub(crate) pool: Pool<Postgres>,
}

impl PostgresScriptRepo {
    pub(crate) fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ScriptRepo for PostgresScriptRepo {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Script>> {
        let script = sqlx::query_as!(Script, "select * from scripts where id = $1", id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(script)
    }

    async fn update(&self, script: &Script) -> anyhow::Result<()> {
        sqlx::query_as!(
            Episode,
            "update scripts set template = $2 where id = $1",
            script.id,
            script.template,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

pub(crate) struct DummyScriptRepo {
    pub template: serde_json::Value,
}

#[async_trait]
impl ScriptRepo for DummyScriptRepo {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<Script>> {
        let script = Script {
            id: *id,
            user_id: Uuid::new_v4(),
            template: self.template.clone(),
        };
        Ok(Some(script))
    }

    async fn update(&self, script: &Script) -> anyhow::Result<()> {
        log::info!("{:?}", script);
        Ok(())
    }
}
