use axum::{extract::Path, response::Html};

use crate::{error::AppError, mdx_options, parse_mdx};

/// Serve the index page at `/`, mapping to `pages/index.mdx`.
///
/// # Returns
///
/// The rendered HTML fragment for the index page.
///
/// # Errors
///
/// Returns `AppError::NotFound` if `pages/index.mdx` does not exist.
/// Returns `AppError::ParseError` if the file cannot be parsed.
pub async fn serve_index() -> Result<Html<String>, AppError> {
    serve_mdx_file("pages/index.mdx").await
}

/// Serve a blog page at `/{*path}`, mapping to `pages/{path}.mdx`.
///
/// # Arguments
///
/// * `path` - The URL path segment(s), extracted by axum's wildcard router
///
/// # Returns
///
/// The rendered HTML fragment for the requested page.
///
/// # Errors
///
/// Returns `AppError::NotFound` if the corresponding `.mdx` file does not exist.
/// Returns `AppError::ParseError` if the file cannot be parsed.
pub async fn serve_page(Path(path): Path<String>) -> Result<Html<String>, AppError> {
    let file_path = format!("pages/{path}.mdx");
    serve_mdx_file(&file_path).await
}

/// Read an MDX file and return its rendered HTML.
///
/// # Arguments
///
/// * `file_path` - Path to the `.mdx` file relative to the working directory
///
/// # Errors
///
/// Returns `AppError::NotFound` if the file does not exist.
/// Returns `AppError::ParseError` if MDX parsing fails.
async fn serve_mdx_file(file_path: &str) -> Result<Html<String>, AppError> {
    // Read the file asynchronously; treat I/O errors as not-found.
    let content = tokio::fs::read_to_string(file_path)
        .await
        .map_err(|_| AppError::NotFound(file_path.to_owned()))?;

    // `markdown::Options` is not `Send`/`Sync` (holds `Box<dyn Fn(...)>`), so it
    // cannot be placed in a static or moved across threads via spawn_blocking.
    // For blog-sized MDX files, parsing is fast enough to run inline without
    // meaningfully blocking the async reactor.
    let opts = mdx_options::default_mdx_compile_options();
    parse_mdx(&content).map_err(AppError::ParseError)?;
    let html = markdown::to_html_with_options(&content, &opts)
        .map_err(|e| AppError::ParseError(e.to_string()))?;

    Ok(Html(html))
}
