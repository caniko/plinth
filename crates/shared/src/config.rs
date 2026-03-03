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
}

impl Default for SiteConfig {
    fn default() -> Self {
        Self {
            name: default_site_name(),
            tagline: default_tagline(),
            description: default_description(),
            lang: default_lang(),
            default_theme: default_theme(),
            base_url: String::new(),
            author: AuthorConfig::default(),
            social: SocialLinks::default(),
            footer: FooterConfig::default(),
            nav: default_nav(),
            pages: PagesConfig::default(),
            analytics: AnalyticsConfig::default(),
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
        assert_eq!(config.nav.len(), 3);
        assert_eq!(config.nav[0].label, "Posts");
        assert_eq!(config.author.name, "Admin");
        assert_eq!(config.footer.project_name, "Plinth");
    }

    #[test]
    fn test_site_config_serde_roundtrip() {
        let config = SiteConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: SiteConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.name, deserialized.name);
        assert_eq!(config.nav.len(), deserialized.nav.len());
    }

    #[test]
    fn test_empty_json_uses_defaults() {
        let config: SiteConfig = serde_json::from_str("{}").unwrap();
        assert_eq!(config.name, "Plinth");
        assert_eq!(config.default_theme, "dark");
        assert_eq!(config.nav.len(), 3);
    }
}
