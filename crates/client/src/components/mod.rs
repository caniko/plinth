// Reusable components

pub mod animated_background;
pub mod error_message;
pub mod footer;
pub mod header;
pub mod support_cta;
pub mod theme_toggle;

// Re-export for convenience
pub use animated_background::{AnimatedBackground, normalize_preset};
pub use error_message::ErrorMessage;
pub use footer::Footer;
pub use header::Header;
pub use support_cta::SupportCta;
pub use theme_toggle::ThemeToggle;
