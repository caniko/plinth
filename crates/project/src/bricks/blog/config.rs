use std::path::{Component, Path, PathBuf};

use serde::Deserialize;

/// The optional `[blog]` table in `plinth-project.toml`.
///
/// A site without this table renders exactly as before; the blog output only
/// exists when the table is present.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlogConfig {
    /// Index heading, feed title, and navigation label. Defaults to `Blog`.
    #[serde(default)]
    pub title: Option<String>,
    /// Markdown post directory, relative to the config file. Defaults to `posts`.
    #[serde(default)]
    pub posts_dir: Option<String>,
}

/// Title used when `[blog]` omits `title`.
pub(crate) const DEFAULT_TITLE: &str = "Blog";

/// Post directory used when `[blog]` omits `posts_dir`.
pub(crate) const DEFAULT_POSTS_DIR: &str = "posts";

impl BlogConfig {
    /// Configured title, or [`DEFAULT_TITLE`] for a blank or missing value.
    pub(crate) fn title(&self) -> &str {
        self.title
            .as_deref()
            .map(str::trim)
            .filter(|title| !title.is_empty())
            .unwrap_or(DEFAULT_TITLE)
    }

    /// Resolve `posts_dir` against the config file's directory.
    ///
    /// Absolute paths and `.`/`..` components are rejected so a stray config
    /// cannot name a directory outside the site. Symlinks are not resolved:
    /// posts are trusted input from the site owner.
    pub(crate) fn posts_dir(&self, base: &Path) -> Result<PathBuf, String> {
        let raw = self
            .posts_dir
            .as_deref()
            .unwrap_or(DEFAULT_POSTS_DIR)
            .trim();
        let relative = Path::new(raw);
        if raw.is_empty()
            || !relative
                .components()
                .all(|component| matches!(component, Component::Normal(_)))
        {
            return Err(format!(
                "blog.posts_dir `{raw}` must be a relative path inside the site directory"
            ));
        }
        Ok(base.join(relative))
    }
}
