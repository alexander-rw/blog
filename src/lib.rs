mod constants;
mod error;
mod handler;
mod mdx_options;
mod page;
mod post;
mod store;
mod template;

use std::sync::LazyLock;

use worker::{Context, Env, Request, Response, Router, event};

/// Global page store, built lazily on the first request in each Worker isolate.
///
/// `LazyLock` ensures the (potentially expensive) MDX parsing and rendering
/// runs exactly once per isolate lifetime, then all subsequent requests read
/// from the pre-built in-memory store without synchronisation overhead.
///
/// # Panics
///
/// Panics if any embedded MDX file fails validation or rendering. This is
/// intentional: a broken build should surface immediately, not serve partial
/// content silently.
static STORE: LazyLock<store::PageStore> = LazyLock::new(|| {
    store::PageStore::build().expect("failed to build page store from embedded MDX")
});

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

    // Frontmatter must be the *first* child -- the parser only emits `Node::Yaml`
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

/// Cloudflare Worker fetch event handler.
///
/// Routes:
/// - `GET /`          -> pre-rendered `pages/index.mdx`
/// - `GET /blog`      -> pre-rendered listing of all published blog posts
/// - `GET /:path+`    -> pre-rendered page matching the given URL key
#[event(fetch)]
async fn fetch(req: Request, _env: Env, _ctx: Context) -> worker::Result<Response> {
    console_error_panic_hook::set_once();

    Router::new()
        .get("/", |_, _| handler::serve_index(&STORE))
        .get("/blog", |_, _| handler::serve_blog_index(&STORE))
        .get("/:path+", |_req, ctx| {
            // The `:path+` wildcard captures the remaining URL segments.
            let path = ctx.param("path").unwrap_or(&String::new()).clone();
            handler::serve_page(&STORE, &path)
        })
        .run(req, _env)
        .await
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
