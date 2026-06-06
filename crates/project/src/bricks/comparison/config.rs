use serde::Deserialize;

use super::{ComparisonRow, ComparisonSection};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComparisonRowConfig {
    pub area: String,
    pub status: String,
    pub notes: String,
}

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
