//! Shared runtime secret loading for Plinth server and CLI binaries.
//!
//! Resolves secrets from private runtime files (explicit file flag/path,
//! `*_FILE` env var, or systemd `CREDENTIALS_DIRECTORY/<name>`) before falling
//! back to legacy direct environment variables.

use std::fmt;
use std::path::{Path, PathBuf};

/// Failure while loading an explicitly configured secret file or credential.
#[derive(Debug)]
pub enum SecretError {
    /// Reading the configured secret file failed.
    ReadFailed {
        path: PathBuf,
        source: std::io::Error,
    },
    /// The configured secret file existed but contained only whitespace/newlines.
    EmptyFile { path: PathBuf },
}

impl fmt::Display for SecretError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadFailed { path, source } => {
                write!(f, "failed to read secret file {}: {source}", path.display())
            }
            Self::EmptyFile { path } => {
                write!(f, "secret file {} is empty", path.display())
            }
        }
    }
}

impl std::error::Error for SecretError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReadFailed { source, .. } => Some(source),
            Self::EmptyFile { .. } => None,
        }
    }
}

/// Read a single-value secret file, trimming trailing CR/LF characters.
pub fn read_secret_file(path: &Path) -> Result<String, SecretError> {
    let raw = std::fs::read_to_string(path).map_err(|source| SecretError::ReadFailed {
        path: path.to_path_buf(),
        source,
    })?;
    let trimmed = raw.trim_end_matches(['\r', '\n']);
    if trimmed.is_empty() {
        return Err(SecretError::EmptyFile {
            path: path.to_path_buf(),
        });
    }
    Ok(trimmed.to_string())
}

/// Resolve a secret using a deterministic file-first precedence:
///
/// 1. `explicit_file` (e.g. `--api-key-file`)
/// 2. `file_env` environment variable (e.g. `PLINTH_API_KEY_FILE`)
/// 3. `$CREDENTIALS_DIRECTORY/<credential_name>` if the file exists
/// 4. `explicit_value` (e.g. `--api-key`)
/// 5. `value_env` environment variable (e.g. `PLINTH_API_KEY`)
pub fn resolve_secret(
    explicit_file: Option<&Path>,
    file_env: &str,
    credential_name: Option<&str>,
    explicit_value: Option<&str>,
    value_env: &str,
) -> Result<Option<String>, SecretError> {
    resolve_secret_with(
        explicit_file,
        file_env,
        credential_name,
        explicit_value,
        value_env,
        |key| std::env::var_os(key),
    )
}

/// Convenience wrapper for server startup paths without CLI flag overrides.
pub fn resolve_from_env(
    file_env: &str,
    credential_name: Option<&str>,
    value_env: &str,
) -> Result<Option<String>, SecretError> {
    resolve_secret(None, file_env, credential_name, None, value_env)
}

fn resolve_secret_with<F>(
    explicit_file: Option<&Path>,
    file_env: &str,
    credential_name: Option<&str>,
    explicit_value: Option<&str>,
    value_env: &str,
    get_env: F,
) -> Result<Option<String>, SecretError>
where
    F: Fn(&str) -> Option<std::ffi::OsString>,
{
    if let Some(path) = explicit_file {
        return read_secret_file(path).map(Some);
    }

    if let Some(path_os) = get_env(file_env).filter(|v| !v.is_empty()) {
        return read_secret_file(Path::new(&path_os)).map(Some);
    }

    if let Some(cred_name) = credential_name
        && let Some(cred_dir) = get_env("CREDENTIALS_DIRECTORY").filter(|v| !v.is_empty())
    {
        let candidate = PathBuf::from(cred_dir).join(cred_name);
        if candidate.exists() {
            return read_secret_file(&candidate).map(Some);
        }
    }

    if let Some(val) = explicit_value.filter(|v| !v.is_empty()) {
        return Ok(Some(val.to_string()));
    }

    if let Some(val) = get_env(value_env)
        .and_then(|v| v.into_string().ok())
        .filter(|v| !v.is_empty())
    {
        return Ok(Some(val));
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::ffi::OsString;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_dir(label: &str) -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "plinth-secret-test-{label}-{}-{n}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn env_map(pairs: &[(&str, &str)]) -> HashMap<String, OsString> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), OsString::from(v)))
            .collect()
    }

    #[test]
    fn trims_trailing_newlines_and_rejects_empty_files() {
        let dir = temp_dir("trim");
        let valid = dir.join("key");
        let empty = dir.join("empty");
        std::fs::write(&valid, "secret-token\r\n").unwrap();
        std::fs::write(&empty, "\n\r\n").unwrap();

        assert_eq!(read_secret_file(&valid).unwrap(), "secret-token");
        assert!(matches!(
            read_secret_file(&empty),
            Err(SecretError::EmptyFile { .. })
        ));
        assert!(matches!(
            read_secret_file(&dir.join("missing")),
            Err(SecretError::ReadFailed { .. })
        ));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn explicit_file_takes_priority_over_env_and_credentials() {
        let dir = temp_dir("priority");
        let explicit = dir.join("explicit");
        let env_file = dir.join("env-file");
        let cred_file = dir.join("api-key");
        std::fs::write(&explicit, "from-explicit\n").unwrap();
        std::fs::write(&env_file, "from-env-file\n").unwrap();
        std::fs::write(&cred_file, "from-cred\n").unwrap();

        let env = env_map(&[
            ("PLINTH_API_KEY_FILE", env_file.to_str().unwrap()),
            ("CREDENTIALS_DIRECTORY", dir.to_str().unwrap()),
            ("PLINTH_API_KEY", "from-env"),
        ]);

        let got = resolve_secret_with(
            Some(&explicit),
            "PLINTH_API_KEY_FILE",
            Some("api-key"),
            Some("from-flag"),
            "PLINTH_API_KEY",
            |k| env.get(k).cloned(),
        )
        .unwrap();
        assert_eq!(got.as_deref(), Some("from-explicit"));

        let got_env_file = resolve_secret_with(
            None,
            "PLINTH_API_KEY_FILE",
            Some("api-key"),
            Some("from-flag"),
            "PLINTH_API_KEY",
            |k| env.get(k).cloned(),
        )
        .unwrap();
        assert_eq!(got_env_file.as_deref(), Some("from-env-file"));

        let env_no_file = env_map(&[
            ("CREDENTIALS_DIRECTORY", dir.to_str().unwrap()),
            ("PLINTH_API_KEY", "from-env"),
        ]);
        let got_cred = resolve_secret_with(
            None,
            "PLINTH_API_KEY_FILE",
            Some("api-key"),
            Some("from-flag"),
            "PLINTH_API_KEY",
            |k| env_no_file.get(k).cloned(),
        )
        .unwrap();
        assert_eq!(got_cred.as_deref(), Some("from-cred"));

        let env_fallback = env_map(&[("PLINTH_API_KEY", "from-env")]);
        let got_flag = resolve_secret_with(
            None,
            "PLINTH_API_KEY_FILE",
            Some("api-key"),
            Some("from-flag"),
            "PLINTH_API_KEY",
            |k| env_fallback.get(k).cloned(),
        )
        .unwrap();
        assert_eq!(got_flag.as_deref(), Some("from-flag"));

        let got_env = resolve_secret_with(
            None,
            "PLINTH_API_KEY_FILE",
            Some("api-key"),
            None,
            "PLINTH_API_KEY",
            |k| env_fallback.get(k).cloned(),
        )
        .unwrap();
        assert_eq!(got_env.as_deref(), Some("from-env"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
