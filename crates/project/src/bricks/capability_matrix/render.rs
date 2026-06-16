use super::CapabilityMatrix;
use crate::render::{escape_attr, escape_text};

/// Render a [`CapabilityMatrix`] into an HTML string.
///
/// Template: `<section class="landing-content">` →
/// `<table class="games-matrix">` →
/// `<tr>` with display name, `<span class="status-pill">`, and
/// `<span class="capability-pill">` list per row.
pub fn render_capability_matrix(matrix: &CapabilityMatrix) -> String {
    let rows = matrix
        .capabilities
        .iter()
        .map(|capability| {
            let details = capability
                .details
                .iter()
                .map(|(name, value)| {
                    format!(
                        "<span class=\"capability-pill capability-{}\"><span class=\"capability-name\">{}</span><span class=\"capability-value\">{}</span></span>",
                        status_key(value),
                        escape_text(name),
                        escape_text(value)
                    )
                })
                .collect::<String>();
            format!(
                "<tr><td><strong>{}</strong><div class=\"game-slug\">{}</div></td><td><span class=\"status-pill status-{}\">{}</span></td><td><div class=\"capability-list\">{}</div></td></tr>",
                escape_text(&capability.display_name),
                escape_text(&capability.slug),
                status_key(&capability.overall),
                escape_text(&capability.overall),
                details
            )
        })
        .collect::<String>();
    format!(
        "<section id=\"{}\" class=\"landing-content\"><h2>{}</h2><p>{}</p><table class=\"games-matrix\"><thead><tr><th scope=\"col\">Game</th><th scope=\"col\">Support tier</th><th scope=\"col\">Capabilities</th></tr></thead><tbody>{}</tbody></table></section>",
        escape_attr(&matrix.id),
        escape_text(&matrix.heading),
        matrix.intro_html,
        rows
    )
}

fn status_key(value: &str) -> String {
    value.to_ascii_lowercase().replace(' ', "_")
}
