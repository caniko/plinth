//! Static blog posts: Markdown files beside the site config rendered as
//! `blog/index.html`, `blog/<slug>/index.html`, and an RSS feed.
//!
//! The feature is opt-in through a `[blog]` table in `plinth-project.toml`.
//! Posts live in `<config dir>/<posts_dir>/<slug>.md` with YAML frontmatter.

mod config;
pub mod model;
pub(crate) mod render;

use std::path::Path;

use chrono::NaiveDate;
use gray_matter::{Matter, engine::YAML};
use pulldown_cmark::{Options, Parser, html};
use serde::Deserialize;

use crate::{Diagnostic, DiagnosticReport, ProjectSite, Severity};

pub use config::BlogConfig;
pub use model::{Blog, BlogError, BlogPost};

/// Registry entry for the opt-in blog brick.
pub struct BlogBrick;

impl crate::bricks::ProjectBrick for BlogBrick {
    fn name(&self) -> &'static str {
        "blog"
    }
}

/// The fixed frontmatter contract for a post file.
#[derive(Deserialize)]
struct Frontmatter {
    title: Option<String>,
    description: Option<String>,
    date: Option<String>,
    tags: Option<Vec<String>>,
    author: Option<String>,
    published: Option<bool>,
}

/// Load every post under the configured directory.
///
/// Authoring problems are collected on the returned [`Blog`] instead of
/// failing the load, so `check` can report all of them at once and `build`
/// refuses to render a site with errors.
pub(crate) fn load(base: &Path, config_path: &Path, config: &BlogConfig) -> Blog {
    let mut blog = Blog {
        title: config.title().to_owned(),
        ..Blog::default()
    };
    let directory = match config.posts_dir(base) {
        Ok(directory) => directory,
        Err(message) => {
            blog.errors.push(BlogError {
                path: config_path.to_path_buf(),
                code: "blog.posts_dir",
                message,
            });
            return blog;
        }
    };
    blog.directory = directory.clone();

    let scan = std::fs::read_dir(&directory).and_then(|entries| {
        entries
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<std::io::Result<Vec<_>>>()
    });
    let mut files = match scan {
        Ok(paths) => paths
            .into_iter()
            .filter(|path| {
                path.is_file() && path.extension().is_some_and(|extension| extension == "md")
            })
            .collect::<Vec<_>>(),
        // A missing directory is an empty blog, not a broken build.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        // Anything else would silently drop every post from the next deploy.
        Err(error) => {
            blog.errors.push(BlogError {
                path: directory,
                code: "blog.read",
                message: format!("cannot read the posts directory: {error}"),
            });
            return blog;
        }
    };
    files.sort();

    for file in files {
        let Some(slug) = file.file_stem().and_then(|stem| stem.to_str()) else {
            blog.errors.push(BlogError {
                path: file,
                code: "blog.slug",
                message: "post file name is not valid UTF-8".to_owned(),
            });
            continue;
        };
        if !valid_slug(slug) {
            blog.errors.push(BlogError {
                path: file.clone(),
                code: "blog.slug",
                message: format!("post slug `{slug}` must match ^[a-z0-9]+(-[a-z0-9]+)*$"),
            });
            continue;
        }
        match read_post(&file, slug) {
            Ok(Some(post)) => blog.posts.push(post),
            Ok(None) => {}
            Err((code, message)) => blog.errors.push(BlogError {
                path: file,
                code,
                message,
            }),
        }
    }

    blog.posts.sort_by(|left, right| {
        right
            .date
            .cmp(&left.date)
            .then_with(|| left.slug.cmp(&right.slug))
    });
    blog
}

/// Render one post file, or `None` when it is explicitly unpublished.
fn read_post(path: &Path, slug: &str) -> Result<Option<BlogPost>, (&'static str, String)> {
    let raw = std::fs::read_to_string(path)
        .map_err(|error| ("blog.read", format!("cannot read post: {error}")))?;
    let parsed = Matter::<YAML>::new()
        .parse::<Frontmatter>(&raw)
        .map_err(|error| ("blog.frontmatter", format!("invalid frontmatter: {error}")))?;
    let frontmatter = parsed.data.ok_or((
        "blog.frontmatter",
        "missing YAML frontmatter between --- lines".to_owned(),
    ))?;

    let title = frontmatter
        .title
        .map(|title| title.trim().to_owned())
        .filter(|title| !title.is_empty())
        .ok_or(("blog.missing_field", "missing required `title`".to_owned()))?;
    let date = frontmatter.date.as_deref().unwrap_or("").trim().to_owned();
    if date.is_empty() {
        return Err(("blog.missing_field", "missing required `date`".to_owned()));
    }
    // The shape check rejects signed and five-digit years, which chrono
    // parses but RSS dates cannot represent.
    let shaped = date.len() == 10
        && date.bytes().enumerate().all(|(index, byte)| {
            if matches!(index, 4 | 7) {
                byte == b'-'
            } else {
                byte.is_ascii_digit()
            }
        });
    let date = Some(date.as_str())
        .filter(|_| shaped)
        .and_then(|date| NaiveDate::parse_from_str(date, "%Y-%m-%d").ok())
        .filter(|parsed| parsed.format("%Y-%m-%d").to_string() == date)
        .ok_or_else(|| {
            (
                "blog.invalid_date",
                format!("date `{date}` must be a YYYY-MM-DD calendar date"),
            )
        })?;

    if !frontmatter.published.unwrap_or(true) {
        return Ok(None);
    }

    let description = frontmatter
        .description
        .map(|description| description.trim().to_owned())
        .unwrap_or_default();
    let tags = frontmatter
        .tags
        .unwrap_or_default()
        .into_iter()
        .map(|tag| tag.trim().to_owned())
        .filter(|tag| !tag.is_empty())
        .collect();
    let author = frontmatter
        .author
        .map(|author| author.trim().to_owned())
        .filter(|author| !author.is_empty());

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    let mut html_output = String::new();
    html::push_html(&mut html_output, Parser::new_ext(&parsed.content, options));

    Ok(Some(BlogPost {
        slug: slug.to_owned(),
        title,
        description,
        date,
        tags,
        author,
        html: html_output,
    }))
}

/// `^[a-z0-9]+(-[a-z0-9]+)*$`, without pulling in a regex engine.
fn valid_slug(slug: &str) -> bool {
    slug.split('-').all(|part| {
        !part.is_empty()
            && part
                .chars()
                .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
    })
}

/// Adds one blocking diagnostic per authoring problem, naming the file.
pub(crate) fn validate(site: &ProjectSite, report: &mut DiagnosticReport) {
    let Some(blog) = &site.blog else {
        return;
    };
    for page in &site.pages {
        if page.slug == "blog" || page.slug.starts_with("blog/") {
            report.diagnostics.push(Diagnostic {
                severity: Severity::Error,
                code: "blog.route_conflict",
                message: format!(
                    "page `{}` conflicts with the generated blog output",
                    page.slug
                ),
            });
        }
    }
    report
        .diagnostics
        .extend(blog.errors.iter().map(|error| Diagnostic {
            severity: Severity::Error,
            code: error.code,
            message: format!("{}: {}", error.path.display(), error.message),
        }));
}
