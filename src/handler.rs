use std::sync::Arc;

use axum::extract::{Path, State};
use maud::{Markup, PreEscaped};

use crate::{error::AppError, store::PageStore};

/// Serve the index page at `/`.
///
/// # Returns
///
/// The pre-rendered HTML page for `pages/index.mdx`.
///
/// # Errors
///
/// Returns [`AppError::NotFound`] if the index page was not compiled into the store.
pub async fn serve_index(State(store): State<Arc<PageStore>>) -> Result<Markup, AppError> {
    let html = store
        .page("index")
        .ok_or_else(|| AppError::NotFound("index".to_owned()))?;
    Ok(PreEscaped(html.to_owned()))
}

/// Serve the blog post listing at `/blog`.
///
/// # Returns
///
/// The pre-rendered HTML listing of all published posts, sorted by date descending.
pub async fn serve_blog_index(State(store): State<Arc<PageStore>>) -> Markup {
    PreEscaped(store.blog_listing().to_owned())
}

/// Serve a page at `/{*path}`.
///
/// # Arguments
///
/// * `path` - URL path segment(s), extracted by axum's wildcard router
///
/// # Returns
///
/// The pre-rendered HTML page for the given path.
///
/// # Errors
///
/// Returns [`AppError::NotFound`] if no page was compiled for the path.
pub async fn serve_page(
    State(store): State<Arc<PageStore>>,
    Path(path): Path<String>,
) -> Result<Markup, AppError> {
    let html = store.page(&path).ok_or(AppError::NotFound(path))?;
    Ok(PreEscaped(html.to_owned()))
}
