use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum Error {
    #[error("Repo: {0}")]
    Repo(repos::error::Error),
    #[error("Script: {0}")]
    Script(anyhow::Error),
    #[error("InvalidInput: {0}")]
    InvalidInput(anyhow::Error),
    #[error("UnAuthorized")]
    UnAuthorized,
    #[error("Other: {0}")]
    Other(anyhow::Error),
}

impl From<repos::error::Error> for Error {
    fn from(e: repos::error::Error) -> Self {
        Error::Repo(e)
    }
}
