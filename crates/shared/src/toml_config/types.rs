use super::defaults::*;
#[cfg(feature = "brick-activity")]
use crate::RankingStrategy;
use serde::Deserialize;

/// Top-level ``[site]`` section in plinth.toml
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
    #[serde(default = "default_animated_background")]
    pub animated_background: String,
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
    #[serde(default)]
    pub logo: Option<String>,
    #[serde(default)]
    pub favicon: Option<String>,
}

impl Default for SiteSection {
    fn default() -> Self {
        Self {
            name: default_site_name(),
            tagline: default_tagline(),
            description: default_description(),
            lang: default_lang(),
            default_theme: default_theme(),
            animated_background: default_animated_background(),
            base_url: String::new(),
            author: AuthorSection::default(),
            social: SocialSection::default(),
            footer: FooterSection::default(),
            nav: default_nav(),
            logo: None,
            favicon: None,
        }
    }
}

/// A single entry in the site navigation bar.
#[derive(Debug, Clone, Deserialize)]
pub struct NavEntry {
    pub label: String,
    pub path: String,
}

/// Site author metadata displayed on the site.
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

/// Social-media profile links displayed in the site header / footer.
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

/// Site footer metadata — project name and its canonical URL.
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

/// `[server]` section
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

/// `[database]` section
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

/// `[observability]` section
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

/// `[search]` section
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

/// `[ranking]` section — activity ranking strategy + params.
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

/// `[forge]` section — freshness + base URLs for activity refresh.
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

/// `[content]` section
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

/// `[immich]` section
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ImmichTomlConfig {
    #[serde(default)]
    pub api_url: String,
}

/// `[images]` section
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

/// `[feeds]` section
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

/// `[pages]` section in the TOML (mirrors shared PagesConfig but uses Deserialize)
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

/// [`[pages.home]`](PagesTomlConfig) section — homepage title and description.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct HomePagesToml {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
}

/// [`[pages.blog]`](PagesTomlConfig) section — blog index page metadata.
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

/// [`[pages.portfolio]`](PagesTomlConfig) section — portfolio index page metadata.
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

/// [`[pages.about]`](PagesTomlConfig) section — about page metadata.
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

/// [`[pages.todos]`](PagesTomlConfig) section — bucket-list / todos page metadata.
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

/// `[analytics]` section
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AnalyticsTomlConfig {
    #[serde(default)]
    pub plausible_domain: String,
    #[serde(default)]
    pub plausible_script_url: String,
}

/// `[donation]` section
#[derive(Debug, Clone, Default, Deserialize)]
pub struct DonationTomlConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub links: Vec<DonationLinkToml>,
    #[serde(default)]
    pub cta_text: String,
}

/// A single donation-platform link inside the [`[donation]`](DonationTomlConfig) section.
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
