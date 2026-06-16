use crate::config::{
    AboutPageConfig, AnalyticsConfig, AuthorConfig, BlogPageConfig, DonationConfig, DonationLink,
    FooterConfig, HomePageConfig, NavItem, PagesConfig, PortfolioPageConfig, SiteConfig,
    SocialLinks, TodosPageConfig,
};

use super::types::*;

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
            animated_background: self.site.animated_background.clone(),
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
            logo: self.site.logo.clone(),
            favicon: self.site.favicon.clone(),
        }
    }
}
