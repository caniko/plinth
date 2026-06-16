//! Trust, rights, and operating posture project brick.

pub mod config;
pub mod model;
pub mod render;

use super::ProjectBrick;

pub use model::{TrustItem, TrustPanel};

/// Brick that renders a trust, rights, and operating-posture panel.
///
/// Displays a `<section class="trust-panel">` with heading, intro,
/// and a list of `<article class="trust-item">` cards, each with
/// a title (`<h3>`) and description (`<p>`).  Suitable for privacy
/// policies, license terms, or governance statements.
pub struct TrustPanelBrick;

impl ProjectBrick for TrustPanelBrick {
    fn name(&self) -> &'static str {
        "trust_panel"
    }
}
