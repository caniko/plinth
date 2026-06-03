#[cfg(feature = "brick-activity")]
use crate::RankingStrategy;
use crate::config::{
    AboutPageConfig, AnalyticsConfig, AuthorConfig, BlogPageConfig, DonationConfig, DonationLink,
    FooterConfig, HomePageConfig, NavItem, PagesConfig, PortfolioPageConfig, SiteConfig,
    SocialLinks, TodosPageConfig,
};
use serde::Deserialize;

/// Error returned when loading the Plinth configuration file.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// The config file exists but could not be read.
    #[error("failed to read config file {path}: {source}")]
    Read {
        path: String,
        source: std::io::Error,
    },
    /// The config file could not be parsed as TOML.
    #[error("failed to parse config file {path}: {source}")]
    Parse {
        path: String,
        source: toml::de::Error,
    },
}

/// Top-level [site] section in plinth.toml
#[derive(Debug, Clone, Deserialize)]
pub struct SiteSection {
    #[serde(default = "default_site_name")]
    pub name: String,
    #[serde(default = "default_tagline")]
    pub tagline: String,
    #[serde(default = "default_description")]
    pub description: String,
    #[serde(default = "default_lang")]
    pub lang: String,
    #[serde(default = "default_theme")]
    pub default_theme: String,
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub author: AuthorSection,
    #[serde(default)]
    pub social: SocialSection,
    #[serde(default)]
    pub footer: FooterSection,
    #[serde(default = "default_nav")]
    pub nav: Vec<NavEntry>,
}

impl Default for SiteSection {
    fn default() -> Self {
        Self {
            name: default_site_name(),
            tagline: default_tagline(),
            description: default_description(),
            lang: default_lang(),
            default_theme: default_theme(),
            base_url: String::new(),
            author: AuthorSection::default(),
            social: SocialSection::default(),
            footer: FooterSection::default(),
            nav: default_nav(),
        }
    }
}

fn default_site_name() -> String {
    "Plinth".to_string()
}
fn default_tagline() -> String {
    "Welcome to my website".to_string()
}
fn default_description() -> String {
    "A personal website".to_string()
}
fn default_lang() -> String {
    "en".to_string()
}
fn default_theme() -> String {
    "dark".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct NavEntry {
    pub label: String,
    pub path: String,
}

fn default_nav() -> Vec<NavEntry> {
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

#[derive(Debug, Clone, Deserialize)]
pub struct AuthorSection {
    #[serde(default = "default_author_name")]
    pub name: String,
    #[serde(default)]
    pub email: String,
}

impl Default for AuthorSection {
    fn default() -> Self {
        Self {
            name: default_author_name(),
            email: String::new(),
        }
    }
}

fn default_author_name() -> String {
    "Admin".to_string()
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SocialSection {
    #[serde(default)]
    pub github: String,
    #[serde(default)]
    pub gitlab: String,
    #[serde(default)]
    pub codeberg: String,
    #[serde(default)]
    pub mastodon: String,
    #[serde(default)]
    pub bluesky: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FooterSection {
    #[serde(default = "default_project_name")]
    pub project_name: String,
    #[serde(default = "default_project_url")]
    pub project_url: String,
}

impl Default for FooterSection {
    fn default() -> Self {
        Self {
            project_name: default_project_name(),
            project_url: default_project_url(),
        }
    }
}

fn default_project_name() -> String {
    "Plinth".to_string()
}
fn default_project_url() -> String {
    "https://codeberg.org/caniko/plinth".to_string()
}

/// [server] section
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
        }
    }
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}
fn default_port() -> u16 {
    3000
}

/// [database] section
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    #[serde(default = "default_database_url")]
    pub database_url: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            database_url: default_database_url(),
        }
    }
}

fn default_database_url() -> String {
    "postgres://plinth:plinth@localhost:5432/plinth".to_string()
}

/// [observability] section
#[derive(Debug, Clone, Deserialize)]
pub struct ObservabilityTomlConfig {
    #[serde(default = "default_service_name")]
    pub service_name: String,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default)]
    pub otlp_endpoint: String,
    #[serde(default)]
    pub otlp_headers: String,
}

impl Default for ObservabilityTomlConfig {
    fn default() -> Self {
        Self {
            service_name: default_service_name(),
            log_level: default_log_level(),
            otlp_endpoint: String::new(),
            otlp_headers: String::new(),
        }
    }
}

fn default_service_name() -> String {
    "plinth".to_string()
}
fn default_log_level() -> String {
    "info".to_string()
}

/// [search] section
#[derive(Debug, Clone, Deserialize)]
pub struct SearchConfig {
    #[serde(default = "default_search_limit")]
    pub default_limit: usize,
    #[serde(default = "default_related_limit")]
    pub related_limit: usize,
    #[serde(default = "default_min_similarity")]
    pub min_similarity: f32,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            default_limit: default_search_limit(),
            related_limit: default_related_limit(),
            min_similarity: default_min_similarity(),
        }
    }
}

fn default_search_limit() -> usize {
    10
}
fn default_related_limit() -> usize {
    5
}
fn default_min_similarity() -> f32 {
    0.5
}

/// [ranking] section — activity ranking strategy + params.
#[cfg(feature = "brick-activity")]
#[derive(Debug, Clone, Deserialize)]
pub struct RankingConfig {
    #[serde(default = "default_ranking_strategy")]
    pub strategy: RankingStrategy,
    #[serde(default = "default_half_life_days")]
    pub half_life_days: f64,
    #[serde(default = "default_window_days")]
    pub window_days: f64,
}

#[cfg(feature = "brick-activity")]
impl Default for RankingConfig {
    fn default() -> Self {
        Self {
            strategy: default_ranking_strategy(),
            half_life_days: default_half_life_days(),
            window_days: default_window_days(),
        }
    }
}

#[cfg(feature = "brick-activity")]
fn default_ranking_strategy() -> RankingStrategy {
    RankingStrategy::Exponential
}

#[cfg(feature = "brick-activity")]
fn default_half_life_days() -> f64 {
    365.0
}

#[cfg(feature = "brick-activity")]
fn default_window_days() -> f64 {
    730.0
}

/// [forge] section — freshness + base URLs for activity refresh.
/// Tokens are env-only (GITHUB_TOKEN / CODEBERG_TOKEN), never toml keys.
#[cfg(feature = "brick-activity")]
#[derive(Debug, Clone, Deserialize)]
pub struct ForgeConfig {
    /// Stale-while-revalidate TTL in seconds.
    #[serde(default = "default_refresh_ttl_secs")]
    pub refresh_ttl_secs: u64,
    /// Backoff after a failed refresh, in seconds.
    #[serde(default = "default_refresh_backoff_secs")]
    pub refresh_backoff_secs: u64,
    /// GitHub REST API base.
    #[serde(default = "default_github_base_url")]
    pub github_base_url: String,
    /// Codeberg/Forgejo API base.
    #[serde(default = "default_codeberg_base_url")]
    pub codeberg_base_url: String,
}

#[cfg(feature = "brick-activity")]
impl Default for ForgeConfig {
    fn default() -> Self {
        Self {
            refresh_ttl_secs: default_refresh_ttl_secs(),
            refresh_backoff_secs: default_refresh_backoff_secs(),
            github_base_url: default_github_base_url(),
            codeberg_base_url: default_codeberg_base_url(),
        }
    }
}

#[cfg(feature = "brick-activity")]
fn default_refresh_ttl_secs() -> u64 {
    3600
}

#[cfg(feature = "brick-activity")]
fn default_refresh_backoff_secs() -> u64 {
    900
}

#[cfg(feature = "brick-activity")]
fn default_github_base_url() -> String {
    "https://api.github.com".to_string()
}

#[cfg(feature = "brick-activity")]
fn default_codeberg_base_url() -> String {
    "https://codeberg.org/api/v1".to_string()
}

/// [content] section
#[derive(Debug, Clone, Deserialize)]
pub struct ContentConfig {
    #[serde(default = "default_wpm")]
    pub words_per_minute: usize,
    #[serde(default = "default_vector_truncation")]
    pub vector_truncation: usize,
    /// Path to a directory containing declarative articles (set via PLINTH_CONTENT_DIR)
    #[serde(default)]
    pub content_dir: Option<String>,
}

impl Default for ContentConfig {
    fn default() -> Self {
        Self {
            words_per_minute: default_wpm(),
            vector_truncation: default_vector_truncation(),
            content_dir: None,
        }
    }
}

fn default_wpm() -> usize {
    200
}
fn default_vector_truncation() -> usize {
    5000
}

/// [immich] section
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ImmichTomlConfig {
    #[serde(default)]
    pub api_url: String,
}

/// [images] section
#[derive(Debug, Clone, Deserialize)]
pub struct ImagesConfig {
    #[serde(default = "default_cache_max_age")]
    pub cache_max_age: u64,
}

impl Default for ImagesConfig {
    fn default() -> Self {
        Self {
            cache_max_age: default_cache_max_age(),
        }
    }
}

fn default_cache_max_age() -> u64 {
    31_536_000
}

/// [feeds] section
#[derive(Debug, Clone, Deserialize)]
pub struct FeedsConfig {
    #[serde(default = "default_feed_limit")]
    pub blog_limit: usize,
    #[serde(default = "default_feed_limit")]
    pub projects_limit: usize,
    #[serde(default = "default_feed_limit")]
    pub activity_limit: usize,
}

impl Default for FeedsConfig {
    fn default() -> Self {
        Self {
            blog_limit: default_feed_limit(),
            projects_limit: default_feed_limit(),
            activity_limit: default_feed_limit(),
        }
    }
}

fn default_feed_limit() -> usize {
    50
}

/// [pages] section in the TOML (mirrors shared PagesConfig but uses Deserialize)
#[derive(Debug, Clone, Default, Deserialize)]
pub struct PagesTomlConfig {
    #[serde(default)]
    pub home: HomePagesToml,
    #[serde(default)]
    pub blog: BlogPagesToml,
    #[serde(default)]
    pub portfolio: PortfolioPagesToml,
    #[serde(default)]
    pub about: AboutPagesToml,
    #[serde(default)]
    pub todos: TodosPagesToml,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct HomePagesToml {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlogPagesToml {
    #[serde(default = "default_blog_title")]
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
    #[serde(default)]
    pub description: String,
}

impl Default for BlogPagesToml {
    fn default() -> Self {
        Self {
            title: default_blog_title(),
            subtitle: String::new(),
            description: String::new(),
        }
    }
}

fn default_blog_title() -> String {
    "Posts".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct PortfolioPagesToml {
    #[serde(default = "default_portfolio_title")]
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
    #[serde(default)]
    pub description: String,
}

impl Default for PortfolioPagesToml {
    fn default() -> Self {
        Self {
            title: default_portfolio_title(),
            subtitle: String::new(),
            description: String::new(),
        }
    }
}

fn default_portfolio_title() -> String {
    "Projects".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct AboutPagesToml {
    #[serde(default = "default_about_title")]
    pub title: String,
    #[serde(default)]
    pub description: String,
}

impl Default for AboutPagesToml {
    fn default() -> Self {
        Self {
            title: default_about_title(),
            description: String::new(),
        }
    }
}

fn default_about_title() -> String {
    "About Me".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct TodosPagesToml {
    #[serde(default = "default_todos_title")]
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
    #[serde(default)]
    pub description: String,
}

impl Default for TodosPagesToml {
    fn default() -> Self {
        Self {
            title: default_todos_title(),
            subtitle: String::new(),
            description: String::new(),
        }
    }
}

fn default_todos_title() -> String {
    "Bucket List".to_string()
}

/// [analytics] section
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AnalyticsTomlConfig {
    #[serde(default)]
    pub plausible_domain: String,
    #[serde(default)]
    pub plausible_script_url: String,
}

/// [donation] section
#[derive(Debug, Clone, Default, Deserialize)]
pub struct DonationTomlConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub links: Vec<DonationLinkToml>,
    #[serde(default)]
    pub cta_text: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DonationLinkToml {
    pub platform: String,
    pub url: String,
    #[serde(default)]
    pub label: String,
}

/// Full server configuration deserialized from plinth.toml
#[derive(Debug, Clone, Default, Deserialize)]
pub struct PlinthConfig {
    #[serde(default)]
    pub site: SiteSection,
    #[serde(default)]
    pub pages: PagesTomlConfig,
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub observability: ObservabilityTomlConfig,
    #[serde(default)]
    pub search: SearchConfig,
    #[serde(default)]
    pub content: ContentConfig,
    #[serde(default)]
    pub immich: ImmichTomlConfig,
    #[serde(default)]
    pub images: ImagesConfig,
    #[serde(default)]
    pub feeds: FeedsConfig,
    #[cfg(feature = "brick-activity")]
    #[serde(default)]
    pub ranking: RankingConfig,
    #[cfg(feature = "brick-activity")]
    #[serde(default)]
    pub forge: ForgeConfig,
    #[serde(default)]
    pub analytics: AnalyticsTomlConfig,
    #[serde(default)]
    pub donation: DonationTomlConfig,
}

impl PlinthConfig {
    /// Load config: TOML file first, then env var overrides.
    /// If no file exists, all fields use their defaults.
    pub fn load() -> Result<Self, ConfigError> {
        let config_path =
            std::env::var("PLINTH_CONFIG").unwrap_or_else(|_| "plinth.toml".to_string());

        let mut config: PlinthConfig = if std::path::Path::new(&config_path).exists() {
            let content =
                std::fs::read_to_string(&config_path).map_err(|source| ConfigError::Read {
                    path: config_path.clone(),
                    source,
                })?;
            toml::from_str(&content).map_err(|source| ConfigError::Parse {
                path: config_path.clone(),
                source,
            })?
        } else {
            PlinthConfig::default()
        };

        config.apply_env_overrides();
        Ok(config)
    }

    /// Parse and validate a TOML string without applying env overrides.
    pub fn parse(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    /// Environment variables override TOML values.
    pub fn apply_env_overrides(&mut self) {
        if let Ok(v) = std::env::var("DATABASE_URL") {
            self.database.database_url = v;
        }
        if let Ok(v) = std::env::var("PLINTH_DATABASE_URL") {
            self.database.database_url = v;
        }
        if let Ok(v) = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
            self.observability.otlp_endpoint = v;
        }
        if let Ok(v) = std::env::var("OTEL_EXPORTER_OTLP_HEADERS") {
            self.observability.otlp_headers = v;
        }
        if let Ok(v) = std::env::var("OTEL_SERVICE_NAME") {
            self.observability.service_name = v;
        }
        if let Ok(v) = std::env::var("RUST_LOG") {
            self.observability.log_level = v;
        }
        if let Ok(v) = std::env::var("IMMICH_API_URL") {
            self.immich.api_url = v;
        }
        if let Ok(v) = std::env::var("PLINTH_BASE_URL") {
            self.site.base_url = v;
        }
        if let Ok(v) = std::env::var("PLAUSIBLE_DOMAIN") {
            self.analytics.plausible_domain = v;
        }
        if let Ok(v) = std::env::var("PLAUSIBLE_SCRIPT_URL") {
            self.analytics.plausible_script_url = v;
        }
        if let Ok(v) = std::env::var("PLINTH_CONTENT_DIR") {
            self.content.content_dir = Some(v);
        }
    }

    /// Extract the client-safe subset (no secrets like API keys)
    pub fn to_site_config(&self) -> SiteConfig {
        SiteConfig {
            name: self.site.name.clone(),
            tagline: self.site.tagline.clone(),
            description: self.site.description.clone(),
            lang: self.site.lang.clone(),
            default_theme: self.site.default_theme.clone(),
            base_url: self.site.base_url.clone(),
            author: AuthorConfig {
                name: self.site.author.name.clone(),
                email: self.site.author.email.clone(),
            },
            social: SocialLinks {
                github: self.site.social.github.clone(),
                gitlab: self.site.social.gitlab.clone(),
                codeberg: self.site.social.codeberg.clone(),
                mastodon: self.site.social.mastodon.clone(),
                bluesky: self.site.social.bluesky.clone(),
            },
            footer: FooterConfig {
                project_name: self.site.footer.project_name.clone(),
                project_url: self.site.footer.project_url.clone(),
            },
            nav: self
                .site
                .nav
                .iter()
                .map(|n| NavItem {
                    label: n.label.clone(),
                    path: n.path.clone(),
                })
                .collect(),
            pages: PagesConfig {
                home: HomePageConfig {
                    title: self.pages.home.title.clone(),
                    description: self.pages.home.description.clone(),
                },
                blog: BlogPageConfig {
                    title: self.pages.blog.title.clone(),
                    subtitle: self.pages.blog.subtitle.clone(),
                    description: self.pages.blog.description.clone(),
                },
                portfolio: PortfolioPageConfig {
                    title: self.pages.portfolio.title.clone(),
                    subtitle: self.pages.portfolio.subtitle.clone(),
                    description: self.pages.portfolio.description.clone(),
                },
                about: AboutPageConfig {
                    title: self.pages.about.title.clone(),
                    description: self.pages.about.description.clone(),
                },
                todos: TodosPageConfig {
                    title: self.pages.todos.title.clone(),
                    subtitle: self.pages.todos.subtitle.clone(),
                    description: self.pages.todos.description.clone(),
                },
            },
            analytics: AnalyticsConfig {
                plausible_domain: self.analytics.plausible_domain.clone(),
                plausible_script_url: self.analytics.plausible_script_url.clone(),
            },
            donation: DonationConfig {
                enabled: self.donation.enabled,
                links: self
                    .donation
                    .links
                    .iter()
                    .map(|l| DonationLink {
                        platform: l.platform.clone(),
                        url: l.url.clone(),
                        label: l.label.clone(),
                    })
                    .collect(),
                cta_text: self.donation.cta_text.clone(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PlinthConfig::default();
        assert_eq!(config.site.name, "Plinth");
        assert_eq!(config.server.port, 3000);
        assert_eq!(
            config.database.database_url,
            "postgres://plinth:plinth@localhost:5432/plinth"
        );
        assert_eq!(config.observability.log_level, "info");
        assert_eq!(config.content.words_per_minute, 200);
        assert_eq!(config.images.cache_max_age, 31_536_000);
    }

    #[test]
    fn test_parse_empty_toml() {
        let config: PlinthConfig = toml::from_str("").unwrap();
        assert_eq!(config.site.name, "Plinth");
        assert_eq!(config.server.port, 3000);
    }

    #[test]
    fn test_parse_partial_toml() {
        let toml_str = r#"
[site]
name = "My Site"

[server]
port = 8080
"#;
        let config: PlinthConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.site.name, "My Site");
        assert_eq!(config.server.port, 8080);
        // Defaults preserved for unspecified fields
        assert_eq!(
            config.database.database_url,
            "postgres://plinth:plinth@localhost:5432/plinth"
        );
        assert_eq!(config.site.lang, "en");
    }

    #[cfg(feature = "brick-activity")]
    #[test]
    fn test_ranking_defaults() {
        let config = PlinthConfig::default();
        assert_eq!(config.ranking.strategy, RankingStrategy::Exponential);
        assert_eq!(config.ranking.half_life_days, 365.0);
        assert_eq!(config.ranking.window_days, 730.0);
    }

    #[cfg(feature = "brick-activity")]
    #[test]
    fn test_parse_ranking_toml() {
        let toml_str = r#"
[ranking]
strategy = "linear"
half_life_days = 180.0
window_days = 90.0
"#;
        let config: PlinthConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.ranking.strategy, RankingStrategy::Linear);
        assert_eq!(config.ranking.half_life_days, 180.0);
        assert_eq!(config.ranking.window_days, 90.0);
    }

    #[cfg(feature = "brick-activity")]
    #[test]
    fn test_forge_defaults() {
        let config: PlinthConfig = toml::from_str("").unwrap();
        assert_eq!(config.forge.refresh_ttl_secs, 3600);
        assert_eq!(config.forge.refresh_backoff_secs, 900);
        assert_eq!(config.forge.github_base_url, "https://api.github.com");
        assert_eq!(
            config.forge.codeberg_base_url,
            "https://codeberg.org/api/v1"
        );
    }

    #[cfg(feature = "brick-activity")]
    #[test]
    fn test_parse_forge_toml() {
        let toml_str = r#"
[forge]
refresh_ttl_secs = 120
refresh_backoff_secs = 30
github_base_url = "http://github.example.test"
codeberg_base_url = "http://codeberg.example.test/api/v1"
"#;
        let config: PlinthConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.forge.refresh_ttl_secs, 120);
        assert_eq!(config.forge.refresh_backoff_secs, 30);
        assert_eq!(config.forge.github_base_url, "http://github.example.test");
        assert_eq!(
            config.forge.codeberg_base_url,
            "http://codeberg.example.test/api/v1"
        );
    }

    #[test]
    fn test_parse_nav_items() {
        let toml_str = r#"
[[site.nav]]
label = "Blog"
path = "/blog"

[[site.nav]]
label = "Contact"
path = "/contact"
"#;
        let config: PlinthConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.site.nav.len(), 2);
        assert_eq!(config.site.nav[0].label, "Blog");
        assert_eq!(config.site.nav[1].path, "/contact");
    }

    #[test]
    fn test_to_site_config() {
        let config = PlinthConfig::default();
        let site = config.to_site_config();
        assert_eq!(site.name, "Plinth");
        assert_eq!(site.nav.len(), 3);
        assert_eq!(site.author.name, "Admin");
    }

    #[test]
    fn test_full_toml_parse() {
        let toml_str = r##"
[site]
name = "Test Site"
tagline = "A test"
description = "Testing"
lang = "de"
default_theme = "light"

[site.author]
name = "Tester"
email = "test@example.com"

[site.social]
github = "https://github.com/test"

[site.footer]
project_name = "TestProject"
project_url = "https://example.com"

[[site.nav]]
label = "Home"
path = "/"

[pages.blog]
title = "Articles"
subtitle = "My writings"

[server]
host = "0.0.0.0"
port = 9000

[database]
database_url = "postgres://test:test@localhost:5432/testdb"

[observability]
service_name = "test-service"
log_level = "debug"
otlp_endpoint = "https://otel.example.com"

[search]
default_limit = 20
related_limit = 10
min_similarity = 0.7

[content]
words_per_minute = 250
vector_truncation = 3000

[immich]
api_url = "https://immich.example.com"

[images]
cache_max_age = 86400

[donation]
enabled = true
cta_text = "Support this project"

[[donation.links]]
platform = "kofi"
url = "https://ko-fi.com/tester"
"##;
        let config: PlinthConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.site.name, "Test Site");
        assert_eq!(config.site.lang, "de");
        assert_eq!(config.site.author.email, "test@example.com");
        assert_eq!(config.site.social.github, "https://github.com/test");
        assert_eq!(config.site.nav.len(), 1);
        assert_eq!(config.pages.blog.title, "Articles");
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(
            config.database.database_url,
            "postgres://test:test@localhost:5432/testdb"
        );
        assert_eq!(
            config.observability.otlp_endpoint,
            "https://otel.example.com"
        );
        assert_eq!(config.search.default_limit, 20);
        assert_eq!(config.content.vector_truncation, 3000);
        assert_eq!(config.immich.api_url, "https://immich.example.com");
        assert_eq!(config.images.cache_max_age, 86400);
        assert!(config.donation.enabled);
        assert_eq!(config.donation.cta_text, "Support this project");
        assert_eq!(config.donation.links.len(), 1);
        assert_eq!(config.donation.links[0].platform, "kofi");
    }

    #[test]
    fn test_default_donation_disabled() {
        let config = PlinthConfig::default();
        assert!(!config.donation.enabled);
        assert!(config.donation.links.is_empty());
        assert!(config.donation.cta_text.is_empty());
    }

    #[test]
    fn test_parse_donation_toml() {
        let toml_str = r#"
[donation]
enabled = true
cta_text = "Help me out!"

[[donation.links]]
platform = "kofi"
url = "https://ko-fi.com/testuser"

[[donation.links]]
platform = "github_sponsors"
url = "https://github.com/sponsors/testuser"

[[donation.links]]
platform = "liberapay"
url = "https://liberapay.com/testuser"
"#;
        let config: PlinthConfig = toml::from_str(toml_str).unwrap();
        assert!(config.donation.enabled);
        assert_eq!(config.donation.cta_text, "Help me out!");
        assert_eq!(config.donation.links.len(), 3);
        assert_eq!(config.donation.links[0].platform, "kofi");
        assert_eq!(config.donation.links[0].url, "https://ko-fi.com/testuser");
        assert!(config.donation.links[0].label.is_empty());
        assert_eq!(config.donation.links[1].platform, "github_sponsors");
        assert_eq!(config.donation.links[2].platform, "liberapay");
    }

    #[test]
    fn test_parse_donation_links_with_custom_label() {
        let toml_str = r#"
[donation]
enabled = true

[[donation.links]]
platform = "kofi"
url = "https://ko-fi.com/testuser"
label = "Buy me a coffee"
"#;
        let config: PlinthConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.donation.links[0].label, "Buy me a coffee");
    }

    #[test]
    fn test_donation_to_site_config() {
        let toml_str = r#"
[donation]
enabled = true
cta_text = "Support me"

[[donation.links]]
platform = "kofi"
url = "https://ko-fi.com/test"
label = "Coffee"

[[donation.links]]
platform = "liberapay"
url = "https://liberapay.com/test"
"#;
        let config: PlinthConfig = toml::from_str(toml_str).unwrap();
        let site = config.to_site_config();
        assert!(site.donation.enabled);
        assert_eq!(site.donation.cta_text, "Support me");
        assert_eq!(site.donation.links.len(), 2);
        assert_eq!(site.donation.links[0].platform, "kofi");
        assert_eq!(site.donation.links[0].url, "https://ko-fi.com/test");
        assert_eq!(site.donation.links[0].label, "Coffee");
        assert_eq!(site.donation.links[1].platform, "liberapay");
        assert!(site.donation.links[1].label.is_empty());
    }

    /// Test env overrides in a single test to avoid thread-safety issues.
    /// Uses `unsafe` because `set_var`/`remove_var` are unsafe in Rust 2024 edition.
    #[test]
    fn test_env_overrides() {
        let mut config = PlinthConfig::default();
        assert_eq!(
            config.database.database_url,
            "postgres://plinth:plinth@localhost:5432/plinth"
        );
        assert!(config.immich.api_url.is_empty());
        assert!(config.site.base_url.is_empty());

        // SAFETY: this test runs serially (single test touching these env vars)
        unsafe {
            std::env::set_var(
                "PLINTH_DATABASE_URL",
                "postgres://env:env@localhost:5432/envdb",
            );
            std::env::set_var("IMMICH_API_URL", "http://immich:2283");
            std::env::set_var("PLINTH_BASE_URL", "https://example.com");
            std::env::set_var("PLAUSIBLE_DOMAIN", "mysite.com");
        }

        config.apply_env_overrides();

        assert_eq!(
            config.database.database_url,
            "postgres://env:env@localhost:5432/envdb"
        );
        assert_eq!(config.immich.api_url, "http://immich:2283");
        assert_eq!(config.site.base_url, "https://example.com");
        assert_eq!(config.analytics.plausible_domain, "mysite.com");

        // Clean up
        unsafe {
            std::env::remove_var("PLINTH_DATABASE_URL");
            std::env::remove_var("IMMICH_API_URL");
            std::env::remove_var("PLINTH_BASE_URL");
            std::env::remove_var("PLAUSIBLE_DOMAIN");
        }
    }

    #[test]
    fn test_env_overrides_precedence_over_toml() {
        let toml_str = r#"
[database]
database_url = "postgres://toml:toml@localhost:5432/tomldb"
"#;
        let mut config: PlinthConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(
            config.database.database_url,
            "postgres://toml:toml@localhost:5432/tomldb"
        );

        unsafe {
            std::env::set_var("DATABASE_URL", "postgres://env:env@localhost:5432/envdb");
        }
        config.apply_env_overrides();
        assert_eq!(
            config.database.database_url,
            "postgres://env:env@localhost:5432/envdb"
        );
        unsafe {
            std::env::remove_var("DATABASE_URL");
        }
    }

    #[test]
    fn test_load_missing_config_file_uses_defaults() {
        unsafe {
            std::env::set_var("PLINTH_CONFIG", "/tmp/plinth-nonexistent-test.toml");
        }
        let config = PlinthConfig::load().unwrap();
        assert_eq!(config.site.name, "Plinth");
        assert_eq!(config.server.port, 3000);
        unsafe {
            std::env::remove_var("PLINTH_CONFIG");
        }
    }

    #[test]
    fn test_content_dir_default_none() {
        let config = PlinthConfig::default();
        assert!(config.content.content_dir.is_none());
    }

    #[test]
    fn test_content_dir_from_toml() {
        let toml_str = r#"
[content]
content_dir = "/srv/articles"
"#;
        let config: PlinthConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.content.content_dir.as_deref(), Some("/srv/articles"));
    }

    #[test]
    fn test_content_dir_env_override() {
        let mut config = PlinthConfig::default();
        assert!(config.content.content_dir.is_none());

        unsafe {
            std::env::set_var("PLINTH_CONTENT_DIR", "/nix/store/articles");
        }
        config.apply_env_overrides();
        assert_eq!(
            config.content.content_dir.as_deref(),
            Some("/nix/store/articles")
        );
        unsafe {
            std::env::remove_var("PLINTH_CONTENT_DIR");
        }
    }
}
