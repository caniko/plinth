use plinth_shared::config::{
    AboutPageConfig, AuthorConfig, BlogPageConfig, FooterConfig, HomePageConfig, NavItem,
    PagesConfig, PortfolioPageConfig, SiteConfig, SocialLinks, TodosPageConfig,
};
use serde::Deserialize;

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
    #[serde(default = "default_db_path")]
    pub path: String,
    #[serde(default = "default_db_namespace")]
    pub namespace: String,
    #[serde(default = "default_db_database")]
    pub database: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: default_db_path(),
            namespace: default_db_namespace(),
            database: default_db_database(),
        }
    }
}

fn default_db_path() -> String {
    "database.db".to_string()
}
fn default_db_namespace() -> String {
    "plinth".to_string()
}
fn default_db_database() -> String {
    "main".to_string()
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

/// [content] section
#[derive(Debug, Clone, Deserialize)]
pub struct ContentConfig {
    #[serde(default = "default_wpm")]
    pub words_per_minute: usize,
    #[serde(default = "default_vector_truncation")]
    pub vector_truncation: usize,
}

impl Default for ContentConfig {
    fn default() -> Self {
        Self {
            words_per_minute: default_wpm(),
            vector_truncation: default_vector_truncation(),
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
}

impl Default for FeedsConfig {
    fn default() -> Self {
        Self {
            blog_limit: default_feed_limit(),
            projects_limit: default_feed_limit(),
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
}

impl PlinthConfig {
    /// Load config: TOML file first, then env var overrides.
    /// If no file exists, all fields use their defaults.
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path =
            std::env::var("PLINTH_CONFIG").unwrap_or_else(|_| "plinth.toml".to_string());

        let mut config: PlinthConfig = if std::path::Path::new(&config_path).exists() {
            let content = std::fs::read_to_string(&config_path)?;
            toml::from_str(&content)?
        } else {
            PlinthConfig::default()
        };

        config.apply_env_overrides();
        Ok(config)
    }

    /// Environment variables override TOML values (backwards compatible)
    fn apply_env_overrides(&mut self) {
        if let Ok(v) = std::env::var("SURREALDB_PATH") {
            self.database.path = v;
        }
        if let Ok(v) = std::env::var("SURREALDB_NAMESPACE") {
            self.database.namespace = v;
        }
        if let Ok(v) = std::env::var("SURREALDB_DATABASE") {
            self.database.database = v;
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
        assert_eq!(config.database.path, "database.db");
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
        assert_eq!(config.database.path, "database.db");
        assert_eq!(config.site.lang, "en");
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
path = "/data/db"
namespace = "test"
database = "testdb"

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
"##;
        let config: PlinthConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.site.name, "Test Site");
        assert_eq!(config.site.lang, "de");
        assert_eq!(config.site.author.email, "test@example.com");
        assert_eq!(config.site.social.github, "https://github.com/test");
        assert_eq!(config.site.nav.len(), 1);
        assert_eq!(config.pages.blog.title, "Articles");
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.database.path, "/data/db");
        assert_eq!(
            config.observability.otlp_endpoint,
            "https://otel.example.com"
        );
        assert_eq!(config.search.default_limit, 20);
        assert_eq!(config.content.vector_truncation, 3000);
        assert_eq!(config.immich.api_url, "https://immich.example.com");
        assert_eq!(config.images.cache_max_age, 86400);
    }
}
