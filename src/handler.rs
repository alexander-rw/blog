use axum::{extract::Path, response::Html};

use crate::{error::AppError, mdx_options, meta, parse_mdx, template};

/// Serve the index page at `/`, mapping to `pages/index.mdx`.
///
/// # Returns
///
/// A full HTML page for the index.
///
/// # Errors
///
/// Returns `AppError::NotFound` if `pages/index.mdx` does not exist.
/// Returns `AppError::ParseError` if the file cannot be parsed.
pub async fn serve_index() -> Result<Html<String>, AppError> {
    serve_mdx_file("pages/index.mdx").await
}

/// Serve the blog post listing at `/blog`.
///
/// Reads all `pages/blog/*/page.mdx` files, skips drafts, and renders a
/// date-sorted list of posts.
///
/// # Returns
///
/// A full HTML page listing all published posts.
///
/// # Errors
///
/// Returns `AppError::NotFound` if the `pages/blog` directory cannot be read.
/// Returns `AppError::ParseError` if any post's frontmatter is malformed.
pub async fn serve_blog_index() -> Result<Html<String>, AppError> {
    let posts = meta::list_posts().await?;
    Ok(Html(template::render_post_list(&posts)))
}

/// Serve a page at `/{*path}`.
///
/// Tries `pages/{path}/page.mdx` first (blog-post layout), then falls back
/// to `pages/{path}.mdx` (flat page layout).
///
/// # Arguments
///
/// * `path` - URL path segment(s), extracted by axum's wildcard router
///
/// # Returns
///
/// A full HTML page for the requested path.
///
/// # Errors
///
/// Returns `AppError::NotFound` if neither file exists.
/// Returns `AppError::ParseError` if MDX parsing fails.
pub async fn serve_page(Path(path): Path<String>) -> Result<Html<String>, AppError> {
    // Blog posts live at pages/{slug}/page.mdx; fall back to a flat MDX file.
    let nested = format!("pages/{path}/page.mdx");
    let flat = format!("pages/{path}.mdx");

    // `try_exists` returns Err on permission issues, which we treat as absent.
    let file_path = if tokio::fs::try_exists(&nested).await.unwrap_or(false) {
        nested
    } else {
        flat
    };

    serve_mdx_file(&file_path).await
}

/// Read an MDX file, parse it, extract metadata, and return a full HTML page.
///
/// # Arguments
///
/// * `file_path` - Path to the `.mdx` file relative to the working directory
///
/// # Errors
///
/// Returns `AppError::NotFound` if the file does not exist.
/// Returns `AppError::ParseError` if MDX parsing or metadata extraction fails.
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
    let ast = parse_mdx(&content).map_err(AppError::ParseError)?;
    let page_meta = meta::extract_meta(&ast, &content)?;
    let html = markdown::to_html_with_options(&content, &opts)
        .map_err(|e| AppError::ParseError(e.to_string()))?;

    Ok(Html(template::render_page(&page_meta, &html)))
}
