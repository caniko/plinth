use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::config::ConfigError;

use super::{Capability, CapabilityMatrix};

#[derive(Debug, Deserialize)]
struct Matrix {
    #[serde(default)]
    games: BTreeMap<String, MatrixItem>,
    #[serde(default)]
    items: BTreeMap<String, MatrixItem>,
}

#[derive(Debug, Deserialize)]
struct MatrixItem {
    display_name: String,
    overall: String,
    #[serde(flatten)]
    capabilities: BTreeMap<String, String>,
}

/// Load a [`CapabilityMatrix`] from an external TOML file.
///
/// The TOML file should contain an `[items]` table keyed by slug,
/// each with `display_name`, `overall`, and additional capability keys.
/// The older `[games]` table is still accepted for existing project sites.
pub fn load_capability_matrix(
    id: String,
    heading: String,
    intro_html: String,
    source: &Path,
) -> Result<CapabilityMatrix, ConfigError> {
    let raw = std::fs::read_to_string(source).map_err(|source_error| ConfigError::MatrixRead {
        path: source.to_path_buf(),
        source: source_error,
    })?;
    let matrix =
        toml::from_str::<Matrix>(&raw).map_err(|source_error| ConfigError::MatrixParse {
            path: source.to_path_buf(),
            source: source_error,
        })?;
    Ok(CapabilityMatrix {
        id,
        heading,
        intro_html,
        capabilities: matrix_items(matrix)
            .into_iter()
            .map(|(slug, item)| Capability {
                slug,
                display_name: item.display_name,
                overall: item.overall,
                details: item
                    .capabilities
                    .into_iter()
                    .filter(|(key, _)| key.as_str() != "overall")
                    .map(|(key, value)| (labelize(&key), value))
                    .collect(),
            })
            .collect(),
    })
}

fn matrix_items(matrix: Matrix) -> BTreeMap<String, MatrixItem> {
    if matrix.items.is_empty() {
        matrix.games
    } else {
        matrix.items
    }
}

/// Return filesystem paths to watch for live-reload when the matrix source
/// TOML file changes. Resolves relative to the project base directory.
pub fn watch_paths(base: &Path, source: &Path) -> Vec<PathBuf> {
    vec![crate::config::resolve_path(base, source.to_path_buf())]
}

fn labelize(input: &str) -> String {
    input
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
