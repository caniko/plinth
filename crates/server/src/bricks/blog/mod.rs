//! Blog brick — blog posts, series, and related content.

pub mod admin;
pub mod cache;
pub mod migrations;

use super::{Brick, BrickMigration};

/// Blog brick providing blog posts and series support.
pub struct BlogBrick;

impl Brick for BlogBrick {
    fn name(&self) -> &'static str {
        "blog"
    }

    fn migrations(&self) -> Vec<BrickMigration> {
        migrations::blog_migrations()
    }
}
