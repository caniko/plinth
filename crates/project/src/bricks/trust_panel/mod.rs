//! Trust, rights, and operating posture project brick.

pub mod config;
pub mod model;
pub mod render;

use super::ProjectBrick;

pub use model::{TrustItem, TrustPanel};

pub struct TrustPanelBrick;

impl ProjectBrick for TrustPanelBrick {
    fn name(&self) -> &'static str {
        "trust_panel"
    }
}
