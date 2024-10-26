use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0} {1} not found")]
    NotFound(String, String),
    #[error("Other: {0}")]
    Other(sqlx::Error),
}
