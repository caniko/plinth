//! Activity brick — forge contribution persistence and ranked public reads.

pub mod admin;
pub mod api;
pub mod cache;
pub mod migrations;
pub mod ranking;
pub mod refresh;

use super::{Brick, BrickMigration};

/// Brick descriptor for the activity feature (forge contributions).
pub struct ActivityBrick;

impl Brick for ActivityBrick {
    fn name(&self) -> &'static str {
        "activity"
    }

    fn migrations(&self) -> Vec<BrickMigration> {
        migrations::activity_migrations()
    }
}
