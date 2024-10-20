use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0} {1} not found")]
    NotFound(String, Uuid),
    #[error("Other: {0}")]
    Other(sqlx::Error),
}
