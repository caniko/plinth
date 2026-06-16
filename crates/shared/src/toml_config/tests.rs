#[cfg(feature = "brick-activity")]
use crate::RankingStrategy;

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
    assert_eq!(config.site.logo, None);
    assert_eq!(config.site.favicon, None);
}

#[test]
fn test_parse_site_logo_and_favicon() {
    let toml_str = r#"
[site]
logo = "/my-logo.svg"
favicon = "/my-favicon.ico"
"#;
    let config: PlinthConfig = toml::from_str(toml_str).unwrap();
    assert_eq!(config.site.logo, Some("/my-logo.svg".to_string()));
    assert_eq!(config.site.favicon, Some("/my-favicon.ico".to_string()));
}

#[test]
fn test_site_logo_favicon_default_to_none() {
    let config: PlinthConfig = PlinthConfig::default();
    assert_eq!(config.site.logo, None);
    assert_eq!(config.site.favicon, None);
}

#[test]
fn test_to_site_config_preserves_logo_favicon() {
    let toml_str = r#"
[site]
name = "Test"
logo = "/custom-logo.svg"
favicon = "/custom-favicon.svg"
"#;
    let config: PlinthConfig = toml::from_str(toml_str).unwrap();
    let site_config = config.to_site_config();
    assert_eq!(site_config.logo, Some("/custom-logo.svg".to_string()));
    assert_eq!(site_config.favicon, Some("/custom-favicon.svg".to_string()));
}

#[test]
fn test_to_site_config_logo_favicon_none_by_default() {
    let config: PlinthConfig = PlinthConfig::default();
    let site_config = config.to_site_config();
    assert_eq!(site_config.logo, None);
    assert_eq!(site_config.favicon, None);
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
    assert_eq!(site.animated_background, "flow-field");
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
animated_background = "constellation"

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
    assert_eq!(config.site.animated_background, "constellation");
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
