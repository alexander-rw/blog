use markdown::mdast::Node;
use serde::{Deserialize, Serialize};

use crate::{error::AppError, page::yaml_from_ast};

/// Summary data for a single blog post, used in the post listing and search index.
#[derive(Debug, Clone, Serialize)]
pub struct PostListing {
    pub title: String,
    /// URL-safe directory name under `pages/blog/`.
    pub slug: String,
    pub date: Option<String>,
    pub description: Option<String>,
    /// Plain-text body content stripped of all markup, used for client-side full-text search.
    pub body: String,
}

/// All frontmatter fields relevant to post listings.
///
/// Unknown keys in YAML are silently ignored by serde.
#[derive(Debug, Deserialize)]
struct Frontmatter {
    title: String,
    date: Option<String>,
    description: Option<String>,
    /// When `true`, the post is excluded from public listings.
    #[serde(default)]
    draft: Option<bool>,
}

/// Extract a [`PostListing`] from an already-parsed MDX AST.
///
/// Returns `None` for posts with `draft: true` in their frontmatter.
///
/// # Arguments
///
/// * `slug`  - The URL-safe directory name (last path segment, e.g. `"my-post"`)
/// * `ast`   - Parsed MDX AST containing a YAML frontmatter node
///
/// # Errors
///
/// Returns [`AppError::ParseError`] if the YAML block cannot be deserialised.
pub(crate) fn extract_listing(slug: &str, ast: &Node) -> Result<Option<PostListing>, AppError> {
    let yaml = yaml_from_ast(ast);
    let fm: Frontmatter =
        serde_yaml::from_str(yaml).map_err(|e| AppError::ParseError(format!("{slug}: {e}")))?;

    if matches!(fm.draft, Some(true)) {
        return Ok(None);
    }

    Ok(Some(PostListing {
        title: fm.title,
        slug: slug.to_owned(),
        date: fm.date,
        description: fm.description,
        body: body_text_from_ast(ast),
    }))
}

/// Walk an MDX AST and collect all visible text content into a single string.
///
/// Text nodes from different blocks are separated by spaces. YAML frontmatter
/// and raw HTML nodes are skipped because they are not user-readable body content.
///
/// # Arguments
///
/// * `ast` - The root [`Node`] of a parsed MDX document
///
/// # Returns
///
/// A `String` containing all text and inline/block code content.
pub(crate) fn body_text_from_ast(ast: &Node) -> String {
    let mut buf = String::new();
    collect_text(ast, &mut buf);
    buf
}

/// Recursively accumulate text content from `node` into `buf`.
fn collect_text(node: &Node, buf: &mut String) {
    match node {
        // Leaf text nodes — the primary source of searchable content.
        Node::Text(t) => push_text(buf, &t.value),
        // Inline and fenced code is also valuable for search (e.g. function names, commands).
        Node::InlineCode(ic) => push_text(buf, &ic.value),
        Node::Code(c) => push_text(buf, &c.value),
        // Skip: YAML frontmatter (metadata, not body) and raw HTML nodes (markup noise).
        Node::Yaml(_) | Node::Html(_) => {}
        // All container nodes: recurse into children.
        _ => {
            if let Some(children) = node.children() {
                for child in children {
                    collect_text(child, buf);
                }
            }
        }
    }
}

/// Append `val` to `buf`, inserting a space separator when `buf` is non-empty.
fn push_text(buf: &mut String, val: &str) {
    if val.is_empty() {
        return;
    }
    if !buf.is_empty() {
        buf.push(' ');
    }
    buf.push_str(val);
}
