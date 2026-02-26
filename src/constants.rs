pub const LINKEDIN_URL: &str = "https://www.linkedin.com/in/alexanderreyeswainwright";
pub const GITHUB_URL: &str = "https://github.com/alexander-rw";

/// CSS embedded at compile time from `assets/styles.css`.
///
/// `include_str!` reads the file once during compilation and bakes the content
/// directly into the binary — no file I/O at runtime.
pub const STYLES: &str = include_str!("../assets/styles.css");

/// JavaScript embedded at compile time from `assets/theme.js`.
pub const THEME_SCRIPT: &str = include_str!("../assets/theme.js");
