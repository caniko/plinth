//! Listing and publishing posts for both target kinds.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use chrono::{Local, NaiveDate, Utc};
use plinth_shared::{ContentFormat, PublishArticleRequest};
use serde::Deserialize;

use crate::{Draft, FieldError, Replaces, Site, Target};

/// How long an admin API request may take before it counts as unreachable.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// The outcome of a successful publish.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Receipt {
    Personal { slug: String, url: String },
    Project { slug: String, path: PathBuf },
}

/// One existing post, as listed by the workspace.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostSummary {
    pub slug: String,
    pub title: String,
    pub date: Option<String>,
    pub published: bool,
    /// The editable post, for a markdown draft the workspace can reopen.
    pub draft: Option<Draft>,
}

/// Why listing or publishing failed.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum PublishError {
    #[error("the draft has {} field problem(s)", .0.len())]
    Invalid(Vec<FieldError>),
    #[error("a post with this slug already exists")]
    SlugTaken,
    /// The opened site draft can no longer be replaced (changed, published or
    /// removed since it was opened); nothing was written.
    #[error("the opened draft can no longer be replaced")]
    DraftChanged,
    #[error("the API key was rejected")]
    Unauthorized,
    #[error("could not reach the site: {0}")]
    Unreachable(String),
    #[error("the site rejected the post ({status}): {message}")]
    Rejected { status: u16, message: String },
    #[error("{0}")]
    Io(String),
}

/// List existing posts for a site.
pub async fn list_posts(site: &Site) -> Result<Vec<PostSummary>, PublishError> {
    match &site.target {
        Target::Personal {
            api_url, api_key, ..
        } => list_personal(api_url, api_key.expose()).await,
        Target::Project { root, .. } => list_project(root),
    }
}

/// Publish a validated draft. `Invalid` is returned before any I/O happens.
pub async fn publish(site: &Site, draft: &Draft) -> Result<Receipt, PublishError> {
    let draft = draft.normalized();
    if let Err(errors) = draft.validate() {
        return Err(PublishError::Invalid(errors));
    }
    match &site.target {
        Target::Personal {
            api_url, api_key, ..
        } => {
            let public = site.target.public_url().unwrap_or(api_url);
            publish_personal(api_url, public, api_key.expose(), &draft).await
        }
        Target::Project { root, .. } => publish_project(root, &draft),
    }
}

// ── Personal (admin API) ────────────────────────────────────────────────

#[derive(Deserialize)]
struct ApiPost {
    slug: String,
    title: String,
    published_at: chrono::DateTime<Utc>,
}

#[derive(Deserialize)]
struct ApiDraft {
    id: i64,
    slug: String,
    title: String,
    description: String,
    tags: Vec<String>,
    content: String,
    content_format: String,
    published_at: chrono::DateTime<Utc>,
}

#[derive(Deserialize)]
struct ApiErrorBody {
    error: Option<String>,
}

#[derive(Deserialize)]
struct ApiPublishResponse {
    slug: Option<String>,
}

fn http_client() -> Result<reqwest::Client, PublishError> {
    reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|error| PublishError::Io(format!("could not build the HTTP client: {error}")))
}

fn unreachable(error: reqwest::Error) -> PublishError {
    // Plain words and the URL, not the HTTP library's phrasing.
    let url = error.url().map(|url| url.to_string()).unwrap_or_default();
    let what = if error.is_timeout() {
        "timed out"
    } else if error.is_connect() {
        "could not connect"
    } else {
        "failed"
    };
    if error.is_connect() || error.is_timeout() || error.is_request() {
        PublishError::Unreachable(format!("request to {url} {what}"))
    } else {
        PublishError::Io(error.to_string())
    }
}

async fn error_body(response: reqwest::Response) -> String {
    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    serde_json::from_str::<ApiErrorBody>(&text)
        .ok()
        .and_then(|body| body.error)
        .unwrap_or_else(|| {
            let snippet: String = text.chars().take(200).collect();
            if snippet.is_empty() {
                status.to_string()
            } else {
                snippet
            }
        })
}

fn day(at: chrono::DateTime<Utc>) -> Option<String> {
    Some(at.date_naive().format("%Y-%m-%d").to_string())
}

async fn list_personal(api_url: &str, api_key: &str) -> Result<Vec<PostSummary>, PublishError> {
    let client = http_client()?;
    let response = client
        .get(format!("{api_url}/api/posts"))
        .send()
        .await
        .map_err(unreachable)?;
    if !response.status().is_success() {
        return Err(rejected(response).await);
    }
    let posts: Vec<ApiPost> = response.json().await.map_err(|error| {
        PublishError::Io(format!("unexpected post list from {api_url}: {error}"))
    })?;
    let mut summaries: Vec<PostSummary> = posts
        .into_iter()
        .map(|post| PostSummary {
            slug: post.slug,
            title: post.title,
            date: day(post.published_at),
            published: true,
            draft: None,
        })
        .collect();

    // Drafts are admin-only; a server without the endpoint has none to list.
    let response = client
        .get(format!("{api_url}/api/admin/drafts"))
        .bearer_auth(api_key)
        .send()
        .await
        .map_err(unreachable)?;
    match response.status().as_u16() {
        404 | 405 => {}
        _ if !response.status().is_success() => return Err(rejected(response).await),
        _ => {
            let drafts: Vec<ApiDraft> = response.json().await.map_err(|error| {
                PublishError::Io(format!("unexpected draft list from {api_url}: {error}"))
            })?;
            summaries.extend(drafts.into_iter().map(|post| PostSummary {
                date: day(post.published_at),
                published: false,
                // Typst drafts carry pre-rendered HTML this editor cannot make.
                draft: (post.content_format == "markdown").then(|| Draft {
                    title: post.title.clone(),
                    slug: post.slug.clone(),
                    description: post.description,
                    tags: post.tags,
                    body: post.content,
                    published: false,
                    replaces: Some(Replaces {
                        slug: post.slug.clone(),
                        version: post.id.to_string(),
                    }),
                }),
                slug: post.slug,
                title: post.title,
            }));
        }
    }
    summaries.sort_by(|a, b| b.date.cmp(&a.date).then(a.slug.cmp(&b.slug)));
    Ok(summaries)
}

async fn rejected(response: reqwest::Response) -> PublishError {
    let status = response.status().as_u16();
    if status == 401 || status == 403 {
        PublishError::Unauthorized
    } else {
        PublishError::Rejected {
            status,
            message: error_body(response).await,
        }
    }
}

async fn publish_personal(
    api_url: &str,
    public_url: &str,
    api_key: &str,
    draft: &Draft,
) -> Result<Receipt, PublishError> {
    let request = PublishArticleRequest {
        title: Some(draft.title.clone()),
        content: draft.body.clone(),
        slug: Some(draft.slug.clone()),
        description: Some(draft.description.clone()).filter(|text| !text.is_empty()),
        author: None,
        tags: Some(draft.tags.clone()),
        featured: None,
        published: Some(draft.published),
        embedding: None,
        content_format: Some(ContentFormat::Markdown),
        html_content: None,
        series: None,
        series_title: None,
        series_position: None,
    };

    // An opened draft is replaced in place (and may be renamed or promoted);
    // anything else is created, and a taken slug is refused.
    let client = http_client()?;
    let request_builder = match draft.replaces.as_ref() {
        Some(replaces) => {
            let mut url = reqwest::Url::parse(&format!("{api_url}/api/admin/drafts"))
                .map_err(|error| PublishError::Io(format!("bad API address {api_url}: {error}")))?;
            url.path_segments_mut()
                .map_err(|()| PublishError::Io(format!("bad API address {api_url}")))?
                .push(&replaces.version);
            client.put(url)
        }
        None => client.post(format!("{api_url}/api/admin/articles")),
    };
    let response = request_builder
        .bearer_auth(api_key)
        .json(&request)
        .send()
        .await
        .map_err(unreachable)?;

    let status = response.status();
    if status.is_success() {
        let slug = response
            .json::<ApiPublishResponse>()
            .await
            .ok()
            .and_then(|body| body.slug)
            .unwrap_or_else(|| draft.slug.clone());
        return Ok(Receipt::Personal {
            url: format!("{public_url}/posts/{slug}"),
            slug,
        });
    }

    if status.as_u16() == 401 || status.as_u16() == 403 {
        return Err(PublishError::Unauthorized);
    }

    if status.as_u16() == 409 {
        return Err(PublishError::SlugTaken);
    }

    // Only the API's 410 means the draft is gone; a 404 (old server, wrong
    // base path, proxy) stays a rejection with its detail.
    if status.as_u16() == 410 && draft.replaces.is_some() {
        return Err(PublishError::DraftChanged);
    }

    let message = error_body(response).await;
    Err(PublishError::Rejected {
        status: status.as_u16(),
        message,
    })
}

// ── Project (repository files) ──────────────────────────────────────────

pub(crate) fn render_post_file(draft: &Draft, date: NaiveDate) -> String {
    let quoted = |value: &str| serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string());
    let tags = serde_json::to_string(&draft.tags).unwrap_or_else(|_| "[]".to_string());
    let body = draft.body.trim_end();
    format!(
        "---\ntitle: {}\ndescription: {}\ndate: \"{}\"\ntags: {}\npublished: {}\n---\n{body}\n",
        quoted(&draft.title),
        quoted(&draft.description),
        date.format("%Y-%m-%d"),
        tags,
        draft.published,
    )
}

/// The site's posts directory, from `[blog].posts_dir` in `plinth-project.toml`.
///
/// Without a `[blog]` table `plinth-project` renders no posts, so writing a
/// file there would report a publish that never appears on the site.
pub(crate) fn project_posts_dir(root: &Path) -> Result<PathBuf, PublishError> {
    let config = root.join("plinth-project.toml");
    let text = fs::read_to_string(&config).map_err(|error| {
        PublishError::Io(format!("could not read {}: {error}", config.display()))
    })?;
    let table: toml::Table = text
        .parse()
        .map_err(|error| PublishError::Io(format!("{}: {error}", config.display())))?;
    let Some(blog) = table.get("blog") else {
        return Err(PublishError::Io(format!(
            "{} has no [blog] table, so the site would not render posts. Add `[blog]` to enable them.",
            config.display()
        )));
    };
    let dir = blog
        .get("posts_dir")
        .and_then(toml::Value::as_str)
        .unwrap_or("posts");
    // Same rule as the reader: a plain relative path inside the site root.
    if Path::new(dir)
        .components()
        .any(|part| !matches!(part, std::path::Component::Normal(_)))
    {
        return Err(PublishError::Io(format!(
            "{}: blog.posts_dir must be a relative path inside the site",
            config.display()
        )));
    }
    Ok(root.join(dir))
}

fn publish_project(root: &Path, draft: &Draft) -> Result<Receipt, PublishError> {
    let posts_dir = project_posts_dir(root)?;
    fs::create_dir_all(&posts_dir).map_err(|error| {
        PublishError::Io(format!("could not create {}: {error}", posts_dir.display()))
    })?;

    let slug = draft.slug.as_str();
    let path = posts_dir.join(format!("{slug}.md"));
    let markdown = render_post_file(draft, Local::now().date_naive());
    if let Some(replaces) = &draft.replaces {
        take_draft(&posts_dir, replaces, |posts_dir| {
            write_atomic(posts_dir, &path, markdown.as_bytes())
        })?;
    } else {
        if path.exists() {
            return Err(PublishError::SlugTaken);
        }
        write_atomic(&posts_dir, &path, markdown.as_bytes())?;
    }
    Ok(Receipt::Project {
        slug: slug.to_string(),
        path,
    })
}

/// Run `write` in place of the opened draft file.
///
/// The file is first renamed aside (atomic), so what is checked is exactly
/// what gets removed: a post published or edited meanwhile is never touched.
/// The aside copy goes back if it is not the opened draft or `write` fails.
fn take_draft(
    posts_dir: &Path,
    replaces: &Replaces,
    write: impl FnOnce(&Path) -> Result<(), PublishError>,
) -> Result<(), PublishError> {
    let old = replaces.slug.as_str();
    let old_path = posts_dir.join(format!("{old}.md"));
    let gone = || PublishError::DraftChanged;
    let single_name = matches!(
        Path::new(old).components().collect::<Vec<_>>().as_slice(),
        [std::path::Component::Normal(_)]
    );
    if !single_name {
        return Err(gone());
    }
    // A fresh directory is an atomic reservation (`create_dir` fails if the
    // name exists), so the move below can never clobber an earlier copy.
    let holder = loop {
        let candidate = posts_dir.join(format!(
            ".plinth-replace-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        match fs::create_dir(&candidate) {
            Ok(()) => break candidate,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(PublishError::Io(format!(
                    "could not create {}: {error}",
                    candidate.display()
                )));
            }
        }
    };
    let aside = holder.join(format!("{old}.md"));
    let discard_holder = || {
        let _ = fs::remove_file(&aside);
        let _ = fs::remove_dir(&holder);
    };
    if let Err(error) = fs::rename(&old_path, &aside) {
        discard_holder();
        return Err(match error.kind() {
            std::io::ErrorKind::NotFound => gone(),
            _ => PublishError::Io(format!(
                "could not move {} aside: {error}",
                old_path.display()
            )),
        });
    }
    // `hard_link` never clobbers: if a file took the old name meanwhile, the
    // copy stays aside and the error says where.
    let put_back = |error: PublishError| match fs::hard_link(&aside, &old_path) {
        Ok(()) => {
            discard_holder();
            error
        }
        Err(link_error) => PublishError::Io(format!(
            "{error}; the opened draft could not be put back ({link_error}) and is kept at {}",
            aside.display()
        )),
    };
    let text = fs::read_to_string(&aside).unwrap_or_default();
    let still_draft = text == replaces.version
        && parse_frontmatter(&text)
            .is_some_and(|(frontmatter, _)| frontmatter.published == Some(false));
    if !still_draft {
        return Err(put_back(gone()));
    }
    if let Err(error) = write(posts_dir) {
        return Err(put_back(error));
    }
    discard_holder();
    Ok(())
}

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

fn write_atomic(directory: &Path, target: &Path, bytes: &[u8]) -> Result<(), PublishError> {
    let name = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("post.md");
    let temporary = directory.join(format!(
        ".{name}.{}.{}.tmp",
        std::process::id(),
        TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&temporary, bytes).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        PublishError::Io(format!("could not write {}: {error}", target.display()))
    })?;
    // `hard_link` fails if the target exists, so a post that appeared while
    // the temp file was written is never replaced (a rename would clobber it).
    let linked = fs::hard_link(&temporary, target);
    let _ = fs::remove_file(&temporary);
    linked.map_err(|error| match error.kind() {
        std::io::ErrorKind::AlreadyExists => PublishError::SlugTaken,
        _ => PublishError::Io(format!("could not move the post into place: {error}")),
    })
}

fn list_project(root: &Path) -> Result<Vec<PostSummary>, PublishError> {
    let posts_dir = project_posts_dir(root)?;
    let entries = match fs::read_dir(&posts_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(PublishError::Io(format!(
                "could not read {}: {error}",
                posts_dir.display()
            )));
        }
    };

    let mut posts = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("md") {
            continue;
        }
        let Some(slug) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        // Unreadable or malformed files are skipped, not fatal: the list is a
        // convenience view, the files remain the source of truth.
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let Some((frontmatter, body)) = parse_frontmatter(&text) else {
            continue;
        };
        let Some(title) = frontmatter.title else {
            continue;
        };
        let published = frontmatter.published.unwrap_or(true);
        posts.push(PostSummary {
            slug: slug.to_string(),
            draft: (!published).then(|| Draft {
                title: title.clone(),
                slug: slug.to_string(),
                description: frontmatter.description.unwrap_or_default(),
                tags: frontmatter.tags.unwrap_or_default(),
                body: body.trim_end().to_string(),
                published: false,
                replaces: Some(Replaces {
                    slug: slug.to_string(),
                    version: text.clone(),
                }),
            }),
            title,
            date: frontmatter.date,
            published,
        });
    }
    posts.sort_by(|a, b| b.date.cmp(&a.date).then(a.slug.cmp(&b.slug)));
    Ok(posts)
}

#[derive(Default)]
struct Frontmatter {
    title: Option<String>,
    description: Option<String>,
    date: Option<String>,
    tags: Option<Vec<String>>,
    published: Option<bool>,
}

/// Minimal frontmatter reader for the fixed post contract: `---`, `key: value`
/// lines, `---`, body. Returns `None` when the fences are missing.
fn parse_frontmatter(text: &str) -> Option<(Frontmatter, &str)> {
    let mut lines = text.split_inclusive('\n');
    if lines.next()?.trim_end() != "---" {
        return None;
    }
    let mut frontmatter = Frontmatter::default();
    let mut offset = text.find('\n')? + 1;
    for line in lines {
        offset += line.len();
        let line = line.trim_end();
        if line == "---" {
            return Some((frontmatter, &text[offset..]));
        }
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let value = value.trim();
        match key.trim() {
            "title" => frontmatter.title = Some(yaml_string(value)),
            "description" => frontmatter.description = Some(yaml_string(value)),
            // The contract writes a JSON array, which is also YAML.
            "tags" => frontmatter.tags = serde_json::from_str(value).ok(),
            "date" => frontmatter.date = Some(yaml_string(value)),
            "published" => {
                frontmatter.published = match value {
                    "true" => Some(true),
                    "false" => Some(false),
                    _ => None,
                }
            }
            _ => {}
        }
    }
    None
}

/// Read a YAML double-quoted scalar (the contract emits JSON strings) or a
/// plain scalar.
fn yaml_string(value: &str) -> String {
    if value.starts_with('"') {
        serde_json::from_str::<String>(value)
            .unwrap_or_else(|_| value.trim_matches('"').to_string())
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn post_file_matches_the_fixed_contract() {
        let draft = Draft {
            title: "A \"quoted\" title".to_string(),
            slug: "a-quoted-title".to_string(),
            description: "One-line summary".to_string(),
            tags: vec!["rust".to_string(), "release".to_string()],
            body: "Hello *world*\n\n".to_string(),
            published: true,
            replaces: None,
        };
        let date = NaiveDate::from_ymd_opt(2026, 11, 10).unwrap();
        assert_eq!(
            render_post_file(&draft, date),
            concat!(
                "---\n",
                "title: \"A \\\"quoted\\\" title\"\n",
                "description: \"One-line summary\"\n",
                "date: \"2026-11-10\"\n",
                "tags: [\"rust\",\"release\"]\n",
                "published: true\n",
                "---\n",
                "Hello *world*\n",
            )
        );
    }

    #[test]
    fn frontmatter_round_trips_through_the_reader() {
        let draft = Draft {
            title: "Title: with colon".to_string(),
            description: "d".to_string(),
            tags: Vec::new(),
            body: "body".to_string(),
            published: false,
            ..Draft::default()
        };
        let date = NaiveDate::from_ymd_opt(2026, 11, 10).unwrap();
        let text = render_post_file(&draft, date);
        let (parsed, body) = parse_frontmatter(&text).unwrap();
        assert_eq!(parsed.title.as_deref(), Some("Title: with colon"));
        assert_eq!(parsed.description.as_deref(), Some("d"));
        assert_eq!(parsed.date.as_deref(), Some("2026-11-10"));
        assert_eq!(parsed.tags, Some(Vec::new()));
        assert_eq!(parsed.published, Some(false));
        assert_eq!(body, "body\n");
    }
}
