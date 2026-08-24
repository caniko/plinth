//! Project-card grid brick.

pub mod config;
pub mod model;
pub mod render;

use super::ProjectBrick;

pub use model::ProjectGrid;

/// Brick that renders site `[[projects]]` as cards.
pub struct ProjectGridBrick;

impl ProjectBrick for ProjectGridBrick {
    fn name(&self) -> &'static str {
        "project_grid"
    }
}
