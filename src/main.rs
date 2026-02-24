mod error;
mod handler;
mod mdx_options;

use axum::{Router, routing::get};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

/// Parse MDX content into an AST, validating that frontmatter is present.
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
pub fn parse_mdx(content: &str) -> Result<markdown::mdast::Node, String> {
    let opts = mdx_options::default_mdx_compile_options();
    let ast = markdown::to_mdast(content, &opts.parse).map_err(|e| e.to_string())?;

    let has_frontmatter = ast
        .children()
        .into_iter()
        .flatten()
        .any(|n| matches!(n, markdown::mdast::Node::Yaml(_)));

    if !has_frontmatter {
        return Err("missing frontmatter".to_string());
    }

    Ok(ast)
}

/// Start the axum HTTP server on `0.0.0.0:3000`.
///
/// Routes:
/// - `GET /`        → serves `pages/index.mdx` as an HTML fragment
/// - `GET /{*path}` → serves `pages/{path}.mdx` as an HTML fragment
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = Router::new()
        .route("/", get(handler::serve_index))
        .route("/{*path}", get(handler::serve_page))
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
        let broken = std::fs::read_to_string("pages/index2.mdx").unwrap();
        assert!(parse_mdx(&broken).is_err());
    }

    #[test]
    fn parses_valid_frontmatter() {
        let valid = std::fs::read_to_string("pages/index.mdx").unwrap();
        assert!(parse_mdx(&valid).is_ok());
    }
}
