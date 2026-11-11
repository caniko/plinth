//! `sites.toml` loading.
//!
//! The host app owns the file and the secret delivery; this crate only turns
//! the file into the [`Site`] list the workspace renders.

use std::{
    fmt, fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

/// A named Plinth target the workspace can publish to.
#[derive(Clone, Debug, PartialEq)]
pub struct Site {
    pub id: String,
    pub name: String,
    pub target: Target,
}

/// Where a site's posts live.
#[derive(Clone, Debug, PartialEq)]
pub enum Target {
    /// Plinth personal site: posts go through its admin API.
    Personal {
        api_url: String,
        api_key: ApiKey,
        /// Public address readers use, when it differs from `api_url`.
        public_url: Option<String>,
    },
    /// plinth-project static site: posts are Markdown files in the repo checkout.
    Project {
        root: PathBuf,
        base_url: Option<String>,
    },
}

impl Target {
    /// Short chip label used by the workspace UI.
    pub fn kind_label(&self) -> &'static str {
        match self {
            Target::Personal { .. } => "API",
            Target::Project { .. } => "Files",
        }
    }

    /// Where readers see the site: the public URL of a personal site (its API
    /// address when none is set) or a project site's `base_url`.
    pub fn public_url(&self) -> Option<&str> {
        match self {
            Target::Personal {
                api_url,
                public_url,
                ..
            } => Some(public_url.as_deref().unwrap_or(api_url)),
            Target::Project { base_url, .. } => base_url.as_deref(),
        }
    }

    pub fn is_personal(&self) -> bool {
        matches!(self, Target::Personal { .. })
    }
}

/// An admin API key. Debug output never contains the secret.
#[derive(Clone, PartialEq, Eq)]
pub struct ApiKey(String);

impl ApiKey {
    pub fn new(secret: impl Into<String>) -> Self {
        Self(secret.into())
    }

    pub(crate) fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("***")
    }
}

/// Why a `sites.toml` could not be loaded.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("could not read {path}: {message}")]
    Read { path: PathBuf, message: String },
    #[error("invalid sites config: {message}")]
    Invalid { message: String },
}

/// Read a `sites.toml` file and resolve every site's API key file.
pub fn load_sites(path: &Path) -> Result<Vec<Site>, ConfigError> {
    let text = fs::read_to_string(path).map_err(|error| ConfigError::Read {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;
    let file: SitesFile = toml::from_str(&text).map_err(|error| ConfigError::Invalid {
        message: format!("{}: {error}", path.display()),
    })?;

    let mut sites: Vec<Site> = Vec::with_capacity(file.site.len());
    for entry in file.site {
        let id = entry.id.trim().to_string();
        if id.is_empty() {
            return Err(ConfigError::Invalid {
                message: "every site needs a non-empty `id`".to_string(),
            });
        }
        if sites.iter().any(|site| site.id == id) {
            return Err(ConfigError::Invalid {
                message: format!("duplicate site id `{id}`"),
            });
        }
        let name = entry.name.trim().to_string();
        if name.is_empty() {
            return Err(ConfigError::Invalid {
                message: format!("site `{id}` needs a non-empty `name`"),
            });
        }

        let target = match entry.kind.as_str() {
            "personal" => {
                let api_url = entry.api_url.ok_or_else(|| ConfigError::Invalid {
                    message: format!("site `{id}` needs `api_url` for kind `personal`"),
                })?;
                let key_path = entry.api_key_file.ok_or_else(|| ConfigError::Invalid {
                    message: format!("site `{id}` needs `api_key_file` for kind `personal`"),
                })?;
                let secret = fs::read_to_string(&key_path).map_err(|error| ConfigError::Read {
                    path: key_path.clone(),
                    message: error.to_string(),
                })?;
                let secret = secret.trim().to_string();
                if secret.is_empty() {
                    return Err(ConfigError::Invalid {
                        message: format!(
                            "API key file for site `{id}` is empty: {}",
                            key_path.display()
                        ),
                    });
                }
                Target::Personal {
                    api_url: api_url.trim_end_matches('/').to_string(),
                    api_key: ApiKey(secret),
                    public_url: entry
                        .public_url
                        .filter(|url| !url.trim().is_empty())
                        .map(|url| url.trim_end_matches('/').to_string()),
                }
            }
            "project" => {
                let root = entry.root.ok_or_else(|| ConfigError::Invalid {
                    message: format!("site `{id}` needs `root` for kind `project`"),
                })?;
                Target::Project {
                    root,
                    base_url: entry
                        .base_url
                        .filter(|url| !url.trim().is_empty())
                        .map(|url| url.trim_end_matches('/').to_string()),
                }
            }
            other => {
                return Err(ConfigError::Invalid {
                    message: format!(
                        "site `{id}` has unknown kind `{other}` (expected `personal` or `project`)"
                    ),
                });
            }
        };

        sites.push(Site { id, name, target });
    }

    Ok(sites)
}

#[derive(Debug, Deserialize)]
struct SitesFile {
    #[serde(default)]
    site: Vec<SiteEntry>,
}

#[derive(Debug, Deserialize)]
struct SiteEntry {
    id: String,
    name: String,
    kind: String,
    #[serde(default)]
    api_url: Option<String>,
    #[serde(default)]
    api_key_file: Option<PathBuf>,
    #[serde(default)]
    public_url: Option<String>,
    #[serde(default)]
    root: Option<PathBuf>,
    #[serde(default)]
    base_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write(dir: &TempDir, name: &str, text: &str) -> PathBuf {
        let path = dir.path().join(name);
        fs::write(&path, text).unwrap();
        path
    }

    #[test]
    fn loads_personal_and_project_sites_and_reads_the_key_file() {
        let dir = TempDir::new().unwrap();
        let key = write(&dir, "plinth-api-key", "  super-secret\n");
        let config = write(
            &dir,
            "sites.toml",
            &format!(
                r#"
[[site]]
id = "personal"
name = "tartanoglu.com"
kind = "personal"
api_url = "https://example.com/"
public_url = "https://tartanoglu.com/"
api_key_file = "{}"

[[site]]
id = "pink-raven"
name = "Pink Raven"
kind = "project"
root = "/work/website"
base_url = "https://pinkraven.example"
"#,
                key.display()
            ),
        );

        let sites = load_sites(&config).unwrap();
        assert_eq!(sites.len(), 2);
        let personal = &sites[0];
        assert_eq!(personal.id, "personal");
        assert_eq!(personal.name, "tartanoglu.com");
        match &personal.target {
            Target::Personal {
                api_url,
                api_key,
                public_url,
            } => {
                assert_eq!(api_url, "https://example.com");
                assert_eq!(public_url.as_deref(), Some("https://tartanoglu.com"));
                assert_eq!(api_key.expose(), "super-secret");
                assert_eq!(format!("{api_key:?}"), "***");
                assert!(!format!("{personal:?}").contains("super-secret"));
            }
            other => panic!("expected a personal target, got {other:?}"),
        }
        assert_eq!(
            sites[1].target,
            Target::Project {
                root: PathBuf::from("/work/website"),
                base_url: Some("https://pinkraven.example".to_string()),
            }
        );
    }

    #[test]
    fn rejects_missing_key_file_and_unknown_kind() {
        let dir = TempDir::new().unwrap();
        let config = write(
            &dir,
            "sites.toml",
            r#"
[[site]]
id = "personal"
name = "site"
kind = "personal"
api_url = "https://example.com"
api_key_file = "/nonexistent/plinth-key"
"#,
        );
        assert!(matches!(load_sites(&config), Err(ConfigError::Read { .. })));

        let config = write(
            &dir,
            "sites.toml",
            r#"
[[site]]
id = "x"
name = "x"
kind = "ftp"
"#,
        );
        match load_sites(&config) {
            Err(ConfigError::Invalid { message }) => assert!(message.contains("unknown kind")),
            other => panic!("expected an invalid-config error, got {other:?}"),
        }
    }
}
