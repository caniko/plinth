//! `plinth-desktop`: a Dioxus 0.8 workspace module for writing and publishing
//! Plinth posts from a host desktop app (Fleetix Hub and similar).
//!
//! The crate renders only the workspace body. Hosts supply the shell, pass the
//! [`Site`] list loaded by [`load_sites`], and append [`STYLE`] to their
//! stylesheet. See `docs/src/guides/fleetix-hub-module.md` for the integration.

mod config;
mod draft;
mod drafts;
mod markdown;
mod publish;
mod ui;

pub use config::{ApiKey, ConfigError, Site, Target, load_sites};
pub use draft::{Draft, Field, FieldError, MAX_DESCRIPTION_LEN, MAX_SLUG_LEN, Replaces, slugify};
pub use publish::{PostSummary, PublishError, Receipt, list_posts, publish};
pub use ui::BlogWorkspace;

/// The workspace stylesheet. Inject it once, inside a `style` element.
pub const STYLE: &str = include_str!("style.css");

/// Workspace nav label supplied by the host.
pub const WORKSPACE_LABEL: &str = "Sites";
/// Workspace hint supplied by the host.
pub const WORKSPACE_HINT: &str = "write and publish posts to Plinth sites";
/// Workspace icon supplied by the host: a document with a written line.
pub const WORKSPACE_ICON_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M4 5.5A1.5 1.5 0 0 1 5.5 4h8L20 10.5v8A1.5 1.5 0 0 1 18.5 20h-13A1.5 1.5 0 0 1 4 18.5z"/><path d="M13 4v6.5h7"/><path d="M8 14h8"/><path d="M8 17h5"/></svg>"##;
