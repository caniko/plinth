//! Portfolio brick — project showcase items.

pub mod cache;
pub mod migrations;

use super::{Brick, BrickMigration};

/// Portfolio brick providing project portfolio items.
pub struct PortfolioBrick;

impl Brick for PortfolioBrick {
    fn name(&self) -> &'static str {
        "portfolio"
    }

    fn migrations(&self) -> Vec<BrickMigration> {
        migrations::portfolio_migrations()
    }
}
