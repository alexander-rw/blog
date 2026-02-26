use maud::{DOCTYPE, Markup, PreEscaped, html};

use crate::{constants, page::PageMeta, post::PostListing};

/// Build the shared HTML shell: `<head>`, `<header>` nav, `<footer>`, and
/// the inline `<style>` + `<script>` blocks.
///
/// `body` is a `Markup` fragment inserted inside `<main>`. Maud auto-escapes
/// all interpolated expressions, so callers can pass user-supplied data safely.
///
/// # Arguments
///
/// * `page_title` - Text for the browser-tab `<title>` element (auto-escaped by Maud)
/// * `body`       - Pre-built Maud `Markup` fragment placed inside `<main>`
fn html_shell(page_title: &str, body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" data-theme="light" {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { (page_title) }
                link rel="preconnect" href="https://fonts.googleapis.com";
                // `crossorigin` is a valueless boolean attribute in HTML; Maud
                // emits it as `crossorigin=""` which browsers treat identically.
                link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="";
                link
                    href="https://fonts.googleapis.com/css2?family=Plus+Jakarta+Sans:wght@400;500;600;700;800&display=swap"
                    rel="stylesheet";
                link
                    rel="stylesheet"
                    href="https://cdn.jsdelivr.net/npm/@picocss/pico@2/css/pico.classless.min.css";
                // `PreEscaped` passes the CSS/JS constants through verbatim —
                // they contain raw `{`, `}`, `<`, `>` that must not be entity-encoded.
                style { (PreEscaped(constants::STYLES)) }
            }
            body {
                header {
                    nav {
                        a href="/" { "About Me" }
                        a href="/blog/blog-of-blogs" { "Other Writers" }
                        a href="/blog" { "Posts" }
                    }
                    button #theme-toggle aria-label="Toggle theme" {
                        span .icon-light { (PreEscaped(constants::THEME_ICON_SVG)) }
                        span .icon-dark  { (PreEscaped(constants::MOON_ICON_SVG)) }
                    }
                }
                main { (body) }
                footer {
                    // Icon SVGs use `currentColor`, so they inherit the link's --muted / --accent colour.
                    a href=(constants::LINKEDIN_URL) target="_blank" rel="noopener noreferrer" aria-label="LinkedIn" {
                        (PreEscaped(constants::LINKEDIN_ICON_SVG))
                    }
                    a href=(constants::GITHUB_URL) target="_blank" rel="noopener noreferrer" aria-label="GitHub" {
                        (PreEscaped(constants::GITHUB_ICON_SVG))
                    }
                }
                script { (PreEscaped(constants::THEME_SCRIPT)) }
            }
        }
    }
}

/// Wrap rendered MDX HTML in a full page with title, read-time, and the shared shell.
///
/// The page-title `<h1>` uses `color: var(--text)` (not the accent colour) so it
/// visually anchors the page. Headings *inside* `.content` get the accent colour
/// via the `.content h1, .content h2` CSS rules.
///
/// # Arguments
///
/// * `meta`         - Frontmatter-derived title and reading-time estimate
/// * `html_content` - MDX-rendered HTML fragment (not escaped — already valid HTML)
pub fn render_page(meta: &PageMeta, html_content: &str) -> Markup {
    let body = html! {
        h1 class="page-title" { (meta.title) }
        p class="read-time" { (meta.read_time_mins) " min read" }
        // `html_content` is the output of the Markdown renderer — already valid HTML,
        // so bypass Maud's auto-escaping with `PreEscaped`.
        div class="content" { (PreEscaped(html_content)) }
    };
    html_shell(&meta.title, body)
}

/// Render a `<ul>` listing of blog posts inside the shared HTML shell.
///
/// Each item shows the post title (linked to `/blog/{slug}`), optional date,
/// and optional description. All user-supplied strings are auto-escaped by Maud.
///
/// # Arguments
///
/// * `posts` - Slice of post summaries, typically pre-sorted by date descending
pub fn render_post_list(posts: &[PostListing]) -> Markup {
    let body = html! {
        h1 class="page-title" { "Posts" }
        ul class="post-list" {
            @for post in posts {
                li {
                    a class="post-card" href={ "/blog/" (post.slug) } {
                        div class="post-card-header" {
                            span class="post-title" { (post.title) }
                            @if let Some(date) = &post.date {
                                span class="post-meta" { (date) }
                            }
                        }
                        @if let Some(desc) = &post.description {
                            p class="post-desc" { (desc) }
                        }
                    }
                }
            }
        }
    };
    html_shell("Posts", body)
}
