use super::InstallSection;
use crate::render::{escape_attr, escape_text};

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
            let command = route.command.as_ref().map_or_else(String::new, |command| {
                render_command(command)
            });
            let note = if route.recommended {
                "Recommended starting point."
            } else {
                "Best fit for this setup."
            };
            format!(
                "<article class=\"install-route{}\"><span class=\"route-meta\">{} {}</span><strong>{}</strong><p>{}</p>{}<a class=\"route-link\" href=\"{}\">Open guide</a></article>",
                if route.recommended { " recommended" } else { "" },
                escape_text(&route.audience),
                badge,
                escape_text(&route.label),
                escape_text(note),
                command,
                escape_attr(&route.href)
            )
        })
        .collect::<String>();
    let secondary = install
        .secondary_routes
        .iter()
        .map(|route| {
            let command = route.command.as_ref().map_or_else(
                || "<p>Open focused guide</p>".to_string(),
                |command| render_command(command),
            );
            format!(
                "<article><span>{}</span><strong>{}</strong>{}<a class=\"route-link\" href=\"{}\">Open guide</a></article>",
                escape_text(&route.audience),
                escape_text(&route.label),
                command,
                escape_attr(&route.href)
            )
        })
        .collect::<String>();
    format!(
        "<section id=\"{}\" class=\"install-section\"><div class=\"section-heading\"><h2>{}</h2><p>{}</p><a href=\"{}\">Open full install guide</a></div><div class=\"install-routes\">{}</div><div class=\"secondary-routes\">{}</div></section>",
        escape_attr(&install.id),
        escape_text(&install.heading),
        escape_text(&install.intro),
        escape_attr(&install.guide_href),
        primary,
        secondary
    )
}

pub fn render_install_fragment(install: &InstallSection) -> String {
    render_install(install)
}

fn render_command(command: &str) -> String {
    format!(
        "<div class=\"command-row\"><pre><code>{}</code></pre><button type=\"button\" class=\"copy-command\" data-copy=\"{}\">Copy command</button></div>",
        escape_text(command),
        escape_attr(command)
    )
}
