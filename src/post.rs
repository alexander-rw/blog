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

        if matches!(fm.draft, Some(true)) {
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
