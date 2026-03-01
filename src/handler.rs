use worker::{Response, Result};

use crate::store::PageStore;

/// Serve the index page at `/`.
///
/// # Returns
///
/// The pre-rendered HTML page for `pages/index.mdx`.
///
/// # Errors
///
/// Returns a worker error if the index page was not compiled into the store.
pub fn serve_index(store: &PageStore) -> Result<Response> {
    let html = store
        .page("index")
        .ok_or_else(|| worker::Error::RustError("index page not found".to_owned()))?;
    Response::from_html(html)
}

/// Serve the blog post listing at `/blog`.
///
/// # Returns
///
/// The pre-rendered HTML listing of all published posts, sorted by date descending.
///
/// # Errors
///
/// Returns a worker error if the response cannot be constructed.
pub fn serve_blog_index(store: &PageStore) -> Result<Response> {
    Response::from_html(store.blog_listing())
}

/// Serve a page at the given path.
///
/// # Arguments
///
/// * `store` - The pre-rendered page store
/// * `path`  - URL path segment(s) extracted from the request
///
/// # Returns
///
/// The pre-rendered HTML page for the given path.
///
/// # Errors
///
/// Returns a worker error if no page was compiled for the path.
pub fn serve_page(store: &PageStore, path: &str) -> Result<Response> {
    let html = store
        .page(path)
        .ok_or_else(|| worker::Error::RustError(format!("page not found: {path}")))?;
    Response::from_html(html)
}
