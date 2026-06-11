use std::path::PathBuf;

use plinth_person::{PersonReference, ProjectReference};

#[cfg(feature = "brick-audience-grid")]
use crate::bricks::audience_grid::AudienceGrid;
#[cfg(feature = "brick-capability-matrix")]
use crate::bricks::capability_matrix::CapabilityMatrix;
#[cfg(feature = "brick-comparison")]
use crate::bricks::comparison::ComparisonSection;
#[cfg(feature = "brick-custom")]
use crate::bricks::custom::CustomSection;
#[cfg(feature = "brick-feature-grid")]
use crate::bricks::feature_grid::FeatureGrid;
#[cfg(feature = "brick-hero")]
use crate::bricks::hero::Hero;
#[cfg(feature = "brick-install")]
use crate::bricks::install::InstallSection;
#[cfg(feature = "brick-person-mention")]
use crate::bricks::person_mention::PersonMention;
#[cfg(feature = "brick-screenshot-grid")]
use crate::bricks::screenshot_grid::ScreenshotGrid;
#[cfg(feature = "brick-trust-panel")]
use crate::bricks::trust_panel::TrustPanel;
#[cfg(feature = "brick-workflow-steps")]
use crate::bricks::workflow_steps::WorkflowSteps;

#[derive(Clone, Default)]
pub struct ProjectSite {
    pub title: String,
    pub description: String,
    pub base_url: String,
    pub nav: Vec<NavLink>,
    pub pages: Vec<Page>,
    pub assets: Vec<Asset>,
    pub footer_note: String,
    pub footer_links: Vec<NavLink>,
    pub people: Vec<PersonReference>,
    pub projects: Vec<ProjectReference>,
    pub primary_person: Option<String>,
    pub theme: ProjectTheme,
}

impl ProjectSite {
    #[must_use]
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            ..Self::default()
        }
    }

    #[must_use]
    pub fn page(mut self, page: Page) -> Self {
        self.pages.push(page);
        self
    }

    #[must_use]
    pub fn asset(mut self, asset: Asset) -> Self {
        self.assets.push(asset);
        self
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProjectTheme {
    pub paper: Option<String>,
    pub surface: Option<String>,
    pub ink: Option<String>,
    pub ink_soft: Option<String>,
    pub line: Option<String>,
    pub accent: Option<String>,
    pub accent_soft: Option<String>,
    pub secondary: Option<String>,
    pub warning: Option<String>,
    pub rust: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavLink {
    pub label: String,
    pub href: String,
}

impl NavLink {
    #[must_use]
    pub fn new(label: impl Into<String>, href: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            href: href.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Asset {
    pub source: PathBuf,
    pub target: String,
}

impl Asset {
    #[must_use]
    pub fn new(source: impl Into<PathBuf>, target: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
        }
    }
}

#[derive(Clone)]
pub struct Page {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub sections: Vec<ProjectSection>,
}

impl Page {
    #[must_use]
    pub fn new(slug: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            slug: slug.into(),
            title: title.into(),
            description: String::new(),
            sections: Vec::new(),
        }
    }

    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    #[must_use]
    pub fn section(mut self, section: ProjectSection) -> Self {
        self.sections.push(section);
        self
    }
}

#[derive(Clone)]
pub enum ProjectSection {
    #[cfg(feature = "brick-hero")]
    Hero(Hero),
    #[cfg(feature = "brick-feature-grid")]
    FeatureGrid(FeatureGrid),
    #[cfg(feature = "brick-install")]
    Install(InstallSection),
    #[cfg(feature = "brick-person-mention")]
    PersonMention(PersonMention),
    #[cfg(feature = "brick-workflow-steps")]
    WorkflowSteps(WorkflowSteps),
    #[cfg(feature = "brick-audience-grid")]
    AudienceGrid(AudienceGrid),
    #[cfg(feature = "brick-trust-panel")]
    TrustPanel(TrustPanel),
    #[cfg(feature = "brick-screenshot-grid")]
    ScreenshotGrid(ScreenshotGrid),
    #[cfg(feature = "brick-capability-matrix")]
    CapabilityMatrix(CapabilityMatrix),
    #[cfg(feature = "brick-comparison")]
    Comparison(ComparisonSection),
    #[cfg(feature = "brick-custom")]
    Custom(CustomSection),
}
