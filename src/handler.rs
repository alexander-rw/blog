use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect, Response},
};
use maud::{Markup, PreEscaped};

use crate::{error::AppError, redirects, store::PageStore};

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

/// Serve a page or permanent redirect at `/{*path}`.
///
/// Resolution order:
/// 1. Pre-rendered page in the store → `200 OK` with HTML.
/// 2. Entry in the static redirect table → `308 Permanent Redirect`.
/// 3. Neither → styled `404` page via [`AppError::NotFound`].
///
/// # Arguments
///
/// * `path` - URL path segment(s), extracted by axum's wildcard router
///
/// # Errors
///
/// Returns [`AppError::NotFound`] if the path matches neither a compiled page
/// nor a configured redirect.
pub async fn serve_page(
    State(store): State<Arc<PageStore>>,
    Path(path): Path<String>,
) -> Result<Response, AppError> {
    if let Some(html) = store.page(&path) {
        return Ok(PreEscaped(html.to_owned()).into_response());
    }
    if let Some(target) = redirects::lookup(&path) {
        return Ok(Redirect::permanent(target).into_response());
    }
    Err(AppError::NotFound(path))
}
