use super::ComparisonSection;
use crate::render::{escape_text, id_attr};

/// Render a [`ComparisonSection`] into an HTML string.
///
/// Template: `<section class="comparison-section">` →
/// `<table class="coverage-table">` →
/// `<tr>` with area, `<span class="badge {low|mid|high}">`, and notes.
pub fn render_comparison(comparison: &ComparisonSection) -> String {
    let rows = comparison
        .rows
        .iter()
        .map(|row| {
            format!(
                "<tr><td>{}</td><td><span class=\"badge {}\">{}</span></td><td>{}</td></tr>",
                escape_text(&row.area),
                badge_class(&row.status),
                escape_text(&row.status),
                escape_text(&row.notes)
            )
        })
        .collect::<String>();
    format!(
        "<section{} class=\"comparison-section\"><h2>{}</h2><p class=\"section-subtitle\">{}</p><table class=\"coverage-table\"><thead><tr><th>Area</th><th>Status</th><th>Notes</th></tr></thead><tbody>{}</tbody></table></section>",
        id_attr(comparison.id.as_deref()),
        escape_text(&comparison.heading),
        escape_text(&comparison.intro),
        rows
    )
}

fn badge_class(status: &str) -> &'static str {
    match status.to_ascii_lowercase().as_str() {
        "done" => "high",
        "partial" => "mid",
        _ => "low",
    }
}
