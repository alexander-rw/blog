/// Static table mapping URL slugs to their permanent redirect targets.
///
/// Each entry is `(from_path, to_url)` where `from_path` is matched against
/// the raw URL path (without a leading `/`) and `to_url` is the full path
/// the client will be redirected to.
///
/// To add a new redirect, append a new `(&str, &str)` tuple here.
pub const REDIRECTS: &[(&str, &str)] = &[("other-writers", "/blog/blog-of-blogs")];

/// Look up the redirect target for a given path slug.
///
/// # Arguments
///
/// * `slug` - The URL path segment to look up (without leading `/`)
///
/// # Returns
///
/// The redirect target URL if a matching entry exists, or `None`.
pub fn lookup(slug: &str) -> Option<&'static str> {
    REDIRECTS
        .iter()
        .find_map(|&(from, to)| (from == slug).then_some(to))
}

/// Render [`REDIRECTS`] as a Cloudflare `_redirects` file.
///
/// Each line is `/<from> <to> 308`, matching the permanent redirect the local
/// server returns, so this table stays the single source of truth.
///
/// # Returns
///
/// The file contents, one rule per line.
pub fn cloudflare_redirects_file() -> String {
    REDIRECTS
        .iter()
        .map(|(from, to)| format!("/{from} {to} 308\n"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_slug_resolves() {
        assert_eq!(lookup("other-writers"), Some("/blog/blog-of-blogs"));
    }

    #[test]
    fn cloudflare_file_lists_every_redirect_as_308() {
        let file = cloudflare_redirects_file();

        assert_eq!(file.lines().count(), REDIRECTS.len());
        assert!(file.contains("/other-writers /blog/blog-of-blogs 308\n"));
    }

    #[test]
    fn unknown_slug_returns_none() {
        assert_eq!(lookup("no-such-page"), None);
    }
}
