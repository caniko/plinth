//! Person mention project brick.

pub mod config;
pub mod model;
pub mod render;

use super::ProjectBrick;

pub use model::PersonMention;

pub struct PersonMentionBrick;

impl ProjectBrick for PersonMentionBrick {
    fn name(&self) -> &'static str {
        "person_mention"
    }
}
