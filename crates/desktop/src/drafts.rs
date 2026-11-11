//! Editor autosave files: one JSON draft per site under `drafts_dir`.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use crate::Draft;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// `<drafts_dir>/<site id>.json`. Bytes outside `[A-Za-z0-9-]` become `_xx`
/// (hex), so a config-provided id cannot escape the directory and two
/// distinct ids never share a file.
pub(crate) fn path(drafts_dir: &Path, site_id: &str) -> PathBuf {
    let mut safe = String::with_capacity(site_id.len());
    for byte in site_id.bytes() {
        if byte.is_ascii_alphanumeric() || byte == b'-' {
            safe.push(char::from(byte));
        } else {
            safe.push_str(&format!("_{byte:02x}"));
        }
    }
    drafts_dir.join(format!("{safe}.json"))
}

pub(crate) fn save(drafts_dir: &Path, site_id: &str, draft: &Draft) -> std::io::Result<()> {
    fs::create_dir_all(drafts_dir)?;
    let target = path(drafts_dir, site_id);
    let json = serde_json::to_vec_pretty(draft).map_err(std::io::Error::other)?;
    let temporary = drafts_dir.join(format!(
        ".{}.{}.{}.tmp",
        target
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("draft"),
        std::process::id(),
        TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&temporary, json)?;
    fs::rename(&temporary, &target)?;
    Ok(())
}

pub(crate) fn load(drafts_dir: &Path, site_id: &str) -> Option<Draft> {
    let text = fs::read_to_string(path(drafts_dir, site_id)).ok()?;
    serde_json::from_str(&text).ok()
}

pub(crate) fn delete(drafts_dir: &Path, site_id: &str) -> std::io::Result<()> {
    match fs::remove_file(path(drafts_dir, site_id)) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn draft_round_trips_and_deletes() {
        let dir = TempDir::new().unwrap();
        let draft = Draft {
            title: "Hello".to_string(),
            slug: "hello".to_string(),
            description: "summary".to_string(),
            tags: vec!["rust".to_string()],
            body: "body".to_string(),
            published: true,
            replaces: Some(crate::Replaces {
                slug: "hello".to_string(),
                version: "7".to_string(),
            }),
        };

        save(dir.path(), "personal", &draft).unwrap();
        assert_eq!(load(dir.path(), "personal"), Some(draft));

        delete(dir.path(), "personal").unwrap();
        assert_eq!(load(dir.path(), "personal"), None);
        // Deleting a missing draft is not an error.
        delete(dir.path(), "personal").unwrap();
    }

    #[test]
    fn site_ids_cannot_escape_the_drafts_directory() {
        let dir = TempDir::new().unwrap();
        save(dir.path(), "../escape", &Draft::default()).unwrap();
        assert!(path(dir.path(), "../escape").starts_with(dir.path()));
        assert_eq!(load(dir.path(), "../escape"), Some(Draft::default()));
    }

    #[test]
    fn distinct_site_ids_get_distinct_files() {
        let dir = Path::new("/drafts");
        assert_ne!(path(dir, "personal.example"), path(dir, "personal_example"));
        assert_eq!(path(dir, "../x"), dir.join("_2e_2e_2fx.json"));
    }
}
