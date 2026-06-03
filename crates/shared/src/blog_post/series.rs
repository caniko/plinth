use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Navigation context for a post within a series
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeriesNav {
    pub series_slug: String,
    pub series_title: String,
    pub current_position: u32,
    pub total_published: u32,
    pub prev: Option<SeriesEntry>,
    pub next: Option<SeriesEntry>,
    pub entries: Vec<SeriesEntry>,
}

/// A single entry in a series table of contents
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeriesEntry {
    pub slug: String,
    pub title: String,
    #[serde(alias = "series_position", default)]
    pub position: u32,
}

/// Lightweight series info for listing pages
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeriesListItem {
    pub slug: String,
    pub title: String,
    pub post_count: u32,
    pub total_reading_time: u32,
    pub latest_published_at: Option<DateTime<Utc>>,
}

/// Convert a slug like "weekly-rust-tips" to "Weekly Rust Tips"
pub fn humanize_slug(slug: &str) -> String {
    slug.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => {
                    let upper: String = first.to_uppercase().collect();
                    upper + chars.as_str()
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
