mod constants;
mod error;
mod export;
mod handler;
mod mdx_options;
mod page;
mod post;
mod redirects;
mod store;
mod template;

use std::{path::Path, sync::Arc};

use axum::{Router, routing::get};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

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

/// Build every page, then either export the static site or start the local server.
///
/// - `cargo run`          → serve on `0.0.0.0:3084` for local development.
/// - `cargo run -- export` → write the static site to `dist/` for Cloudflare.
///
/// Both modes share one [`store::PageStore`], so the export matches what the
/// server returns. The program refuses to run if any page fails validation.
///
/// # Errors
///
/// Returns an error if a page fails to build, the argument is unknown, the
/// export cannot be written, or the server fails to bind or serve.
fn main() -> anyhow::Result<()> {
    // Initialise structured logging. Level is controlled by the RUST_LOG env var
    // (e.g. RUST_LOG=info or RUST_LOG=blog=debug,tower_http=debug).
    // Falls back to INFO for this crate and WARN for everything else if unset.
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("blog=info,tower_http=info,warn")),
        )
        .init();

    let store = store::PageStore::build().map_err(|e| anyhow::anyhow!("startup failed: {e}"))?;

    // `nth(1)` skips the program name, like `sys.argv[1]` in Python.
    match std::env::args().nth(1).as_deref() {
        None => serve(store),
        Some("export") => {
            let files = export::site_files(&store);
            export::write_site(Path::new(export::EXPORT_DIR), &files)?;
            info!(
                files = files.len(),
                dir = export::EXPORT_DIR,
                "exported static site"
            );
            Ok(())
        }
        Some(other) => anyhow::bail!("unknown command {other:?}; expected none or `export`"),
    }
}

/// Serve the pre-rendered site with axum on `0.0.0.0:3084`.
///
/// Requests are served entirely from the in-memory store; no file I/O occurs
/// at request time. `#[tokio::main]` builds the async runtime only for this
/// mode, so `export` never starts one.
///
/// Routes:
/// - `GET /`        → pre-rendered `pages/index.mdx`
/// - `GET /blog`    → pre-rendered listing of all published blog posts
/// - `GET /search.json` → JSON search index, fetched lazily by the search dialog
/// - `GET /{*path}` → pre-rendered page matching the given URL key
///
/// # Arguments
///
/// * `store` - The fully built page store
///
/// # Errors
///
/// Returns an error if the listener cannot bind or the server fails.
#[tokio::main]
async fn serve(store: store::PageStore) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/", get(handler::serve_index))
        .route("/blog", get(handler::serve_blog_index))
        .route("/search.json", get(handler::serve_search_index))
        .route("/{*path}", get(handler::serve_page))
        .with_state(Arc::new(store))
        .layer(TraceLayer::new_for_http());

    let listener = TcpListener::bind("0.0.0.0:3084").await?;
    info!(addr = %listener.local_addr()?, "listening");
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
