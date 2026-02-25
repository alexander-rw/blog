use markdown::mdast::Node;
use serde::Deserialize;

use crate::error::AppError;

/// Metadata extracted from a page's frontmatter and body text.
#[derive(Debug, Clone)]
pub struct PageMeta {
    pub title: String,
    /// Estimated reading time in minutes (minimum 1).
    pub read_time_mins: usize,
}

/// Frontmatter fields needed for single-page metadata.
///
/// Unknown keys in YAML are silently ignored by serde.
#[derive(Debug, Deserialize)]
struct Frontmatter {
    title: String,
}

/// Return the raw YAML string from the first [`Node::Yaml`] child of `ast`.
///
/// The returned slice borrows from data owned by `ast`.
/// Returns `""` if no YAML node is found.
///
/// Marked `pub(crate)` so `post.rs` can reuse this without a third module.
pub(crate) fn yaml_from_ast(ast: &Node) -> &str {
    // `Node::children()` returns `Option<&Vec<Node>>`.
    // `.into_iter()` on `Option<T>` yields 0 or 1 items; `.flatten()` then
    // iterates the inner Vec so we get individual `&Node` references.
    ast.children()
        .into_iter()
        .flatten()
        .find_map(|n| {
            if let Node::Yaml(y) = n {
                Some(y.value.as_str())
            } else {
                None
            }
        })
        .unwrap_or("")
}

/// Extract page title and estimated reading time from a parsed MDX AST.
///
/// Reading time uses a 200 wpm rate on the body text (frontmatter excluded),
/// with a minimum of 1 minute.
///
/// # Arguments
///
/// * `ast`     - Parsed AST containing a YAML frontmatter node
/// * `content` - Raw MDX source, used for word counting
///
/// # Returns
///
/// A [`PageMeta`] with the page title and read-time estimate.
///
/// # Errors
///
/// Returns [`AppError::ParseError`] if the YAML block cannot be deserialised.
pub fn extract_meta(ast: &Node, content: &str) -> Result<PageMeta, AppError> {
    let yaml = yaml_from_ast(ast);
    let fm: Frontmatter =
        serde_yaml::from_str(yaml).map_err(|e| AppError::ParseError(e.to_string()))?;

    // Exclude the frontmatter fence itself from the word count.
    let body = strip_frontmatter(content);
    let word_count = body.split_whitespace().count();
    // Integer division: 400 words → 2 min; fewer than 200 words → 1 min (minimum).
    let read_time_mins = (word_count / 200).max(1);

    Ok(PageMeta {
        title: fm.title,
        read_time_mins,
    })
}

/// Remove the leading YAML frontmatter fence (`---\n…\n---\n`) from `content`.
///
/// If the content does not begin with `---`, it is returned unchanged.
fn strip_frontmatter(content: &str) -> &str {
    // Frontmatter must start at the very first byte.
    let Some(rest) = content.strip_prefix("---") else {
        return content;
    };
    // The closing fence begins on its own line.
    let Some(idx) = rest.find("\n---") else {
        return content;
    };
    // Advance past "\n---" (4 bytes) and skip any immediately following newline.
    rest[idx + 4..].trim_start_matches(['\n', '\r'])
}
