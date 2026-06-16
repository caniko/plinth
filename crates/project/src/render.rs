#![allow(clippy::format_collect)]

use std::fs;
use std::path::{Path, PathBuf};

use leptos::prelude::*;
use thiserror::Error;

use crate::{DiagnosticReport, Page, ProjectSection, ProjectSite, ProjectTheme, assert_valid};

#[derive(Clone, Debug)]
pub struct RenderOptions {
    pub output_dir: PathBuf,
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

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("site diagnostics failed: {0:?}")]
    Diagnostics(DiagnosticReport),
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
}

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

#[allow(dead_code)]
pub(crate) fn id_attr(id: Option<&str>) -> String {
    id.map_or_else(String::new, |id| format!(" id=\"{}\"", escape_attr(id)))
}

pub(crate) fn escape_text(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub(crate) fn escape_attr(input: &str) -> String {
    escape_text(input).replace('"', "&quot;")
}

pub(crate) fn external_attrs(href: &str) -> &'static str {
    if href.starts_with("http://") || href.starts_with("https://") {
        " target=\"_blank\" rel=\"noopener noreferrer\""
    } else {
        ""
    }
}

#[cfg(feature = "brick-person-mention")]
pub(crate) fn render_external_link(link: &plinth_person::ExternalLink) -> String {
    format!(
        "<a class=\"person-link link-{}\" href=\"{}\"{}>{}</a>",
        link_kind_class(&link.kind),
        escape_attr(&link.href),
        external_attrs(&link.href),
        escape_text(&link.label),
    )
}

#[cfg(feature = "brick-person-mention")]
fn link_kind_class(kind: &plinth_person::LinkKind) -> &'static str {
    match kind {
        plinth_person::LinkKind::Person => "person",
        plinth_person::LinkKind::ProjectSite => "project-site",
        plinth_person::LinkKind::Source => "source",
        plinth_person::LinkKind::Demo => "demo",
        plinth_person::LinkKind::Docs => "docs",
        plinth_person::LinkKind::Contact => "contact",
        plinth_person::LinkKind::Other => "other",
    }
}

fn escape_json(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn stylesheet(site: &ProjectSite) -> String {
    format!(
        "{}\n{}",
        theme_styles(&site.theme),
        include_str!("style.css")
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

#[cfg(test)]
mod tests {
    use crate::{Page, ProjectSite, ProjectTheme, RenderOptions, render_static};

    #[cfg(feature = "brick-custom")]
    use crate::CustomSection;

    #[cfg(all(feature = "brick-hero", feature = "brick-person-mention"))]
    use crate::{ExternalLink, Hero, LinkKind, PersonMention, PersonReference, ProjectSection};

    #[cfg(feature = "brick-screenshot-grid")]
    use crate::{Screenshot, ScreenshotGrid};

    #[cfg(all(
        feature = "brick-workflow-steps",
        feature = "brick-audience-grid",
        feature = "brick-trust-panel"
    ))]
    use crate::{Audience, AudienceGrid, TrustItem, TrustPanel, WorkflowStep, WorkflowSteps};

    #[cfg(feature = "brick-custom")]
    #[test]
    fn renders_custom_section() {
        let dir = tempfile::tempdir().unwrap();
        let site = ProjectSite::new("example", "example site").page(
            Page::new("index", "example").section(crate::ProjectSection::Custom(
                CustomSection::new(|| "<section id=\"custom\">custom slot</section>".into()),
            )),
        );

        render_static(&site, &RenderOptions::new(dir.path())).unwrap();
        let html = std::fs::read_to_string(dir.path().join("index.html")).unwrap();
        assert!(html.contains("custom slot"));
        assert!(html.contains("plinth-project"));
    }

    #[test]
    fn dev_reload_script_is_opt_in() {
        let dir = tempfile::tempdir().unwrap();
        let site = ProjectSite::new("example", "example site").page(Page::new("index", "example"));

        render_static(&site, &RenderOptions::new(dir.path())).unwrap();
        let html = std::fs::read_to_string(dir.path().join("index.html")).unwrap();
        assert!(!html.contains("__plinth_project_reload"));

        render_static(
            &site,
            &RenderOptions::new(dir.path()).with_dev_reload("/__plinth_project_reload"),
        )
        .unwrap();
        let html = std::fs::read_to_string(dir.path().join("index.html")).unwrap();
        assert!(html.contains("__plinth_project_reload"));
    }

    #[test]
    fn renders_theme_css_variables_when_configured() {
        let dir = tempfile::tempdir().unwrap();
        let site = ProjectSite {
            theme: ProjectTheme {
                paper: Some("#fbf8f4".into()),
                ink: Some("#2a2724".into()),
                accent: Some("#c9a0a6".into()),
                ..ProjectTheme::default()
            },
            ..ProjectSite::new("example", "example site")
        }
        .page(Page::new("index", "example"));

        render_static(&site, &RenderOptions::new(dir.path())).unwrap();
        let css = std::fs::read_to_string(dir.path().join("style.css")).unwrap();
        assert!(css.contains("--pp-paper:#fbf8f4"));
        assert!(css.contains("--pp-ink:#2a2724"));
        assert!(css.contains("--pp-accent:#c9a0a6"));
        assert!(css.contains("var(--pp-paper"));
    }

    #[cfg(feature = "brick-screenshot-grid")]
    #[test]
    fn screenshot_grid_images_are_lightbox_triggers() {
        let dir = tempfile::tempdir().unwrap();
        let site = ProjectSite::new("example", "example site").page(
            Page::new("index", "example").section(crate::ProjectSection::ScreenshotGrid(
                ScreenshotGrid {
                    id: "shots".into(),
                    heading: "Screenshots".into(),
                    intro: "Generated from the app.".into(),
                    screenshots: vec![Screenshot {
                        src: "/screenshots/main.png".into(),
                        alt: "Main app view".into(),
                        caption: "Main view".into(),
                    }],
                },
            )),
        );

        render_static(&site, &RenderOptions::new(dir.path())).unwrap();
        let html = std::fs::read_to_string(dir.path().join("index.html")).unwrap();
        assert!(html.contains("data-lightbox-image=\"/screenshots/main.png\""));
        assert!(html.contains("data-lightbox-caption=\"Main view\""));
        assert!(html.contains("image-lightbox"));
        assert!(html.matches("class=\\\"image-lightbox\\\"").count() <= 1);
    }

    #[cfg(all(
        feature = "brick-hero",
        feature = "brick-person-mention",
        feature = "brick-screenshot-grid"
    ))]
    #[test]
    fn identity_images_do_not_become_lightbox_triggers() {
        let dir = tempfile::tempdir().unwrap();
        let person = PersonReference {
            id: "maintainer".into(),
            name: "Maintainer".into(),
            url: "https://person.example".into(),
            role: Some("Project lead".into()),
            avatar_url: Some("/avatar.png".into()),
            links: Vec::new(),
        };
        let site = ProjectSite::new("example", "example site").page(
            Page::new("index", "example")
                .section(ProjectSection::Hero(Hero {
                    logo_src: Some("/logo.svg".into()),
                    title: "Example".into(),
                    tagline: "Built plainly".into(),
                    subtitle: "A test site".into(),
                    person: None,
                    ctas: Vec::new(),
                }))
                .section(ProjectSection::PersonMention(PersonMention {
                    id: Some("maintainer".into()),
                    heading: "Maintainer".into(),
                    intro: "Who keeps this project moving.".into(),
                    person,
                })),
        );

        render_static(&site, &RenderOptions::new(dir.path())).unwrap();
        let html = std::fs::read_to_string(dir.path().join("index.html")).unwrap();
        assert!(html.contains("class=\"hero-logo\""));
        assert!(html.contains("class=\"person-avatar\""));
        assert!(!html.contains("data-lightbox-image=\"/logo.svg\""));
        assert!(!html.contains("data-lightbox-image=\"/avatar.png\""));
    }

    #[cfg(all(
        feature = "brick-workflow-steps",
        feature = "brick-audience-grid",
        feature = "brick-trust-panel"
    ))]
    #[test]
    fn renders_product_brick_markers() {
        let dir = tempfile::tempdir().unwrap();
        let site = ProjectSite::new("example", "example site").page(
            Page::new("index", "example")
                .section(crate::ProjectSection::WorkflowSteps(WorkflowSteps {
                    id: Some("flow".into()),
                    heading: "Workflow".into(),
                    intro: "How the work moves.".into(),
                    steps: vec![WorkflowStep {
                        title: "Discover".into(),
                        description: "Find relevant precedent records.".into(),
                    }],
                }))
                .section(crate::ProjectSection::AudienceGrid(AudienceGrid {
                    id: Some("roles".into()),
                    heading: "Roles".into(),
                    intro: "Who uses it.".into(),
                    audiences: vec![Audience {
                        label: "Curator".into(),
                        description: "Reviews and organizes records.".into(),
                    }],
                }))
                .section(crate::ProjectSection::TrustPanel(TrustPanel {
                    id: Some("trust".into()),
                    heading: "Trust".into(),
                    intro: "How safety stays visible.".into(),
                    items: vec![TrustItem {
                        title: "Rights visible".into(),
                        description: "Rights travel with each record.".into(),
                    }],
                })),
        );

        render_static(&site, &RenderOptions::new(dir.path())).unwrap();
        let html = std::fs::read_to_string(dir.path().join("index.html")).unwrap();
        assert!(html.contains("workflow-steps"));
        assert!(html.contains("audience-grid"));
        assert!(html.contains("trust-panel"));
    }

    #[cfg(all(feature = "brick-hero", feature = "brick-person-mention"))]
    #[test]
    fn renders_primary_person_links_and_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let person = PersonReference {
            id: "maintainer".into(),
            name: "Maintainer".into(),
            url: "https://person.example".into(),
            role: Some("Project lead".into()),
            avatar_url: None,
            links: vec![ExternalLink::new(
                "Contact",
                "https://person.example/contact",
                LinkKind::Contact,
            )],
        };
        let site = ProjectSite {
            primary_person: Some("maintainer".into()),
            people: vec![person.clone()],
            ..ProjectSite::new("example", "example site")
        }
        .page(
            Page::new("index", "example")
                .section(ProjectSection::Hero(Hero {
                    logo_src: None,
                    title: "Example".into(),
                    tagline: "Built plainly".into(),
                    subtitle: "A test site".into(),
                    person: Some("maintainer".into()),
                    ctas: Vec::new(),
                }))
                .section(ProjectSection::PersonMention(PersonMention {
                    id: Some("maintainer".into()),
                    heading: "Maintainer".into(),
                    intro: "Who keeps this project moving.".into(),
                    person,
                })),
        );

        render_static(&site, &RenderOptions::new(dir.path())).unwrap();
        let html = std::fs::read_to_string(dir.path().join("index.html")).unwrap();
        assert!(html.contains("hero-byline"));
        assert!(html.contains("Maintained by"));
        assert!(html.contains("application/ld+json"));
        assert!(html.contains("person-mention"));
        assert!(html.contains("link-contact"));
    }
}
