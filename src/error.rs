use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

/// Application-level errors returned by route handlers.
#[derive(Debug, Error)]
pub enum AppError {
    /// The requested MDX page file could not be found.
    ///
    /// # Arguments
    ///
    /// * `0` - The path that was not found
    #[error("page not found: {0}")]
    NotFound(String),

    /// The MDX content could not be parsed.
    ///
    /// # Arguments
    ///
    /// * `0` - The parse error message
    #[error("parse error: {0}")]
    ParseError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match &self {
            AppError::NotFound(path) => (StatusCode::NOT_FOUND, format!("404 Not Found: {path}")),
            AppError::ParseError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("500 Internal Server Error: {msg}"),
            ),
        };
        (status, body).into_response()
    }
}
