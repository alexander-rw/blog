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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_slug_resolves() {
        assert_eq!(lookup("other-writers"), Some("/blog/blog-of-blogs"));
    }

    #[test]
    fn unknown_slug_returns_none() {
        assert_eq!(lookup("no-such-page"), None);
    }
}
