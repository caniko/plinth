use std::path::{Path, PathBuf};

use crate::config::types::{
    ConfigError, PageConfig, PersonConfig, ProjectConfig, ProjectReferenceConfig, SectionConfig,
    ThemeConfig,
};
use crate::{Asset, NavLink, Page, ProjectSection, ProjectSite, ProjectTheme};
use plinth_person::{ExternalLink, PersonReference, ProjectReference};

#[cfg(feature = "brick-content")]
use crate::bricks::content::config::build_content;

pub fn load_project_site(path: impl AsRef<Path>) -> Result<ProjectSite, ConfigError> {
    let path = path.as_ref();
    let config = load_project_config(path)?;
    let base = path.parent().unwrap_or_else(|| Path::new("."));
    build_site(config, base)
}

pub fn project_watch_paths(path: impl AsRef<Path>) -> Result<Vec<PathBuf>, ConfigError> {
    let path = path.as_ref();
    let config = load_project_config(path)?;
    let base = path.parent().unwrap_or_else(|| Path::new("."));
    let mut paths = vec![path.to_path_buf()];
    if let Some(parent) = path.parent() {
        paths.push(parent.to_path_buf());
    }

    for asset in &config.assets {
        push_watch_path(&mut paths, resolve_path(base, asset.source.clone()));
    }
    for page in &config.pages {
        #[allow(unused_variables)]
        for section in &page.sections {
            #[cfg(feature = "brick-capability-matrix")]
            if let SectionConfig::CapabilityMatrix { source, .. } = section {
                for path in crate::bricks::capability_matrix::config::watch_paths(base, source) {
                    push_watch_path(&mut paths, path);
                }
            }
        }
    }

    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn load_project_config(path: &Path) -> Result<ProjectConfig, ConfigError> {
    let raw = std::fs::read_to_string(path).map_err(|source| ConfigError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    toml::from_str::<ProjectConfig>(&raw).map_err(|source| ConfigError::Parse {
        path: path.to_path_buf(),
        source,
    })
}

fn push_watch_path(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if path.is_dir() {
        paths.push(path);
    } else if let Some(parent) = path.parent() {
        paths.push(parent.to_path_buf());
    } else {
        paths.push(path);
    }
}

fn build_site(config: ProjectConfig, base: &Path) -> Result<ProjectSite, ConfigError> {
    let mut site = ProjectSite::new(config.site.title, config.site.description);
    site.base_url = config.site.base_url;
    site.footer_note = config.site.footer_note;
    site.primary_person = config.site.primary_person;
    site.theme = build_theme(config.theme);
    site.people = config.people.into_iter().map(build_person).collect();
    site.projects = config.projects.into_iter().map(build_project).collect();
    if let Some(primary_person) = &site.primary_person {
        find_person(&site.people, primary_person)?;
    }
    site.nav = config
        .nav
        .into_iter()
        .map(|link| NavLink::new(link.label, link.href))
        .collect();
    site.footer_links = config
        .footer_links
        .into_iter()
        .map(|link| NavLink::new(link.label, link.href))
        .collect();

    for asset in config.assets {
        site = site.asset(Asset::new(resolve_path(base, asset.source), asset.target));
    }

    for page in config.pages {
        site.pages.push(build_page(page, base, &site.people)?);
    }

    Ok(site)
}

fn build_theme(config: ThemeConfig) -> ProjectTheme {
    ProjectTheme {
        paper: config.paper,
        surface: config.surface,
        ink: config.ink,
        ink_soft: config.ink_soft,
        line: config.line,
        accent: config.accent,
        accent_soft: config.accent_soft,
        secondary: config.secondary,
        warning: config.warning,
        rust: config.rust,
    }
}

fn build_page(
    config: PageConfig,
    base: &Path,
    people: &[PersonReference],
) -> Result<Page, ConfigError> {
    let mut page = Page::new(config.slug, config.title).description(config.description);
    for section in config.sections {
        page = page.section(build_section(section, base, people)?);
    }
    Ok(page)
}

fn build_person(config: PersonConfig) -> PersonReference {
    PersonReference {
        id: config.id,
        name: config.name,
        url: config.url,
        role: config.role,
        avatar_url: config.avatar_url,
        links: config
            .links
            .into_iter()
            .map(|link| ExternalLink::new(link.label, link.href, link.kind))
            .collect(),
    }
}

fn build_project(config: ProjectReferenceConfig) -> ProjectReference {
    ProjectReference {
        title: config.title,
        url: config.url,
        source_url: config.source_url,
        demo_url: config.demo_url,
        links: config
            .links
            .into_iter()
            .map(|link| ExternalLink::new(link.label, link.href, link.kind))
            .collect(),
    }
}

#[allow(unreachable_code, unused_variables)]
fn build_section(
    config: SectionConfig,
    base: &Path,
    people: &[PersonReference],
) -> Result<ProjectSection, ConfigError> {
    Ok(match config {
        #[cfg(feature = "brick-hero")]
        SectionConfig::Hero {
            logo_src,
            title,
            tagline,
            subtitle,
            person,
            ctas,
        } => ProjectSection::Hero(crate::bricks::hero::config::build_hero(
            logo_src, title, tagline, subtitle, person, ctas,
        )),
        #[cfg(feature = "brick-feature-grid")]
        SectionConfig::FeatureGrid { id, features } => ProjectSection::FeatureGrid(
            crate::bricks::feature_grid::config::build_feature_grid(id, features),
        ),
        #[cfg(feature = "brick-install")]
        SectionConfig::Install {
            id,
            heading,
            intro,
            guide_href,
            primary_routes,
            secondary_routes,
        } => ProjectSection::Install(crate::bricks::install::config::build_install_section(
            id,
            heading,
            intro,
            guide_href,
            primary_routes,
            secondary_routes,
        )),
        #[cfg(feature = "brick-person-mention")]
        SectionConfig::PersonMention {
            id,
            heading,
            intro,
            person,
        } => {
            let person = find_person(people, &person)?;
            ProjectSection::PersonMention(
                crate::bricks::person_mention::config::build_person_mention(
                    id, heading, intro, person,
                ),
            )
        }
        #[cfg(feature = "brick-workflow-steps")]
        SectionConfig::WorkflowSteps {
            id,
            heading,
            intro,
            steps,
        } => ProjectSection::WorkflowSteps(
            crate::bricks::workflow_steps::config::build_workflow_steps(id, heading, intro, steps),
        ),
        #[cfg(feature = "brick-audience-grid")]
        SectionConfig::AudienceGrid {
            id,
            heading,
            intro,
            audiences,
        } => {
            ProjectSection::AudienceGrid(crate::bricks::audience_grid::config::build_audience_grid(
                id, heading, intro, audiences,
            ))
        }
        #[cfg(feature = "brick-trust-panel")]
        SectionConfig::TrustPanel {
            id,
            heading,
            intro,
            items,
        } => ProjectSection::TrustPanel(crate::bricks::trust_panel::config::build_trust_panel(
            id, heading, intro, items,
        )),
        #[cfg(feature = "brick-screenshot-grid")]
        SectionConfig::ScreenshotGrid {
            id,
            heading,
            intro,
            screenshots,
        } => ProjectSection::ScreenshotGrid(
            crate::bricks::screenshot_grid::config::build_screenshot_grid(
                id,
                heading,
                intro,
                screenshots,
            ),
        ),
        #[cfg(feature = "brick-capability-matrix")]
        SectionConfig::CapabilityMatrix {
            id,
            heading,
            intro_html,
            source,
        } => ProjectSection::CapabilityMatrix(
            crate::bricks::capability_matrix::config::load_capability_matrix(
                id,
                heading,
                intro_html,
                &resolve_path(base, source),
            )?,
        ),
        #[cfg(feature = "brick-comparison")]
        SectionConfig::Comparison {
            id,
            heading,
            intro,
            rows,
        } => ProjectSection::Comparison(crate::bricks::comparison::config::build_comparison(
            id, heading, intro, rows,
        )),
        #[cfg(feature = "brick-content")]
        SectionConfig::Content { id, heading, html } => {
            ProjectSection::Content(build_content(id.unwrap_or_default(), heading, html))
        }
    })
}

fn find_person(people: &[PersonReference], id: &str) -> Result<PersonReference, ConfigError> {
    people
        .iter()
        .find(|person| person.id == id)
        .cloned()
        .ok_or_else(|| ConfigError::UnknownPerson { id: id.to_string() })
}

pub(crate) fn resolve_path(base: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        base.join(path)
    }
}
