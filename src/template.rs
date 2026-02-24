use crate::meta::{PageMeta, PostListing};

const LINKEDIN_URL: &str = "https://www.linkedin.com/in/alexanderreyeswainwright";
const GITHUB_URL: &str = "https://github.com/alexander-rw";

/// CSS injected into every page's `<style>` block.
///
/// Raw string literal (`r"..."`) is used so the CSS braces don't need
/// escaping — the `{{` / `}}` escape is only required inside `format!` itself,
/// not in the *values* substituted into it.
const STYLES: &str = r"
:root {
  --font-family: 'Plus Jakarta Sans', sans-serif;
  --bg: #F7F4EC;
  --text: #1C1B33;
  --accent: #7C3AED;
  --muted: #6B7280;
  --max-width: 720px;
}
[data-theme='dark'] {
  --bg: #0F0E1A;
  --text: #E4E2F0;
  --accent: #A78BFA;
}
*, *::before, *::after { box-sizing: border-box; }
html, body {
  background: var(--bg);
  color: var(--text);
  font-family: var(--font-family);
  margin: 0;
  padding: 0;
}
body > header, body > main, body > footer {
  max-width: var(--max-width);
  margin: 0 auto;
  padding: 0 1.5rem;
}
body > header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-top: 1.5rem;
  padding-bottom: 1.5rem;
  border-bottom: 1px solid color-mix(in srgb, var(--text) 15%, transparent);
}
nav a {
  color: var(--text);
  text-decoration: none;
  font-weight: 600;
  margin-right: 1.5rem;
}
nav a:hover { color: var(--accent); }
#theme-toggle {
  background: none;
  border: 1px solid color-mix(in srgb, var(--text) 30%, transparent);
  border-radius: 6px;
  color: var(--text);
  cursor: pointer;
  font-family: var(--font-family);
  font-size: 0.85rem;
  font-weight: 600;
  padding: 0.3rem 0.8rem;
}
#theme-toggle:hover { border-color: var(--accent); color: var(--accent); }
main { padding-top: 2.5rem; padding-bottom: 2.5rem; }
.page-title {
  font-size: 2rem;
  font-weight: 800;
  color: var(--text);
  margin-bottom: 0.25rem;
}
.read-time {
  color: var(--muted);
  font-size: 0.9rem;
  margin-top: 0;
  margin-bottom: 2rem;
}
.content h1, .content h2 { color: var(--accent); }
.content h1 { font-size: 1.6rem; font-weight: 700; }
.content h2 { font-size: 1.3rem; font-weight: 700; }
.content a { color: var(--accent); }
.content p { line-height: 1.75; }
.post-list { list-style: none; padding: 0; margin-top: 1.5rem; }
.post-list li {
  border-bottom: 1px solid color-mix(in srgb, var(--text) 10%, transparent);
  padding: 1rem 0;
}
.post-list li:first-child { padding-top: 0; }
.post-list a {
  color: var(--text);
  text-decoration: none;
  font-size: 1.1rem;
  font-weight: 700;
}
.post-list a:hover { color: var(--accent); }
.post-meta { font-size: 0.85rem; color: var(--muted); margin-top: 0.25rem; }
.post-desc { font-size: 0.9rem; color: var(--muted); margin-top: 0.25rem; }
footer {
  border-top: 1px solid color-mix(in srgb, var(--text) 15%, transparent);
  padding-top: 1rem;
  padding-bottom: 1.5rem;
  font-size: 0.85rem;
  color: var(--muted);
}
footer a { color: var(--muted); text-decoration: none; }
footer a:hover { color: var(--accent); }
";

/// Inline JavaScript for the light/dark theme toggle.
///
/// On load: reads `localStorage`, then falls back to `prefers-color-scheme`.
/// On button click: flips the theme and persists the choice.
const THEME_SCRIPT: &str = r"
(function () {
  var btn = document.getElementById('theme-toggle');
  var html = document.documentElement;
  function applyTheme(theme) {
    html.setAttribute('data-theme', theme);
    btn.textContent = theme === 'dark' ? 'Light' : 'Dark';
  }
  var stored = localStorage.getItem('theme');
  if (stored) {
    applyTheme(stored);
  } else if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
    applyTheme('dark');
  }
  btn.addEventListener('click', function () {
    var next = html.getAttribute('data-theme') === 'dark' ? 'light' : 'dark';
    localStorage.setItem('theme', next);
    applyTheme(next);
  });
}());
";

/// Escape characters that carry special meaning in HTML.
///
/// Used for all user-supplied strings placed in HTML text nodes or attributes.
fn escape_html(s: &str) -> String {
    // Process in a single pass to avoid repeated allocations.
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

/// Build the shared HTML shell: `<head>`, `<header>` nav, `<footer>`, and
/// the inline `<style>` + `<script>` blocks.
///
/// `body_content` is inserted verbatim inside `<main>` — callers are
/// responsible for escaping any user-supplied data within it.
///
/// # Arguments
///
/// * `page_title`   - Text for the browser-tab `<title>` element (will be HTML-escaped)
/// * `body_content` - Pre-rendered HTML fragment placed inside `<main>`
fn html_shell(page_title: &str, body_content: &str) -> String {
    let page_title = escape_html(page_title);
    format!(
        r#"<!DOCTYPE html>
<html lang="en" data-theme="light">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>{page_title}</title>
  <link rel="preconnect" href="https://fonts.googleapis.com" />
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
  <link href="https://fonts.googleapis.com/css2?family=Plus+Jakarta+Sans:wght@400;600;700;800&display=swap" rel="stylesheet" />
  <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@picocss/pico@2/css/pico.classless.min.css" />
  <style>{STYLES}</style>
</head>
<body>
  <header>
    <nav>
      <a href="/">About Me</a>
      <a href="/blog">Posts</a>
    </nav>
    <button id="theme-toggle" aria-label="Toggle theme">Dark</button>
  </header>
  <main>
    {body_content}
  </main>
  <footer>
    <a href="{LINKEDIN_URL}" target="_blank" rel="noopener noreferrer">LinkedIn</a>
    &nbsp;&bull;&nbsp;
    <a href="{GITHUB_URL}" target="_blank" rel="noopener noreferrer">GitHub</a>
  </footer>
  <script>{THEME_SCRIPT}</script>
</body>
</html>"#
    )
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
/// * `html_content` - MDX-rendered HTML fragment (not escaped — already HTML)
pub fn render_page(meta: &PageMeta, html_content: &str) -> String {
    let title = escape_html(&meta.title);
    let mins = meta.read_time_mins;
    let body_content = format!(
        r#"<h1 class="page-title">{title}</h1>
<p class="read-time">{mins} min read</p>
<div class="content">{html_content}</div>"#
    );
    html_shell(&meta.title, &body_content)
}

/// Render a `<ul>` listing of blog posts inside the shared HTML shell.
///
/// Each item shows the post title (linked to `/blog/{slug}`), optional date,
/// and optional description.
///
/// # Arguments
///
/// * `posts` - Slice of post summaries, typically pre-sorted by date descending
pub fn render_post_list(posts: &[PostListing]) -> String {
    let items: String = posts
        .iter()
        .map(|post| {
            let title = escape_html(&post.title);
            let href = format!("/blog/{}", post.slug);
            let meta_line = post
                .date
                .as_deref()
                .map(|d| format!(r#"<p class="post-meta">{d}</p>"#))
                .unwrap_or_default();
            let desc_line = post
                .description
                .as_deref()
                .map(|d| {
                    let d = escape_html(d);
                    format!(r#"<p class="post-desc">{d}</p>"#)
                })
                .unwrap_or_default();
            format!(r#"<li><a href="{href}">{title}</a>{meta_line}{desc_line}</li>"#)
        })
        .collect();

    let body_content = format!(r#"<h1>Posts</h1><ul class="post-list">{items}</ul>"#);
    html_shell("Posts", &body_content)
}
