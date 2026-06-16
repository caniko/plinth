use std::path::PathBuf;

use plinth_person::{PersonReference, ProjectReference};

#[cfg(feature = "brick-audience-grid")]
use crate::bricks::audience_grid::AudienceGrid;
#[cfg(feature = "brick-capability-matrix")]
use crate::bricks::capability_matrix::CapabilityMatrix;
#[cfg(feature = "brick-comparison")]
use crate::bricks::comparison::ComparisonSection;
#[cfg(feature = "brick-content")]
use crate::bricks::content::ContentSection;
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

/// The complete configuration for a project landing site.
///
/// `ProjectSite` holds all metadata, pages, assets, navigation, people, and
/// theme information needed to render a static HTML site.
#[derive(Clone, Default)]
pub struct ProjectSite {
    /// Site title, shown in the browser tab and nav bar.
    pub title: String,
    /// Short description, used in the `<meta name="description">` fallback.
    pub description: String,
    /// Base URL (e.g. `https://example.com/`) used for absolute links.
    pub base_url: String,
    /// Links displayed in the top navigation bar.
    pub nav: Vec<NavLink>,
    /// Content pages, each rendered to a separate HTML file.
    pub pages: Vec<Page>,
    /// Static files (images, fonts, etc.) copied verbatim during render.
    pub assets: Vec<Asset>,
    /// Text displayed in the site footer.
    pub footer_note: String,
    /// Links displayed in the footer alongside the footer note.
    pub footer_links: Vec<NavLink>,
    /// People referenced in person-mention bricks.
    pub people: Vec<PersonReference>,
    /// Related project references displayed on the site.
    pub projects: Vec<ProjectReference>,
    /// The ID of the primary person (must match a `PersonReference.id`).
    pub primary_person: Option<String>,
    /// CSS custom-property colour theme.
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

/// CSS custom-property colour theme for the site.
///
/// Each field is an optional CSS colour value (e.g. `"#fff"`).  When `None`
/// the corresponding custom property is omitted from the stylesheet.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProjectTheme {
    /// `--pp-paper` — main background colour.
    pub paper: Option<String>,
    /// `--pp-surface` — card / surface background colour.
    pub surface: Option<String>,
    /// `--pp-ink` — primary text colour.
    pub ink: Option<String>,
    /// `--pp-ink-soft` — secondary / muted text colour.
    pub ink_soft: Option<String>,
    /// `--pp-line` — border / divider colour.
    pub line: Option<String>,
    /// `--pp-accent` — primary accent / link colour.
    pub accent: Option<String>,
    /// `--pp-accent-soft` — muted accent colour.
    pub accent_soft: Option<String>,
    /// `--pp-secondary` — secondary brand colour.
    pub secondary: Option<String>,
    /// `--pp-warning` — warning / destructive colour.
    pub warning: Option<String>,
    /// `--pp-rust` — rust / code-themed accent.
    pub rust: Option<String>,
}

/// A single navigation link with a visible label and destination URL.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavLink {
    /// Human-readable link text.
    pub label: String,
    /// Link target URL.
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

/// A static file to copy from the source tree into the output directory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Asset {
    /// Absolute or relative path to the source file on disk.
    pub source: PathBuf,
    /// Target path in the output directory (e.g. `"/images/logo.svg"`).
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

/// A page within the project site, rendered to `{slug}/index.html`.
#[derive(Clone)]
pub struct Page {
    /// URL slug (e.g. `"about"` produces `/about/`).  Use `"index"` for the
    /// home page.
    pub slug: String,
    /// Page title, shown in the browser tab and `<title>` element.
    pub title: String,
    /// Page-level description for `<meta name="description">`.
    pub description: String,
    /// Ordered list of content bricks that make up the page body.
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

/// A single content brick on a page.
///
/// Each variant corresponds to a feature-gated brick type.  Only variants
/// whose feature is enabled are available at compile time.
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
    #[cfg(feature = "brick-content")]
    Content(ContentSection),
    #[cfg(feature = "brick-custom")]
    Custom(CustomSection),
}
