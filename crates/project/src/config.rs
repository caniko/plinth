use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::{Asset, NavLink, Page, ProjectSection, ProjectSite};
use plinth_person::{ExternalLink, LinkKind, PersonReference};

#[cfg(feature = "brick-comparison")]
use crate::bricks::comparison::config::ComparisonRowConfig;
#[cfg(feature = "brick-feature-grid")]
use crate::bricks::feature_grid::config::FeatureConfig;
#[cfg(feature = "brick-hero")]
use crate::bricks::hero::config::CtaConfig;
#[cfg(feature = "brick-install")]
use crate::bricks::install::config::InstallRouteConfig;
#[cfg(feature = "brick-screenshot-grid")]
use crate::bricks::screenshot_grid::config::ScreenshotConfig;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfig {
    pub site: SiteConfig,
    #[serde(default)]
    pub nav: Vec<LinkConfig>,
    #[serde(default)]
    pub footer_links: Vec<LinkConfig>,
    #[serde(default)]
    pub assets: Vec<AssetConfig>,
    #[serde(default)]
    pub pages: Vec<PageConfig>,
    #[serde(default)]
    pub people: Vec<PersonConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SiteConfig {
    pub title: String,
    pub description: String,
    #[serde(default = "default_base_url")]
    pub base_url: String,
    #[serde(default)]
    pub footer_note: String,
    #[serde(default)]
    pub primary_person: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinkConfig {
    pub label: String,
    #[serde(alias = "path")]
    pub href: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetConfig {
    pub source: PathBuf,
    pub target: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PersonConfig {
    pub id: String,
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub links: Vec<PersonLinkConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PersonLinkConfig {
    pub label: String,
    pub href: String,
    #[serde(default)]
    pub kind: LinkKind,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PageConfig {
    pub slug: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub sections: Vec<SectionConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum SectionConfig {
    #[cfg(feature = "brick-hero")]
    Hero {
        #[serde(default)]
        logo_src: Option<String>,
        title: String,
        tagline: String,
        subtitle: String,
        #[serde(default)]
        person: Option<String>,
        #[serde(default)]
        ctas: Vec<CtaConfig>,
    },
    #[cfg(feature = "brick-feature-grid")]
    FeatureGrid {
        #[serde(default)]
        id: Option<String>,
        #[serde(default)]
        features: Vec<FeatureConfig>,
    },
    #[cfg(feature = "brick-install")]
    Install {
        id: String,
        heading: String,
        intro: String,
        guide_href: String,
        #[serde(default)]
        primary_routes: Vec<InstallRouteConfig>,
        #[serde(default)]
        secondary_routes: Vec<InstallRouteConfig>,
    },
    #[cfg(feature = "brick-person-mention")]
    PersonMention {
        #[serde(default)]
        id: Option<String>,
        heading: String,
        intro: String,
        person: String,
    },
    #[cfg(feature = "brick-screenshot-grid")]
    ScreenshotGrid {
        id: String,
        heading: String,
        intro: String,
        #[serde(default)]
        screenshots: Vec<ScreenshotConfig>,
    },
    #[cfg(feature = "brick-capability-matrix")]
    CapabilityMatrix {
        id: String,
        heading: String,
        intro_html: String,
        source: PathBuf,
    },
    #[cfg(feature = "brick-comparison")]
    Comparison {
        #[serde(default)]
        id: Option<String>,
        heading: String,
        intro: String,
        #[serde(default)]
        rows: Vec<ComparisonRowConfig>,
    },
}

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

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse {path}: {source}")]
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[error("failed to read capability matrix {path}: {source}")]
    MatrixRead {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse capability matrix {path}: {source}")]
    MatrixParse {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[error("unknown person reference `{id}`")]
    UnknownPerson { id: String },
}

fn build_site(config: ProjectConfig, base: &Path) -> Result<ProjectSite, ConfigError> {
    let mut site = ProjectSite::new(config.site.title, config.site.description);
    site.base_url = config.site.base_url;
    site.footer_note = config.site.footer_note;
    site.primary_person = config.site.primary_person;
    site.people = config.people.into_iter().map(build_person).collect();
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

fn default_base_url() -> String {
    "/".into()
}

#[cfg(test)]
mod tests {
    use super::load_project_site;
    #[cfg(feature = "brick-capability-matrix")]
    use super::project_watch_paths;

    #[test]
    fn unknown_project_config_field_fails_fast() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("plinth-project.toml");
        std::fs::write(
            &config,
            r#"
[site]
title = "Example"
description = "Example site"
unexpected = "not allowed"

[[pages]]
slug = "index"
title = "Example"
"#,
        )
        .unwrap();

        let error = match load_project_site(&config) {
            Ok(_) => panic!("unknown site field should fail"),
            Err(error) => error.to_string(),
        };
        assert!(error.contains("unknown field"));
        assert!(error.contains("unexpected"));
    }

    #[cfg(all(feature = "brick-hero", feature = "brick-person-mention"))]
    #[test]
    fn person_config_renders_author_metadata_and_links() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("plinth-project.toml");
        let out = dir.path().join("public");
        std::fs::write(
            &config,
            r#"
[site]
title = "Example"
description = "Example site"
primary_person = "maintainer"

[[people]]
id = "maintainer"
name = "Maintainer"
url = "https://person.example"
role = "Project lead"

[[people.links]]
label = "Contact"
href = "https://person.example/contact"
kind = "contact"

[[pages]]
slug = "index"
title = "Example"

[[pages.sections]]
type = "hero"
title = "Example"
tagline = "Built plainly"
subtitle = "A project site"
person = "maintainer"

[[pages.sections]]
type = "person_mention"
id = "maintainer"
heading = "Maintainer"
intro = "Who keeps this project moving."
person = "maintainer"
"#,
        )
        .unwrap();

        let site = load_project_site(&config).unwrap();
        crate::render_static(&site, &crate::RenderOptions::new(&out)).unwrap();
        let html = std::fs::read_to_string(out.join("index.html")).unwrap();
        assert!(html.contains("application/ld+json"));
        assert!(html.contains("hero-byline"));
        assert!(html.contains("person-attribution"));
        assert!(html.contains("person-mention"));
        assert!(html.contains("link-contact"));
    }

    #[cfg(feature = "brick-capability-matrix")]
    #[test]
    fn capability_matrix_source_contributes_watch_path() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("plinth-project.toml");
        let matrix_dir = dir.path().join("data");
        std::fs::create_dir_all(&matrix_dir).unwrap();
        std::fs::write(
            &config,
            r#"
[site]
title = "Example"
description = "Example site"

[[pages]]
slug = "index"
title = "Example"

[[pages.sections]]
type = "capability_matrix"
id = "matrix"
heading = "Matrix"
intro_html = "Intro"
source = "data/capability-matrix.toml"
"#,
        )
        .unwrap();

        let paths = project_watch_paths(&config).unwrap();
        assert!(paths.contains(&matrix_dir));
    }
}
