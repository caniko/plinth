use serde::Deserialize;

use super::{ComparisonRow, ComparisonSection};

/// Deserialized config for a single comparison row.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComparisonRowConfig {
    /// The area being compared (e.g. "Rendering", "Accessibility").
    pub area: String,
    /// Status string; mapped to CSS class `badge low|mid|high`.
    pub status: String,
    /// Notes or explanation for the status.
    pub notes: String,
}

/// Build a [`ComparisonSection`] model from deserialized config.
///
/// Template markers: `<section class="comparison-section">`,
/// `<table class="coverage-table">`.
pub fn build_comparison(
    id: Option<String>,
    heading: String,
    intro: String,
    rows: Vec<ComparisonRowConfig>,
) -> ComparisonSection {
    ComparisonSection {
        id,
        heading,
        intro,
        rows: rows
            .into_iter()
            .map(|row| ComparisonRow {
                area: row.area,
                status: row.status,
                notes: row.notes,
            })
            .collect(),
    }
}
