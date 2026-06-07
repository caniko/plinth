//! Static project-site primitives, rendering, development serving, and UX guardrails.

pub mod bricks;
pub mod config;
pub mod dev;
mod diagnostics;
mod model;
mod render;

#[cfg(feature = "brick-audience-grid")]
pub use bricks::audience_grid::{Audience, AudienceGrid};
#[cfg(feature = "brick-capability-matrix")]
pub use bricks::capability_matrix::{Capability, CapabilityMatrix};
#[cfg(feature = "brick-comparison")]
pub use bricks::comparison::{ComparisonRow, ComparisonSection};
#[cfg(feature = "brick-custom")]
pub use bricks::custom::CustomSection;
#[cfg(feature = "brick-feature-grid")]
pub use bricks::feature_grid::{Feature, FeatureGrid};
#[cfg(feature = "brick-hero")]
pub use bricks::hero::{Cta, Hero};
#[cfg(feature = "brick-install")]
pub use bricks::install::{InstallRoute, InstallSection, render_install_fragment};
#[cfg(feature = "brick-person-mention")]
pub use bricks::person_mention::PersonMention;
#[cfg(feature = "brick-screenshot-grid")]
pub use bricks::screenshot_grid::{Screenshot, ScreenshotGrid};
#[cfg(feature = "brick-trust-panel")]
pub use bricks::trust_panel::{TrustItem, TrustPanel};
#[cfg(feature = "brick-workflow-steps")]
pub use bricks::workflow_steps::{WorkflowStep, WorkflowSteps};
pub use config::{ProjectConfig, load_project_site, project_watch_paths};
pub use diagnostics::{Diagnostic, DiagnosticReport, Severity, assert_valid, validate_site};
#[cfg(feature = "brick-install")]
pub use diagnostics::{InstallRouteUxFinding, InstallUxReport, install_ux_report};
pub use model::{Asset, NavLink, Page, ProjectSection, ProjectSite, ProjectTheme};
pub use plinth_person::{ExternalLink, LinkKind, PersonReference, ProjectReference};
pub use render::{RenderError, RenderOptions, render_static};
