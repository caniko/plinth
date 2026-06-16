#[cfg(feature = "brick-activity")]
use crate::RankingStrategy;
use crate::toml_config::types::NavEntry;

pub(super) fn default_site_name() -> String {
    "Plinth".to_string()
}
pub(super) fn default_tagline() -> String {
    "Welcome to my website".to_string()
}
pub(super) fn default_description() -> String {
    "A personal website".to_string()
}
pub(super) fn default_lang() -> String {
    "en".to_string()
}
pub(super) fn default_theme() -> String {
    "dark".to_string()
}

pub(super) fn default_animated_background() -> String {
    "flow-field".to_string()
}

pub(super) fn default_nav() -> Vec<NavEntry> {
    vec![
        NavEntry {
            label: "Posts".to_string(),
            path: "/posts".to_string(),
        },
        NavEntry {
            label: "Projects".to_string(),
            path: "/projects".to_string(),
        },
        NavEntry {
            label: "About".to_string(),
            path: "/about".to_string(),
        },
    ]
}

pub(super) fn default_author_name() -> String {
    "Admin".to_string()
}

pub(super) fn default_project_name() -> String {
    "Plinth".to_string()
}
pub(super) fn default_project_url() -> String {
    "https://codeberg.org/caniko/plinth".to_string()
}

pub(super) fn default_host() -> String {
    "127.0.0.1".to_string()
}
pub(super) fn default_port() -> u16 {
    3000
}

pub(super) fn default_database_url() -> String {
    "postgres://plinth:plinth@localhost:5432/plinth".to_string()
}

pub(super) fn default_service_name() -> String {
    "plinth".to_string()
}
pub(super) fn default_log_level() -> String {
    "info".to_string()
}

pub(super) fn default_search_limit() -> usize {
    10
}
pub(super) fn default_related_limit() -> usize {
    5
}
pub(super) fn default_min_similarity() -> f32 {
    0.5
}

#[cfg(feature = "brick-activity")]
pub(super) fn default_ranking_strategy() -> RankingStrategy {
    RankingStrategy::Exponential
}

#[cfg(feature = "brick-activity")]
pub(super) fn default_half_life_days() -> f64 {
    365.0
}

#[cfg(feature = "brick-activity")]
pub(super) fn default_window_days() -> f64 {
    730.0
}

#[cfg(feature = "brick-activity")]
pub(super) fn default_refresh_ttl_secs() -> u64 {
    3600
}

#[cfg(feature = "brick-activity")]
pub(super) fn default_refresh_backoff_secs() -> u64 {
    900
}

#[cfg(feature = "brick-activity")]
pub(super) fn default_github_base_url() -> String {
    "https://api.github.com".to_string()
}

#[cfg(feature = "brick-activity")]
pub(super) fn default_codeberg_base_url() -> String {
    "https://codeberg.org/api/v1".to_string()
}

pub(super) fn default_wpm() -> usize {
    200
}
pub(super) fn default_vector_truncation() -> usize {
    5000
}

pub(super) fn default_cache_max_age() -> u64 {
    31_536_000
}

pub(super) fn default_feed_limit() -> usize {
    50
}

pub(super) fn default_blog_title() -> String {
    "Posts".to_string()
}

pub(super) fn default_portfolio_title() -> String {
    "Projects".to_string()
}

pub(super) fn default_about_title() -> String {
    "About Me".to_string()
}

pub(super) fn default_todos_title() -> String {
    "Bucket List".to_string()
}
