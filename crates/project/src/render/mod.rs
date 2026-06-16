#![allow(clippy::format_collect)]

use std::fs;
use std::path::{Path, PathBuf};

use leptos::prelude::*;
use thiserror::Error;

use crate::{DiagnosticReport, Page, ProjectSection, ProjectSite, ProjectTheme, assert_valid};

mod html;
#[cfg(test)]
mod tests;

pub(crate) use html::*;

/// Options for controlling static site generation.
///
/// Specifies where to write output files and whether to inject a
/// development-reload script pointing at a given endpoint.
#[derive(Clone, Debug)]
pub struct RenderOptions {
    /// Directory into which all rendered HTML, CSS, and assets are written.
    pub output_dir: PathBuf,
    /// When `Some(url)`, a `<script>` tag for live reload is appended to
    /// every page.  The script polls `url` and reloads the page when the
    /// version changes.
    pub dev_reload_endpoint: Option<String>,
}

impl RenderOptions {
    #[must_use]
    pub fn new(output_dir: impl Into<PathBuf>) -> Self {
        Self {
            output_dir: output_dir.into(),
            dev_reload_endpoint: None,
        }
    }

    #[must_use]
    pub fn with_dev_reload(mut self, endpoint: impl Into<String>) -> Self {
        self.dev_reload_endpoint = Some(endpoint.into());
        self
    }
}

/// Errors that can occur during static site rendering.
#[derive(Debug, Error)]
pub enum RenderError {
    /// The site configuration failed validation.
    #[error("site diagnostics failed: {0:?}")]
    Diagnostics(DiagnosticReport),
    /// A filesystem read or write operation failed.
    #[error("I/O error at {path}: {source}")]
    Io {
        /// The path that caused the I/O error.
        path: PathBuf,
        /// The underlying I/O error.
        source: std::io::Error,
    },
}

/// Renders the entire site to static HTML files.
///
/// Validates the site first, then writes every page as `{slug}/index.html`
/// (or `index.html` for the home page), copies all static assets, and
/// generates `style.css` from the site's theme.
pub fn render_static(site: &ProjectSite, options: &RenderOptions) -> Result<(), RenderError> {
    assert_valid(site).map_err(RenderError::Diagnostics)?;
    create_clean_dir(&options.output_dir)?;

    for asset in &site.assets {
        let target = options
            .output_dir
            .join(asset.target.trim_start_matches('/'));
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|source| RenderError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        fs::copy(&asset.source, &target).map_err(|source| RenderError::Io {
            path: target,
            source,
        })?;
    }

    write_file(&options.output_dir.join("style.css"), &stylesheet(site))?;

    for page in &site.pages {
        let html = render_page(site, page, options);
        let page_path = if page.slug == "index" {
            options.output_dir.join("index.html")
        } else {
            options.output_dir.join(&page.slug).join("index.html")
        };
        write_file(&page_path, &html)?;
    }

    Ok(())
}

fn create_clean_dir(path: &Path) -> Result<(), RenderError> {
    if path.exists() {
        fs::remove_dir_all(path).map_err(|source| RenderError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    }
    fs::create_dir_all(path).map_err(|source| RenderError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn write_file(path: &Path, contents: &str) -> Result<(), RenderError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| RenderError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    fs::write(path, contents).map_err(|source| RenderError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn render_page(site: &ProjectSite, page: &Page, options: &RenderOptions) -> String {
    let leptos_marker = view! { <meta name="generator" content="plinth-project"/> }.to_html();
    let json_ld = primary_person(site).map_or_else(String::new, |person| {
        format!(
            "<script type=\"application/ld+json\">{{\"@context\":\"https://schema.org\",\"@type\":\"WebSite\",\"name\":\"{}\",\"author\":{{\"@type\":\"Person\",\"name\":\"{}\",\"url\":\"{}\"}}}}</script>",
            escape_json(&site.title),
            escape_json(&person.name),
            escape_json(&person.url),
        )
    });

    let mut body = String::new();
    body.push_str(&render_nav(site));
    body.push_str("<main>");
    for section in &page.sections {
        body.push_str(&render_section(site, section));
    }
    body.push_str("</main>");
    body.push_str(&render_footer(site));
    body.push_str(interaction_script());
    if let Some(endpoint) = &options.dev_reload_endpoint {
        body.push_str(&dev_reload_script(endpoint));
    }

    format!(
        "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>{} - {}</title><meta name=\"description\" content=\"{}\">{}{}<link rel=\"stylesheet\" href=\"/style.css\"></head><body>{}</body></html>",
        escape_text(&page.title),
        escape_text(&site.title),
        escape_attr(if page.description.is_empty() {
            &site.description
        } else {
            &page.description
        }),
        leptos_marker,
        json_ld,
        body
    )
}

fn render_nav(site: &ProjectSite) -> String {
    let links = site
        .nav
        .iter()
        .map(|link| {
            format!(
                "<a href=\"{}\">{}</a>",
                escape_attr(&link.href),
                escape_text(&link.label)
            )
        })
        .collect::<String>();
    format!(
        "<nav class=\"top-nav\"><div class=\"nav-inner\"><a href=\"{}\" class=\"nav-brand\">{}</a><div class=\"nav-links\">{}</div></div></nav>",
        escape_attr(if site.base_url.is_empty() {
            "/"
        } else {
            &site.base_url
        }),
        escape_text(&site.title),
        links
    )
}

fn render_footer(site: &ProjectSite) -> String {
    let links = site
        .footer_links
        .iter()
        .map(|link| {
            format!(
                "<a href=\"{}\">{}</a>",
                escape_attr(&link.href),
                escape_text(&link.label)
            )
        })
        .collect::<String>();
    let attribution = primary_person(site).map_or_else(String::new, |person| {
        format!(
            "<p class=\"person-attribution\">Maintained by <a href=\"{}\"{}>{}</a></p>",
            escape_attr(&person.url),
            external_attrs(&person.url),
            escape_text(&person.name)
        )
    });
    format!(
        "<footer class=\"site-footer\"><div class=\"footer-inner\"><div><p>{}</p>{}</div><div class=\"footer-links\">{}</div></div></footer>",
        escape_text(&site.footer_note),
        attribution,
        links
    )
}

fn render_section(_site: &ProjectSite, section: &ProjectSection) -> String {
    #[allow(unreachable_patterns)]
    match section {
        #[cfg(feature = "brick-hero")]
        ProjectSection::Hero(hero) => crate::bricks::hero::render::render_hero(
            hero,
            hero.person
                .as_deref()
                .and_then(|id| person_by_id(_site, id)),
        ),
        #[cfg(feature = "brick-feature-grid")]
        ProjectSection::FeatureGrid(grid) => {
            crate::bricks::feature_grid::render::render_feature_grid(grid)
        }
        #[cfg(feature = "brick-install")]
        ProjectSection::Install(install) => crate::bricks::install::render::render_install(install),
        #[cfg(feature = "brick-screenshot-grid")]
        ProjectSection::ScreenshotGrid(grid) => {
            crate::bricks::screenshot_grid::render::render_screenshots(grid)
        }
        #[cfg(feature = "brick-capability-matrix")]
        ProjectSection::CapabilityMatrix(matrix) => {
            crate::bricks::capability_matrix::render::render_capability_matrix(matrix)
        }
        #[cfg(feature = "brick-comparison")]
        ProjectSection::Comparison(comparison) => {
            crate::bricks::comparison::render::render_comparison(comparison)
        }
        #[cfg(feature = "brick-person-mention")]
        ProjectSection::PersonMention(mention) => {
            crate::bricks::person_mention::render::render_person_mention(mention)
        }
        #[cfg(feature = "brick-workflow-steps")]
        ProjectSection::WorkflowSteps(workflow) => {
            crate::bricks::workflow_steps::render::render_workflow_steps(workflow)
        }
        #[cfg(feature = "brick-audience-grid")]
        ProjectSection::AudienceGrid(grid) => {
            crate::bricks::audience_grid::render::render_audience_grid(grid)
        }
        #[cfg(feature = "brick-trust-panel")]
        ProjectSection::TrustPanel(panel) => {
            crate::bricks::trust_panel::render::render_trust_panel(panel)
        }
        #[cfg(feature = "brick-content")]
        ProjectSection::Content(content) => crate::bricks::content::render::render_content(content),
        #[cfg(feature = "brick-custom")]
        ProjectSection::Custom(custom) => (custom.render)(),
        _ => String::new(),
    }
}

fn primary_person(site: &ProjectSite) -> Option<&plinth_person::PersonReference> {
    site.primary_person
        .as_deref()
        .and_then(|id| person_by_id(site, id))
}

fn person_by_id<'a>(site: &'a ProjectSite, id: &str) -> Option<&'a plinth_person::PersonReference> {
    site.people.iter().find(|person| person.id == id)
}

fn interaction_script() -> &'static str {
    r#"<script>
document.addEventListener('click', async event => {
  const button = event.target.closest('[data-copy]');
  if (!button) return;
  try {
    await navigator.clipboard.writeText(button.dataset.copy);
    button.textContent = 'Copied';
    setTimeout(() => { button.textContent = 'Copy'; }, 1200);
  } catch (_) {
    button.textContent = 'Select';
  }
});

(() => {
  let lastTrigger = null;
  const dialog = document.createElement('div');
  dialog.className = 'image-lightbox';
  dialog.setAttribute('role', 'dialog');
  dialog.setAttribute('aria-modal', 'true');
  dialog.setAttribute('aria-hidden', 'true');
  dialog.innerHTML = '<div class=\"image-lightbox-panel\" role=\"document\"><button type=\"button\" class=\"image-lightbox-close\" aria-label=\"Close enlarged image\">Close</button><img alt=\"\"><p></p></div>';
  document.body.appendChild(dialog);

  const image = dialog.querySelector('img');
  const caption = dialog.querySelector('p');
  const closeButton = dialog.querySelector('button');

  function closeLightbox() {
    dialog.classList.remove('open');
    dialog.setAttribute('aria-hidden', 'true');
    image.removeAttribute('src');
    document.body.classList.remove('lightbox-open');
    if (lastTrigger) {
      lastTrigger.focus();
    }
  }

  document.addEventListener('click', event => {
    const trigger = event.target.closest('[data-lightbox-image]');
    if (!trigger) return;
    lastTrigger = trigger;
    image.src = trigger.dataset.lightboxImage;
    image.alt = trigger.dataset.lightboxAlt || '';
    caption.textContent = trigger.dataset.lightboxCaption || trigger.dataset.lightboxAlt || '';
    dialog.classList.add('open');
    dialog.setAttribute('aria-hidden', 'false');
    document.body.classList.add('lightbox-open');
    closeButton.focus();
  });

  closeButton.addEventListener('click', closeLightbox);
  dialog.addEventListener('click', event => {
    if (event.target === dialog) closeLightbox();
  });
  document.addEventListener('keydown', event => {
    if (event.key === 'Escape' && dialog.classList.contains('open')) {
      closeLightbox();
    }
  });
})();
</script>"#
}

fn dev_reload_script(endpoint: &str) -> String {
    format!(
        r#"<script>
(() => {{
  const endpoint = "{}";
  let version = null;
  async function poll() {{
    try {{
      const response = await fetch(endpoint, {{ cache: 'no-store' }});
      if (!response.ok) return;
      const next = await response.text();
      if (version === null) {{
        version = next;
      }} else if (version !== next) {{
        window.location.reload();
      }}
    }} catch (_) {{}}
  }}
  setInterval(poll, 700);
  poll();
}})();
</script>"#,
        escape_attr(endpoint)
    )
}

/// Escapes a string for safe embedding inside a JSON string literal.
///
/// Replaces backslash, double-quote, and newline characters.
fn escape_json(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

/// Builds the complete stylesheet string for the site.
///
/// Prepends CSS custom-property theme variables from the site's theme
/// configuration to the content of `style.css`.
fn stylesheet(site: &ProjectSite) -> String {
    format!(
        "{}\n{}",
        theme_styles(&site.theme),
        include_str!("../style.css")
    )
}

fn theme_styles(theme: &ProjectTheme) -> String {
    let mut css = String::from(":root{");
    push_var(&mut css, "--pp-paper", &theme.paper);
    push_var(&mut css, "--pp-surface", &theme.surface);
    push_var(&mut css, "--pp-ink", &theme.ink);
    push_var(&mut css, "--pp-ink-soft", &theme.ink_soft);
    push_var(&mut css, "--pp-line", &theme.line);
    push_var(&mut css, "--pp-accent", &theme.accent);
    push_var(&mut css, "--pp-accent-soft", &theme.accent_soft);
    push_var(&mut css, "--pp-secondary", &theme.secondary);
    push_var(&mut css, "--pp-warning", &theme.warning);
    push_var(&mut css, "--pp-rust", &theme.rust);
    css.push('}');
    css
}

fn push_var(css: &mut String, name: &str, value: &Option<String>) {
    if let Some(value) = value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        css.push_str(name);
        css.push(':');
        css.push_str(&escape_css_value(value));
        css.push(';');
    }
}

fn escape_css_value(value: &str) -> String {
    value
        .chars()
        .filter(|ch| !matches!(ch, '{' | '}' | ';' | '<' | '>' | '\\'))
        .collect()
}
