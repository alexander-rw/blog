use axum::{
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use thiserror::Error;

use crate::template::render_not_found;

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
        match self {
            AppError::NotFound(_) => (
                StatusCode::NOT_FOUND,
                [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
                render_not_found().into_string(),
            )
                .into_response(),
            AppError::ParseError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("500 Internal Server Error: {msg}"),
            )
                .into_response(),
        }
    }
}
