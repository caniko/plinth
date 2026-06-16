//! Person mention project brick.

pub mod config;
pub mod model;
pub mod render;

use super::ProjectBrick;

pub use model::PersonMention;

/// Brick that renders a person / team-member mention card.
///
/// Displays a `<section class="person-mention">` with heading, intro,
/// and an `<article class="person-card">` containing avatar, name,
/// role, and external links from a `PersonReference`.
pub struct PersonMentionBrick;

impl ProjectBrick for PersonMentionBrick {
    fn name(&self) -> &'static str {
        "person_mention"
    }
}
