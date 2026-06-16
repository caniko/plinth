use serde::{Deserialize, Serialize};

/// Navigation item (label + path)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NavItem {
    pub label: String,
    pub path: String,
}

/// Social links (all optional — only non-empty values render in UI)
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SocialLinks {
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

/// Footer configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FooterConfig {
    #[serde(default = "default_project_name")]
    pub project_name: String,
    #[serde(default = "default_project_url")]
    pub project_url: String,
}

impl Default for FooterConfig {
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

/// Author information (client-safe — no secrets)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthorConfig {
    #[serde(default = "default_author_name")]
    pub name: String,
    #[serde(default)]
    pub email: String,
}

impl Default for AuthorConfig {
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

/// Home page configuration
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HomePageConfig {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
}

/// Blog page configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlogPageConfig {
    #[serde(default = "default_blog_title")]
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
    #[serde(default)]
    pub description: String,
}

impl Default for BlogPageConfig {
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

/// Portfolio page configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PortfolioPageConfig {
    #[serde(default = "default_portfolio_title")]
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
    #[serde(default)]
    pub description: String,
}

impl Default for PortfolioPageConfig {
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

/// About page configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AboutPageConfig {
    #[serde(default = "default_about_title")]
    pub title: String,
    #[serde(default)]
    pub description: String,
}

impl Default for AboutPageConfig {
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

/// Todos/bucket list page configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TodosPageConfig {
    #[serde(default = "default_todos_title")]
    pub title: String,
    #[serde(default)]
    pub subtitle: String,
    #[serde(default)]
    pub description: String,
}

impl Default for TodosPageConfig {
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

/// Analytics configuration (Plausible)
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    #[serde(default)]
    pub plausible_domain: String,
    #[serde(default)]
    pub plausible_script_url: String,
}

/// A single donation/support link
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DonationLink {
    pub platform: String,
    pub url: String,
    #[serde(default)]
    pub label: String,
}

/// Donation configuration
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DonationConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub links: Vec<DonationLink>,
    #[serde(default)]
    pub cta_text: String,
}

/// Pages configuration
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PagesConfig {
    #[serde(default)]
    pub home: HomePageConfig,
    #[serde(default)]
    pub blog: BlogPageConfig,
    #[serde(default)]
    pub portfolio: PortfolioPageConfig,
    #[serde(default)]
    pub about: AboutPageConfig,
    #[serde(default)]
    pub todos: TodosPageConfig,
}

/// Client-safe site configuration (no secrets, serializable over the wire)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SiteConfig {
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
    pub author: AuthorConfig,
    #[serde(default)]
    pub social: SocialLinks,
    #[serde(default)]
    pub footer: FooterConfig,
    #[serde(default = "default_nav")]
    pub nav: Vec<NavItem>,
    #[serde(default)]
    pub pages: PagesConfig,
    #[serde(default)]
    pub analytics: AnalyticsConfig,
    #[serde(default)]
    pub donation: DonationConfig,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logo: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,
}

impl Default for SiteConfig {
    fn default() -> Self {
        Self {
            name: default_site_name(),
            tagline: default_tagline(),
            description: default_description(),
            lang: default_lang(),
            default_theme: default_theme(),
            animated_background: default_animated_background(),
            base_url: String::new(),
            author: AuthorConfig::default(),
            social: SocialLinks::default(),
            footer: FooterConfig::default(),
            nav: default_nav(),
            pages: PagesConfig::default(),
            analytics: AnalyticsConfig::default(),
            donation: DonationConfig::default(),
            logo: None,
            favicon: None,
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

fn default_animated_background() -> String {
    "flow-field".to_string()
}

fn default_nav() -> Vec<NavItem> {
    vec![
        NavItem {
            label: "Posts".to_string(),
            path: "/posts".to_string(),
        },
        NavItem {
            label: "Projects".to_string(),
            path: "/projects".to_string(),
        },
        NavItem {
            label: "About".to_string(),
            path: "/about".to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_site_config_default() {
        let config = SiteConfig::default();
        assert_eq!(config.name, "Plinth");
        assert_eq!(config.lang, "en");
        assert_eq!(config.default_theme, "dark");
        assert_eq!(config.animated_background, "flow-field");
        assert_eq!(config.nav.len(), 3);
        assert_eq!(config.nav[0].label, "Posts");
        assert_eq!(config.author.name, "Admin");
        assert_eq!(config.footer.project_name, "Plinth");
        assert_eq!(config.logo, None);
        assert_eq!(config.favicon, None);
    }

    #[test]
    fn test_site_config_serde_roundtrip() {
        let config = SiteConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: SiteConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.name, deserialized.name);
        assert_eq!(config.nav.len(), deserialized.nav.len());
        assert_eq!(deserialized.logo, None);
        assert_eq!(deserialized.favicon, None);
    }

    #[test]
    fn test_site_config_logo_favicon_custom() {
        let config = SiteConfig {
            logo: Some("/my-logo.svg".to_string()),
            favicon: Some("/my-favicon.ico".to_string()),
            ..Default::default()
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"logo\":\"/my-logo.svg\""));
        let deserialized: SiteConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.logo, Some("/my-logo.svg".to_string()));
        assert_eq!(deserialized.favicon, Some("/my-favicon.ico".to_string()));
    }

    #[test]
    fn test_site_config_logo_favicon_excluded_when_none() {
        let config = SiteConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        assert!(!json.contains("\"logo\""));
        assert!(!json.contains("\"favicon\""));
    }

    #[test]
    fn test_empty_json_uses_defaults() {
        let config: SiteConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(config.name, "Plinth");
        assert_eq!(config.default_theme, "dark");
        assert_eq!(config.animated_background, "flow-field");
        assert_eq!(config.nav.len(), 3);
        assert_eq!(config.logo, None);
        assert_eq!(config.favicon, None);
    }

    #[test]
    fn test_donation_config_default() {
        let config = DonationConfig::default();
        assert!(!config.enabled);
        assert!(config.links.is_empty());
        assert!(config.cta_text.is_empty());
    }

    #[test]
    fn test_donation_config_serde_roundtrip() {
        let config = DonationConfig {
            enabled: true,
            links: vec![
                DonationLink {
                    platform: "kofi".to_string(),
                    url: "https://ko-fi.com/test".to_string(),
                    label: String::new(),
                },
                DonationLink {
                    platform: "github_sponsors".to_string(),
                    url: "https://github.com/sponsors/test".to_string(),
                    label: "Sponsor me".to_string(),
                },
            ],
            cta_text: "Support my work!".to_string(),
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: DonationConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_donation_config_empty_json() {
        let config: DonationConfig = serde_json::from_str("{}").unwrap();
        assert!(!config.enabled);
        assert!(config.links.is_empty());
        assert!(config.cta_text.is_empty());
    }

    #[test]
    fn test_site_config_default_has_donation_disabled() {
        let config = SiteConfig::default();
        assert!(!config.donation.enabled);
        assert!(config.donation.links.is_empty());
    }
}
