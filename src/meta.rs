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

/// Summary data for a single blog post, used in the post listing.
#[derive(Debug, Clone)]
pub struct PostListing {
    pub title: String,
    /// URL-safe directory name under `pages/blog/`.
    pub slug: String,
    pub date: Option<String>,
    pub description: Option<String>,
}

/// All supported frontmatter fields.
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

/// Return the raw YAML string from the first [`Node::Yaml`] child of `ast`.
///
/// The returned slice borrows from data owned by `ast`.
/// Returns `""` if no YAML node is found.
fn yaml_from_ast(ast: &Node) -> &str {
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

/// List all published blog posts found under `pages/blog/*/page.mdx`.
///
/// Posts with `draft: true` in frontmatter are excluded.
/// The returned list is sorted by `date` descending; posts without a date
/// appear at the end.
///
/// # Errors
///
/// Returns [`AppError::NotFound`] if `pages/blog` cannot be read.
/// Returns [`AppError::ParseError`] if any post's frontmatter is malformed.
pub async fn list_posts() -> Result<Vec<PostListing>, AppError> {
    let mut dir = tokio::fs::read_dir("pages/blog")
        .await
        .map_err(|e| AppError::NotFound(format!("pages/blog: {e}")))?;

    let mut posts: Vec<PostListing> = Vec::new();

    while let Some(entry) = dir
        .next_entry()
        .await
        .map_err(|e| AppError::ParseError(e.to_string()))?
    {
        // Use the async file_type check to avoid blocking the reactor.
        let file_type = entry
            .file_type()
            .await
            .map_err(|e| AppError::ParseError(e.to_string()))?;
        if !file_type.is_dir() {
            continue;
        }

        let slug = match entry.file_name().to_str() {
            Some(s) => s.to_owned(),
            // Skip entries whose names are not valid UTF-8.
            None => continue,
        };

        let file_path = entry.path().join("page.mdx");
        let content = match tokio::fs::read_to_string(&file_path).await {
            Ok(c) => c,
            // Skip subdirectories that have no page.mdx.
            Err(_) => continue,
        };

        // Re-use the shared parse path so validation stays consistent.
        let ast = crate::parse_mdx(&content).map_err(AppError::ParseError)?;
        let yaml = yaml_from_ast(&ast);
        let fm: Frontmatter =
            serde_yaml::from_str(yaml).map_err(|e| AppError::ParseError(format!("{slug}: {e}")))?;

        if fm.draft == Some(true) {
            continue;
        }

        posts.push(PostListing {
            title: fm.title,
            slug,
            date: fm.date,
            description: fm.description,
        });
    }

    // Descending date sort; entries without dates sink to the bottom.
    posts.sort_by(|a, b| match (&b.date, &a.date) {
        (Some(bd), Some(ad)) => bd.cmp(ad),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    });

    Ok(posts)
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
