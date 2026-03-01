use thiserror::Error;

/// Application-level errors returned during page store construction.
#[derive(Debug, Error)]
pub enum AppError {
    /// The MDX content could not be parsed.
    ///
    /// # Arguments
    ///
    /// * `0` - The parse error message
    #[error("parse error: {0}")]
    ParseError(String),
}
