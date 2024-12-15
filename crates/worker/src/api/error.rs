use crate::error::Error;
use axum::response::IntoResponse;
use repos::error::Error as ReposError;
use reqwest::StatusCode;

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::Repo(ReposError::NotFound(resource, id)) => (
                StatusCode::NOT_FOUND,
                format!("{} {} not found", resource, id),
            )
                .into_response(),
            Error::InvalidInput(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
            Error::Repo(ReposError::Other(e)) => {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
            }
            Error::Script(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
            Error::UnAuthorized => {
                (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response()
            }
            Error::Other(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    }
}
