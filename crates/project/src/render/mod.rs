#![allow(clippy::format_collect)]

use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::{DiagnosticReport, Page, ProjectSection, ProjectSite, ProjectTheme, assert_valid};

mod html;
#[cfg(test)]
mod tests;

/// Output file name for the copied nix-theme-broker stylesheet.
const BROKER_CSS_FILE: &str = "broker-theme.css";

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

    if let Some(source) = &site.theme.broker_css {
        let target = options.output_dir.join(BROKER_CSS_FILE);
        fs::copy(source, &target).map_err(|error| RenderError::Io {
            path: source.clone(),
            source: error,
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

    #[cfg(feature = "brick-blog")]
    write_blog(site, options)?;

    write_machine_files(site, &options.output_dir)?;

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

/// Canonical domain with whitespace trimmed, or `None` when unset/blank.
fn canonical_domain(site: &ProjectSite) -> Option<&str> {
    site.canonical_domain
        .as_deref()
        .map(str::trim)
        .filter(|domain| !domain.is_empty())
}

/// Absolute origin for machine-readable files: the canonical domain when set,
/// otherwise an absolute `base_url`. Falls back to path-only URLs.
fn site_origin(site: &ProjectSite) -> Option<String> {
    if let Some(domain) = canonical_domain(site) {
        return Some(format!("https://{}", domain.trim_matches('/')));
    }
    let base = site.base_url.trim();
    if base.starts_with("http://") || base.starts_with("https://") {
        return Some(base.trim_end_matches('/').to_owned());
    }
    None
}

fn page_url(site: &ProjectSite, slug: &str) -> String {
    let path = if slug == "index" {
        "/".to_owned()
    } else {
        format!("/{slug}/")
    };
    match site_origin(site) {
        Some(origin) => format!("{origin}{path}"),
        None => path,
    }
}

/// Machine-readable companions to the HTML pages: sitemap, robots, llms.txt,
/// a projects JSON index, and the Pages `CNAME` file when a canonical domain
/// is configured.
fn write_machine_files(site: &ProjectSite, out: &Path) -> Result<(), RenderError> {
    let mut sitemap = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );
    for page in &site.pages {
        sitemap.push_str(&format!(
            "  <url><loc>{}</loc></url>\n",
            escape_attr(&page_url(site, &page.slug))
        ));
    }
    #[cfg(feature = "brick-blog")]
    if let Some(blog) = published_blog(site) {
        sitemap.push_str(&format!(
            "  <url><loc>{}</loc></url>\n",
            escape_attr(&page_url(site, "blog"))
        ));
        for post in &blog.posts {
            sitemap.push_str(&format!(
                "  <url><loc>{}</loc></url>\n",
                escape_attr(&page_url(site, &format!("blog/{}", post.slug)))
            ));
        }
    }
    sitemap.push_str("</urlset>\n");
    write_file(&out.join("sitemap.xml"), &sitemap)?;

    let mut robots = String::from("User-agent: *\nAllow: /\n");
    if let Some(origin) = site_origin(site) {
        robots.push_str(&format!("Sitemap: {origin}/sitemap.xml\n"));
    }
    write_file(&out.join("robots.txt"), &robots)?;

    let mut llms = format!("# {}\n\n> {}\n", site.title, one_line(&site.description));
    llms.push_str("\n## Pages\n");
    for page in &site.pages {
        let description = if page.description.is_empty() {
            &site.description
        } else {
            &page.description
        };
        llms.push_str(&format!(
            "\n- [{}]({}): {}",
            page.title,
            page_url(site, &page.slug),
            one_line(description)
        ));
    }
    #[cfg(feature = "brick-blog")]
    if let Some(blog) = published_blog(site) {
        llms.push_str(&format!("\n\n## {}", blog.title));
        for post in &blog.posts {
            let description = one_line(&post.description);
            llms.push_str(&format!(
                "\n- [{}]({})",
                post.title,
                page_url(site, &format!("blog/{}", post.slug))
            ));
            if !description.is_empty() {
                llms.push_str(&format!(": {description}"));
            }
        }
    }
    if !site.projects.is_empty() {
        llms.push_str("\n\n## Projects");
        for project in &site.projects {
            let mut line = format!("\n- [{}]({})", project.title, project.url);
            if let Some(description) = project
                .description
                .as_deref()
                .map(one_line)
                .filter(|description| !description.is_empty())
            {
                line.push_str(&format!(": {description}"));
            }
            let mut extra = Vec::new();
            if let Some(source) = project
                .source_url
                .as_deref()
                .filter(|source| !source.is_empty())
            {
                extra.push(format!("Source: {source}"));
            }
            if let Some(demo) = project.demo_url.as_deref().filter(|demo| !demo.is_empty()) {
                extra.push(format!("Demo: {demo}"));
            }
            for link in &project.links {
                extra.push(format!("{}: {}", link.label, link.href));
            }
            if !extra.is_empty() {
                line.push_str(&format!(" ({})", extra.join("; ")));
            }
            llms.push_str(&line);
        }
    }
    llms.push('\n');
    write_file(&out.join("llms.txt"), &llms)?;

    let projects =
        serde_json::to_string_pretty(&site.projects).map_err(|source| RenderError::Io {
            path: out.join("projects.json"),
            source: std::io::Error::other(source),
        })?;
    write_file(&out.join("projects.json"), &(projects + "\n"))?;

    if let Some(domain) = canonical_domain(site) {
        write_file(&out.join("CNAME"), &format!("{domain}\n"))?;
    }

    Ok(())
}

fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn render_page(site: &ProjectSite, page: &Page, options: &RenderOptions) -> String {
    // Keep the generator marker framework-neutral: project sites are static
    // output and no longer depend on the retired Leptos renderer.
    let generator_marker = "<meta name=\"generator\" content=\"plinth-project\">";
    let json_ld = primary_person(site).map_or_else(String::new, |person| {
        format!(
            "<script type=\"application/ld+json\">{{\"@context\":\"https://schema.org\",\"@type\":\"WebSite\",\"name\":\"{}\",\"author\":{{\"@type\":\"Person\",\"name\":\"{}\",\"url\":\"{}\"}}}}</script>",
            escape_json(&site.title),
            escape_json(&person.name),
            escape_json(&person.url),
        )
    });

    let broker = site.theme.broker_css.is_some();
    let mut body = String::new();
    body.push_str(&render_nav(site));
    body.push_str("<main>");
    for section in &page.sections {
        body.push_str(&render_section(site, section));
    }
    body.push_str("</main>");
    body.push_str(&render_footer(site));
    body.push_str(interaction_script());
    if broker {
        body.push_str(THEME_TOGGLE_SCRIPT);
    }
    if let Some(endpoint) = &options.dev_reload_endpoint {
        body.push_str(&dev_reload_script(endpoint));
    }

    let canonical = canonical_domain(site).map_or_else(String::new, |_| {
        format!(
            "<link rel=\"canonical\" href=\"{}\">",
            escape_attr(&page_url(site, &page.slug))
        )
    });

    let description = if page.description.is_empty() {
        &site.description
    } else {
        &page.description
    };
    let favicon = site.favicon.as_deref().map_or_else(String::new, |href| {
        let svg = if href.ends_with(".svg") {
            " type=\"image/svg+xml\""
        } else {
            ""
        };
        format!("<link rel=\"icon\" href=\"{}\"{svg}>", escape_attr(href))
    });
    let social = site.og_image.as_deref().map_or_else(String::new, |image| {
        let domain = canonical_domain(site);
        let image = match domain {
            Some(domain) if image.starts_with('/') && !image.starts_with("//") => {
                format!("https://{}{image}", domain.trim_matches('/'))
            }
            _ => image.to_owned(),
        };
        let url = domain.map_or_else(String::new, |_| {
            format!(
                "<meta property=\"og:url\" content=\"{}\">",
                escape_attr(&page_url(site, &page.slug))
            )
        });
        format!(
            "<meta property=\"og:title\" content=\"{}\"><meta property=\"og:description\" content=\"{}\"><meta property=\"og:type\" content=\"website\">{url}<meta property=\"og:image\" content=\"{}\"><meta name=\"twitter:card\" content=\"summary_large_image\">",
            escape_attr(&page.title),
            escape_attr(description),
            escape_attr(&image),
        )
    });
    let stylesheets = site
        .stylesheets
        .iter()
        .map(|href| format!("<link rel=\"stylesheet\" href=\"{}\">", escape_attr(href)))
        .collect::<String>();

    format!(
        "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>{} - {}</title><meta name=\"description\" content=\"{}\">{}{}{}{favicon}{social}{}<link rel=\"stylesheet\" href=\"/style.css\">{stylesheets}</head><body{}>{}</body></html>",
        escape_text(&page.title),
        escape_text(&site.title),
        escape_attr(description),
        generator_marker,
        json_ld,
        canonical,
        if broker { BROKER_HEAD } else { "" },
        if broker { " class=\"pp-minimal\"" } else { "" },
        body
    )
}

fn render_nav(site: &ProjectSite) -> String {
    let links = site
        .nav
        .iter()
        .map(|link| {
            format!(
                "<a href=\"{}\"{}>{}</a>",
                escape_attr(&link.href),
                if link.primary {
                    " class=\"nav-primary\""
                } else {
                    ""
                },
                escape_text(&link.label)
            )
        })
        .collect::<String>();
    let toggle = if site.theme.broker_css.is_some() {
        THEME_TOGGLE_BUTTON
    } else {
        ""
    };
    format!(
        "<nav class=\"top-nav\"><div class=\"nav-inner\"><a href=\"{}\" class=\"nav-brand\">{}</a><div class=\"nav-links\">{}{}</div></div></nav>",
        escape_attr(if site.base_url.is_empty() {
            "/"
        } else {
            &site.base_url
        }),
        escape_text(&site.title),
        links,
        toggle
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
        #[cfg(feature = "brick-project-grid")]
        ProjectSection::ProjectGrid(grid) => {
            crate::bricks::project_grid::render::render_project_grid(grid, &_site.projects)
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
    const still = trigger.dataset.lightboxStill;
    image.src = still && window.matchMedia('(prefers-reduced-motion: reduce)').matches ? still : trigger.dataset.lightboxImage;
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
        "@layer base, tartan-layout, layout, components, prose, minimal;\n{}\n{}\n{}{}",
        theme_styles(&site.theme),
        tartan_ui_assets::LAYOUT_STYLES,
        include_str!("../style.css"),
        blog_styles(site)
    )
}

/// Blog stylesheet fragment, or the empty string when the site has no
/// published posts. Sites without `[blog]` keep their byte-identical output.
fn blog_styles(_site: &ProjectSite) -> &'static str {
    #[cfg(feature = "brick-blog")]
    if published_blog(_site).is_some() {
        return include_str!("../bricks/blog/style.css");
    }
    ""
}

fn theme_styles(theme: &ProjectTheme) -> String {
    let mut css = String::from(":root{");
    if let Some(family) = &theme.font_family {
        css.push_str(&format!(
            "--pp-font-family:\"{}\";",
            escape_css_string(family)
        ));
    }
    let broker = theme.broker_css.is_some();
    // The broker owns the muted accent role; the configured soft colour (or the
    // derived tint) is only the fallback for an older broker stylesheet.
    let accent_soft = theme
        .accent_soft
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map_or_else(
            || "color-mix(in srgb,var(--pp-accent) 16%,transparent)".to_owned(),
            str::to_owned,
        );
    // With a broker stylesheet each `--pp-*` aliases a `--tb-*` role and the
    // configured colour (or preset) is the fallback, so a missing broker file
    // degrades to the pre-broker look.
    for (name, value, role) in [
        ("--pp-paper", &theme.paper, "--tb-ui-background"),
        ("--pp-surface", &theme.surface, "--tb-ui-surface"),
        ("--pp-ink", &theme.ink, "--tb-ui-foreground"),
        ("--pp-ink-soft", &theme.ink_soft, "--tb-ui-foreground-muted"),
        ("--pp-line", &theme.line, "--tb-ui-border"),
        ("--pp-accent", &theme.accent, "--tb-ui-accent"),
        (
            "--pp-accent-soft",
            &theme.accent_soft,
            "--tb-ui-accent-muted",
        ),
        ("--pp-secondary", &theme.secondary, "--tb-status-success"),
        ("--pp-warning", &theme.warning, "--tb-status-warning"),
        ("--pp-rust", &theme.rust, "--tb-status-error"),
    ] {
        if !broker {
            push_var(&mut css, name, value);
        } else if name == "--pp-accent-soft" {
            // The broker owns the muted accent role; the configured soft colour
            // (or the derived tint) is only the fallback.
            push_alias(&mut css, name, role, &Some(accent_soft.clone()));
        } else {
            push_alias(&mut css, name, role, value);
        }
    }
    if broker {
        // UI state roles are optional: newer broker variants add them, and an
        // older stylesheet falls back to the pre-existing semantic colours.
        for (name, role, fallback) in [
            (
                "--pp-accent-hover",
                "--tb-ui-accent-hover",
                // Optional like ink-bright: link and button use sites keep their own
                // pre-broker hover fallbacks.
                "",
            ),
            (
                "--pp-accent-active",
                "--tb-ui-accent-active",
                "var(--pp-accent-hover)",
            ),
            ("--pp-on-accent", "--tb-ui-on-accent", "var(--pp-paper)"),
            ("--pp-focus", "--tb-ui-focus", "var(--pp-accent)"),
            // Optional: each use site keeps its own fallback (accent in base,
            // inherited ink in minimal).
            ("--pp-link", "--tb-ui-link", ""),
            (
                "--pp-selection-background",
                "--tb-ui-selection-background",
                "var(--pp-accent)",
            ),
            (
                "--pp-selection-foreground",
                "--tb-ui-selection-foreground",
                "var(--pp-paper)",
            ),
            ("--pp-panel", "--tb-ui-background-alt", "var(--pp-paper)"),
            (
                "--pp-surface-alt",
                "--tb-ui-surface-alt",
                "var(--pp-surface)",
            ),
            (
                "--pp-ink-subtle",
                "--tb-ui-foreground-subtle",
                "var(--pp-ink-soft)",
            ),
            (
                "--pp-ink-bright",
                "--tb-ui-foreground-bright",
                // Each CSS use site retains its own pre-broker fallback.
                "",
            ),
            (
                "--pp-line-subtle",
                "--tb-ui-border-subtle",
                "var(--pp-line)",
            ),
        ] {
            push_alias_expr(&mut css, name, role, fallback);
        }
    }
    css.push('}');
    if let (Some(family), Some(url)) = (&theme.font_family, &theme.font_url) {
        css.push_str(&format!(
            "@font-face{{font-family:\"{}\";src:url(\"{}\") format(\"woff2\");font-weight:100 900;font-style:normal;font-display:swap;}}",
            escape_css_string(family), escape_css_string(url),
        ));
    }
    css
}

fn escape_css_string(value: &str) -> String {
    value
        .chars()
        .filter(|ch| !ch.is_control())
        .flat_map(|ch| {
            if matches!(ch, '\\' | '"') {
                vec!['\\', ch]
            } else {
                vec![ch]
            }
        })
        .collect()
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

fn push_alias(css: &mut String, name: &str, role: &str, fallback: &Option<String>) {
    let fallback = fallback
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or_default();
    push_alias_expr(css, name, role, fallback);
}

fn push_alias_expr(css: &mut String, name: &str, role: &str, fallback: &str) {
    css.push_str(name);
    css.push_str(":var(");
    css.push_str(role);
    if !fallback.is_empty() {
        css.push(',');
        css.push_str(&escape_css_value(fallback));
    }
    css.push_str(");");
}

fn escape_css_value(value: &str) -> String {
    value
        .chars()
        .filter(|ch| !matches!(ch, '{' | '}' | ';' | '<' | '>' | '\\'))
        .collect()
}

/// Head fragment for broker sites: the broker stylesheet (before `style.css`)
/// and a tiny script that applies a stored light/dark choice before first
/// paint. `system` stores nothing, leaving the `prefers-color-scheme` media
/// query in charge.
const BROKER_HEAD: &str = concat!(
    "<link rel=\"stylesheet\" href=\"/broker-theme.css\">",
    "<script>try{var t=localStorage.getItem('tartan-theme');",
    "if(t==='light'||t==='dark')document.documentElement.dataset.theme=t}catch(e){}</script>"
);

/// Segmented system / light / dark control with Lucide icons (ISC licence,
/// 16px, 1.5 stroke, `currentColor`). Hidden until the script below enables
/// it, so there is no inert control without JS.
const THEME_TOGGLE_BUTTON: &str = concat!(
    "<div class=\"theme-toggle\" role=\"group\" aria-label=\"Theme\" data-theme-toggle hidden>",
    "<button type=\"button\" data-theme-mode=\"system\" aria-label=\"System theme\" title=\"System\" aria-pressed=\"false\">",
    "<svg viewBox=\"0 0 24 24\" width=\"16\" height=\"16\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.5\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><rect width=\"20\" height=\"14\" x=\"2\" y=\"3\" rx=\"2\"/><path d=\"M8 21h8M12 17v4\"/></svg></button>",
    "<button type=\"button\" data-theme-mode=\"light\" aria-label=\"Light theme\" title=\"Light\" aria-pressed=\"false\">",
    "<svg viewBox=\"0 0 24 24\" width=\"16\" height=\"16\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.5\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><circle cx=\"12\" cy=\"12\" r=\"4\"/><path d=\"M12 2v2M12 20v2m-7.07-17.07 1.41 1.41m11.32 11.32 1.41 1.41M2 12h2m16 0h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41\"/></svg></button>",
    "<button type=\"button\" data-theme-mode=\"dark\" aria-label=\"Dark theme\" title=\"Dark\" aria-pressed=\"false\">",
    "<svg viewBox=\"0 0 24 24\" width=\"16\" height=\"16\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"1.5\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z\"/></svg></button>",
    "</div>"
);

/// Applies the chosen mode on `<html data-theme>` (none for `system`),
/// persists it under the same `tartan-theme` key as tartan-ui and marks the
/// active button with `aria-pressed`.
const THEME_TOGGLE_SCRIPT: &str = r#"<script>
(() => {
  const group = document.querySelector('[data-theme-toggle]');
  if (!group) return;
  const buttons = group.querySelectorAll('[data-theme-mode]');
  const modes = ['system', 'light', 'dark'];
  let mode = 'system';
  try { mode = localStorage.getItem('tartan-theme') || 'system'; } catch (_) {}
  if (!modes.includes(mode)) mode = 'system';
  const apply = () => {
    if (mode === 'system') document.documentElement.removeAttribute('data-theme');
    else document.documentElement.dataset.theme = mode;
    buttons.forEach((b) => b.setAttribute('aria-pressed', String(b.dataset.themeMode === mode)));
  };
  group.hidden = false;
  apply();
  buttons.forEach((b) => b.addEventListener('click', () => {
    mode = b.dataset.themeMode;
    try { localStorage.setItem('tartan-theme', mode); } catch (_) {}
    apply();
  }));
})();
</script>"#;

/// The blog when it has at least one published post.
#[cfg(feature = "brick-blog")]
fn published_blog(site: &ProjectSite) -> Option<&crate::bricks::blog::Blog> {
    site.blog.as_ref().filter(|blog| !blog.posts.is_empty())
}

/// Writes `blog/index.html`, one page per published post, and the RSS feed.
///
/// Unpublished posts never reach the output: the loader drops them, and a
/// blog without published posts emits nothing at all.
#[cfg(feature = "brick-blog")]
fn write_blog(site: &ProjectSite, options: &RenderOptions) -> Result<(), RenderError> {
    let Some(blog) = published_blog(site) else {
        return Ok(());
    };
    write_file(
        &options.output_dir.join("blog/index.html"),
        &render_blog_page(
            site,
            &blog.title,
            &site.description,
            &crate::bricks::blog::render::render_index(blog),
            "blog",
            options,
        ),
    )?;
    for post in &blog.posts {
        write_file(
            &options
                .output_dir
                .join("blog")
                .join(&post.slug)
                .join("index.html"),
            &render_blog_page(
                site,
                &post.title,
                &post.description,
                &crate::bricks::blog::render::render_post(blog, post),
                &format!("blog/{}", post.slug),
                options,
            ),
        )?;
    }
    if let Some(feed) = crate::bricks::blog::render::render_feed(site, blog) {
        write_file(&options.output_dir.join("blog/feed.xml"), &feed)?;
    }
    Ok(())
}

/// Renders a blog document inside the site's usual chrome, theme, and nav.
#[cfg(feature = "brick-blog")]
fn render_blog_page(
    site: &ProjectSite,
    title: &str,
    description: &str,
    body: &str,
    slug: &str,
    options: &RenderOptions,
) -> String {
    let description = if description.is_empty() {
        &site.description
    } else {
        description
    };

    let mut shell = String::new();
    shell.push_str(&render_nav(site));
    shell.push_str("<main>");
    shell.push_str(body);
    shell.push_str("</main>");
    shell.push_str(&render_footer(site));
    shell.push_str(interaction_script());
    if let Some(endpoint) = &options.dev_reload_endpoint {
        shell.push_str(&dev_reload_script(endpoint));
    }
    let canonical = canonical_domain(site).map_or_else(String::new, |_| {
        format!(
            "<link rel=\"canonical\" href=\"{}\">",
            escape_attr(&page_url(site, slug))
        )
    });
    format!(
        "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>{} - {}</title><meta name=\"description\" content=\"{}\"><meta name=\"generator\" content=\"plinth-project\">{}<link rel=\"stylesheet\" href=\"/style.css\"></head><body>{shell}</body></html>",
        escape_text(title),
        escape_text(&site.title),
        escape_attr(description),
        canonical,
    )
}
