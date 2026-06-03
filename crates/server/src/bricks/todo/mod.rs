//! Todo/Bucket List brick — trackable items with completion state.

pub mod admin;
pub mod api;
pub mod cache;
pub mod migrations;

use super::{Brick, BrickMigration};

/// Todo brick providing bucket list / TODO items.
pub struct TodoBrick;

impl Brick for TodoBrick {
    fn name(&self) -> &'static str {
        "todo"
    }

    fn migrations(&self) -> Vec<BrickMigration> {
        migrations::todo_migrations()
    }
}
