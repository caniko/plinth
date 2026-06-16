/// Reusable UI components.
pub mod animated_background;
/// Error message display component.
pub mod error_message;
/// Site footer component.
pub mod footer;
/// Site header with navigation and theme toggle.
pub mod header;
/// End-of-article donation call-to-action.
pub mod support_cta;
/// Dark mode toggle button.
pub mod theme_toggle;

/// Home-page animated background and its preset normalizer.
pub use animated_background::{AnimatedBackground, normalize_preset};
/// User-friendly error display with icon.
pub use error_message::ErrorMessage;
/// Site footer with links and attribution.
pub use footer::Footer;
/// Site header with navigation and theme toggle.
pub use header::Header;
/// Compact end-of-article donation CTA.
pub use support_cta::SupportCta;
/// Dark mode toggle button.
pub use theme_toggle::ThemeToggle;
