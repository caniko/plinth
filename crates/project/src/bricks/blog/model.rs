use std::path::PathBuf;

use chrono::NaiveDate;

/// A site's blog: published posts newest first, plus authoring problems that
/// `check` and `build` report as blocking diagnostics.
#[derive(Clone, Debug, Default)]
pub struct Blog {
    /// Index heading, feed title, and navigation label.
    pub title: String,
    /// Directory scanned for `<slug>.md` posts.
    pub directory: PathBuf,
    /// Published posts, newest first.
    pub posts: Vec<BlogPost>,
    /// Frontmatter problems keyed by the file that caused them.
    pub errors: Vec<BlogError>,
}

/// One published post parsed from `<slug>.md`.
#[derive(Clone, Debug)]
pub struct BlogPost {
    /// File stem, already checked against `^[a-z0-9]+(-[a-z0-9]+)*$`.
    pub slug: String,
    /// Required frontmatter title.
    pub title: String,
    /// Optional summary; empty when absent.
    pub description: String,
    /// Required publication date.
    pub date: NaiveDate,
    /// Optional tags, empty when absent.
    pub tags: Vec<String>,
    /// Optional author name.
    pub author: Option<String>,
    /// Markdown body rendered to HTML.
    pub html: String,
}

/// A post-level problem that should fail `plinth-project check` and `build`.
#[derive(Clone, Debug)]
pub struct BlogError {
    /// Post file (or config file for a bad `posts_dir`) the problem belongs to.
    pub path: PathBuf,
    /// Stable diagnostic code, e.g. `"blog.invalid_date"`.
    pub code: &'static str,
    /// Human-readable explanation without the path prefix.
    pub message: String,
}
