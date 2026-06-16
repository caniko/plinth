use std::path::PathBuf;

use serde::Deserialize;

#[cfg(feature = "brick-audience-grid")]
use crate::bricks::audience_grid::config::AudienceConfig;
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
#[cfg(feature = "brick-trust-panel")]
use crate::bricks::trust_panel::config::TrustItemConfig;
#[cfg(feature = "brick-workflow-steps")]
use crate::bricks::workflow_steps::config::WorkflowStepConfig;
use plinth_person::LinkKind;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfig {
    pub site: SiteConfig,
    #[serde(default)]
    pub theme: ThemeConfig,
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
    #[serde(default)]
    pub projects: Vec<ProjectReferenceConfig>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThemeConfig {
    #[serde(default)]
    pub paper: Option<String>,
    #[serde(default)]
    pub surface: Option<String>,
    #[serde(default)]
    pub ink: Option<String>,
    #[serde(default)]
    pub ink_soft: Option<String>,
    #[serde(default)]
    pub line: Option<String>,
    #[serde(default)]
    pub accent: Option<String>,
    #[serde(default)]
    pub accent_soft: Option<String>,
    #[serde(default)]
    pub secondary: Option<String>,
    #[serde(default)]
    pub warning: Option<String>,
    #[serde(default)]
    pub rust: Option<String>,
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
pub struct ProjectReferenceConfig {
    pub title: String,
    pub url: String,
    #[serde(default)]
    pub source_url: Option<String>,
    #[serde(default)]
    pub demo_url: Option<String>,
    #[serde(default)]
    pub links: Vec<PersonLinkConfig>,
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
    #[cfg(feature = "brick-workflow-steps")]
    WorkflowSteps {
        #[serde(default)]
        id: Option<String>,
        heading: String,
        intro: String,
        #[serde(default)]
        steps: Vec<WorkflowStepConfig>,
    },
    #[cfg(feature = "brick-audience-grid")]
    AudienceGrid {
        #[serde(default)]
        id: Option<String>,
        heading: String,
        intro: String,
        #[serde(default)]
        audiences: Vec<AudienceConfig>,
    },
    #[cfg(feature = "brick-trust-panel")]
    TrustPanel {
        #[serde(default)]
        id: Option<String>,
        heading: String,
        intro: String,
        #[serde(default)]
        items: Vec<TrustItemConfig>,
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
    #[cfg(feature = "brick-content")]
    Content {
        #[serde(default)]
        id: Option<String>,
        #[serde(default)]
        heading: Option<String>,
        html: String,
    },
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

fn default_base_url() -> String {
    "/".into()
}
