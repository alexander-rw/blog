//! Static export of the pre-rendered site, served by Cloudflare Workers static assets.
//!
//! The export reuses [`PageStore`], so the files it writes are byte-for-byte what
//! the local axum server returns for the same URLs.

use std::{
    borrow::Cow,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use crate::{redirects, store::PageStore, template};

/// Directory the `export` command writes to, relative to the working directory.
pub const EXPORT_DIR: &str = "dist";

/// One file of the exported site.
#[derive(Debug, Clone, PartialEq)]
pub struct SiteFile<'a> {
    /// Path relative to the export directory, e.g. `blog/my-post.html`.
    pub path: PathBuf,
    /// File contents. `Cow` borrows the HTML already held by the store and only
    /// owns the few files rendered here (like Python's "view vs copy" distinction).
    pub contents: Cow<'a, [u8]>,
}

/// List every file the static site consists of.
///
/// Pages are written as `<key>.html`, which Cloudflare serves at `/<key>`.
/// The listing is `blog.html` (served at `/blog`), the search index is
/// `search.json`, the styled not-found page is `404.html`, and redirects go
/// to `_redirects`.
///
/// # Arguments
///
/// * `store` - The fully built page store
///
/// # Returns
///
/// All site files, borrowing from `store` where possible.
pub fn site_files(store: &PageStore) -> Vec<SiteFile<'_>> {
    let pages = store.pages();
    let mut files = Vec::with_capacity(pages.len() + 4);

    files.extend(pages.map(|(key, html)| SiteFile {
        path: PathBuf::from(format!("{key}.html")),
        contents: Cow::Borrowed(html.as_bytes()),
    }));
    files.extend([
        SiteFile {
            path: PathBuf::from("blog.html"),
            contents: Cow::Borrowed(store.blog_listing().as_bytes()),
        },
        SiteFile {
            path: PathBuf::from("search.json"),
            contents: Cow::Borrowed(store.search_index()),
        },
        SiteFile {
            path: PathBuf::from("404.html"),
            contents: Cow::Owned(template::render_not_found().into_string().into_bytes()),
        },
        SiteFile {
            path: PathBuf::from("_redirects"),
            contents: Cow::Owned(redirects::cloudflare_redirects_file().into_bytes()),
        },
    ]);
    files
}

/// Replace `out_dir` with a fresh directory containing `files`.
///
/// The directory is removed first so pages deleted from `pages/` do not linger.
///
/// # Arguments
///
/// * `out_dir` - Directory to (re)create; any existing contents are deleted
/// * `files`   - Files to write, with paths relative to `out_dir`
///
/// # Errors
///
/// Returns an error if the directory cannot be removed or created, or a file
/// cannot be written.
pub fn write_site(out_dir: &Path, files: &[SiteFile<'_>]) -> Result<()> {
    if out_dir.exists() {
        fs::remove_dir_all(out_dir)
            .with_context(|| format!("removing old export at {}", out_dir.display()))?;
    }
    for file in files {
        let path = out_dir.join(&file.path);
        // `parent()` is always `Some` here because `path` starts with `out_dir`.
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("creating directory {}", parent.display()))?;
        }
        fs::write(&path, &file.contents).with_context(|| format!("writing {}", path.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    fn build_store() -> PageStore {
        PageStore::build().expect("embedded pages should build")
    }

    #[test]
    fn site_files_cover_every_route() {
        let store = build_store();

        let files = site_files(&store);
        let paths: HashSet<&Path> = files.iter().map(|f| f.path.as_path()).collect();

        for expected in [
            "index.html",
            "blog.html",
            "blog/blog-of-blogs.html",
            "blog/draft.html",
            "search.json",
            "404.html",
            "_redirects",
        ] {
            assert!(paths.contains(Path::new(expected)), "missing {expected}");
        }
        assert_eq!(paths.len(), files.len(), "export paths must be unique");
    }

    #[test]
    fn site_files_match_store_contents() {
        let store = build_store();

        let files = site_files(&store);
        let index = files
            .iter()
            .find(|f| f.path == Path::new("index.html"))
            .expect("index.html should be exported");

        assert_eq!(
            index.contents.as_ref(),
            store.page("index").expect("index page").as_bytes()
        );
    }

    #[test]
    fn write_site_replaces_directory_contents() {
        let out_dir = tempfile::tempdir().expect("temp dir");
        let stale = out_dir.path().join("stale.html");
        fs::write(&stale, "old").expect("write stale file");
        let files = [SiteFile {
            path: PathBuf::from("blog/post.html"),
            contents: Cow::Borrowed(b"<p>post</p>".as_slice()),
        }];

        write_site(out_dir.path(), &files).expect("export should succeed");

        assert!(!stale.exists());
        assert_eq!(
            fs::read(out_dir.path().join("blog/post.html")).expect("read post"),
            b"<p>post</p>"
        );
    }
}
