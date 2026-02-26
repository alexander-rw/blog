mod constants;
mod error;
mod handler;
mod mdx_options;
mod page;
mod post;
mod store;
mod template;

use std::sync::Arc;

use axum::{Router, routing::get};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

/// Parse MDX content into an AST, validating that frontmatter is present at the start.
///
/// # Arguments
///
/// * `content` - Raw MDX file contents as a string slice
///
/// # Returns
///
/// The parsed [`markdown::mdast::Node`] AST on success.
///
/// # Errors
///
/// Returns a `String` error message if parsing fails or frontmatter is absent.
pub(crate) fn parse_mdx(content: &str) -> Result<markdown::mdast::Node, String> {
    let opts = mdx_options::default_mdx_compile_options();
    let ast = markdown::to_mdast(content, &opts.parse).map_err(|e| e.to_string())?;

    // Frontmatter must be the *first* child — the parser only emits `Node::Yaml`
    // when `---` appears at byte 0, so checking `.first()` is both necessary
    // and sufficient to confirm it starts the file.
    let starts_with_frontmatter = ast
        .children()
        .and_then(|c| c.first())
        .is_some_and(|n| matches!(n, markdown::mdast::Node::Yaml(_)));

    if !starts_with_frontmatter {
        return Err("frontmatter must be at the start of the file".to_string());
    }

    Ok(ast)
}

/// Start the axum HTTP server on `0.0.0.0:3084`.
///
/// All MDX pages are parsed and rendered at startup; the server refuses to start
/// if any page fails validation. Subsequent requests are served entirely from
/// the in-memory store — no file I/O occurs at request time.
///
/// Routes:
/// - `GET /`        → pre-rendered `pages/index.mdx`
/// - `GET /blog`    → pre-rendered listing of all published blog posts
/// - `GET /{*path}` → pre-rendered page matching the given URL key
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let store =
        Arc::new(store::PageStore::build().map_err(|e| anyhow::anyhow!("startup failed: {e}"))?);

    let app = Router::new()
        .route("/", get(handler::serve_index))
        .route("/blog", get(handler::serve_blog_index))
        .route("/{*path}", get(handler::serve_page))
        .with_state(store)
        .layer(TraceLayer::new_for_http());

    let listener = TcpListener::bind("0.0.0.0:3084").await?;
    println!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_on_missing_frontmatter() {
        assert!(parse_mdx("# No frontmatter here\n").is_err());
    }

    #[test]
    fn errors_on_invalid_frontmatter() {
        assert!(
            parse_mdx("--\ntitle: Invalid Frontmatter\n---\n\n# No frontmatter here\n").is_err()
        );
    }

    #[test]
    fn parses_valid_frontmatter() {
        assert!(parse_mdx("---\ntitle: About Me\n---\n\n# Introduction\n").is_ok());
    }
}
