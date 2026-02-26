use markdown::mdast::Node;
use serde::Deserialize;

use crate::{error::AppError, page::yaml_from_ast};

/// Summary data for a single blog post, used in the post listing.
#[derive(Debug, Clone)]
pub struct PostListing {
    pub title: String,
    /// URL-safe directory name under `pages/blog/`.
    pub slug: String,
    pub date: Option<String>,
    pub description: Option<String>,
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
    }))
}
