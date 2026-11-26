use super::{InstallRoute, InstallSection};
use crate::render::{escape_attr, escape_text};

/// Render an [`InstallSection`] into an HTML string.
///
/// Template: `<section class="install-section">` →
/// `<div class="install-routes">` for primary routes and
/// `<div class="secondary-routes">` for secondary ones.  Each route
/// is `<article class="install-route">` with optional
/// `<div class="command-row">` containing a copyable `<pre><code>` block.
pub fn render_install(install: &InstallSection) -> String {
    let primary = install
        .primary_routes
        .iter()
        .map(|route| {
            let badge = if route.recommended {
                "<span class=\"route-badge\">Recommended</span>"
            } else {
                ""
            };
            format!(
                "<article class=\"install-route{}\"{}><span class=\"route-meta\">{} {}</span><strong>{}</strong>{}</article>",
                if route.recommended { " recommended" } else { "" },
                platforms_attr(route),
                escape_text(&route.audience),
                badge,
                escape_text(&route.label),
                render_route_details(route),
            )
        })
        .collect::<String>();
    let secondary = install
        .secondary_routes
        .iter()
        .map(|route| {
            format!(
                "<article{}><span>{}</span><strong>{}</strong>{}</article>",
                platforms_attr(route),
                escape_text(&route.audience),
                escape_text(&route.label),
                render_route_details(route),
            )
        })
        .collect::<String>();
    format!(
        "<section id=\"{}\" class=\"install-section\"><div class=\"section-heading\"><h2>{}</h2><p>{}</p><a href=\"{}\">{}</a></div><div class=\"install-routes\">{}</div><div class=\"secondary-routes\">{}</div></section>",
        escape_attr(&install.id),
        escape_text(&install.heading),
        escape_text(&install.intro),
        escape_attr(&install.guide_href),
        escape_text(nonblank(&install.guide_label).unwrap_or("Open full install guide")),
        primary,
        secondary
    )
}

fn render_route_details(route: &InstallRoute) -> String {
    let note = nonblank(&route.note)
        .map_or_else(String::new, |note| format!("<p>{}</p>", escape_text(note)));
    let command = route
        .command
        .as_deref()
        .map_or_else(String::new, render_command);
    format!(
        "{note}{command}<a class=\"route-link\" href=\"{}\">{}</a>",
        escape_attr(&route.href),
        escape_text(nonblank(&route.link_label).unwrap_or("Open guide")),
    )
}

fn nonblank(value: &Option<String>) -> Option<&str> {
    value.as_deref().filter(|text| !text.trim().is_empty())
}

/// Render an [`InstallSection`] as an HTML fragment (delegates to
/// [`render_install`]).  Provided as a public entry point for
/// external use (e.g. CLI previews).
pub fn render_install_fragment(install: &InstallSection) -> String {
    render_install(install)
}

/// ` data-platforms="linux macos"` for tagged routes, which the page script
/// filters by the visitor's OS; empty for untagged routes.
fn platforms_attr(route: &InstallRoute) -> String {
    if route.platforms.is_empty() {
        return String::new();
    }
    let tokens = route
        .platforms
        .iter()
        .map(|platform| platform.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    format!(" data-platforms=\"{tokens}\"")
}

fn render_command(command: &str) -> String {
    format!(
        "<div class=\"command-row\"><pre><code>{}</code></pre><button type=\"button\" class=\"copy-command\" data-copy=\"{}\">Copy command</button></div>",
        escape_text(command),
        escape_attr(command)
    )
}
