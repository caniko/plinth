//! Deterministic, self-contained HTML rendering for one progress document.
//!
//! The page makes no network requests: the theme variables and the local
//! layout stylesheet are inlined, and every text field is escaped. Output
//! depends only on the document and the theme.

use std::fmt::Write as _;

use plinth_project::{ProjectTheme, render_inline_text, theme_styles};

use crate::{Entry, Milestone, Progress};

/// Layout stylesheet, inlined after the theme's custom properties.
const STYLE: &str = include_str!("style.css");

/// Renders one offline HTML page: newest entry in full, older ones collapsed.
pub fn render(progress: &Progress, theme: &ProjectTheme) -> String {
    let mut html = String::with_capacity(4096);
    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
    html.push_str("<meta charset=\"utf-8\">\n");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    html.push_str("<meta name=\"robots\" content=\"noindex,nofollow\">\n");
    html.push_str(concat!(
        "<meta http-equiv=\"Content-Security-Policy\" ",
        "content=\"default-src 'none'; style-src 'unsafe-inline'; base-uri 'none'; form-action 'none'\">\n"
    ));
    let _ = writeln!(html, "<title>{}</title>", escape_text(&progress.title));
    let _ = writeln!(
        html,
        "<style>\n{}{}</style>\n</head>\n<body>\n<main class=\"page\">",
        theme_styles(theme),
        STYLE
    );

    html.push_str("<header class=\"page-header\">\n");
    let _ = writeln!(html, "<h1>{}</h1>", render_inline_text(&progress.title));
    if let Some(newest) = progress.entries.first() {
        let _ = writeln!(
            html,
            "<p class=\"meta\">Version {} · {}</p>",
            escape_text(&newest.version),
            escape_text(&newest.date)
        );
    }
    html.push_str("</header>\n");

    if let Some(newest) = progress.entries.first() {
        html.push_str(&entry(newest, 2));
    }
    if let Some(older) = progress.entries.get(1..).filter(|older| !older.is_empty()) {
        html.push_str("<h2>History</h2>\n");
        for older_entry in older {
            let _ = writeln!(
                html,
                "<details>\n<summary>{} · {} · {}</summary>",
                escape_text(&older_entry.version),
                escape_text(&older_entry.date),
                render_inline_text(&older_entry.summary)
            );
            html.push_str(&entry(older_entry, 3));
            html.push_str("</details>\n");
        }
    }

    html.push_str("</main>\n</body>\n</html>\n");
    html
}

/// One entry: summary, milestone table, Done/Next/Risks and sections.
///
/// `level` is the heading level for the lists and sections, so a collapsed
/// history entry nests below the page's `History` heading.
fn entry(entry: &Entry, level: usize) -> String {
    let mut html = String::new();
    html.push_str("<article class=\"entry\">\n");
    let _ = writeln!(
        html,
        "<p class=\"summary\">{}</p>",
        render_inline_text(&entry.summary)
    );

    if !entry.milestones.is_empty() {
        html.push_str(concat!(
            "<div class=\"table-wrap\">\n<table>\n<thead>\n<tr>",
            "<th scope=\"col\">Milestone</th><th scope=\"col\">Status</th>",
            "</tr>\n</thead>\n<tbody>\n"
        ));
        for milestone in &entry.milestones {
            html.push_str(&milestone_row(milestone));
        }
        html.push_str("</tbody>\n</table>\n</div>\n");
    }

    list(&mut html, "Done", &entry.done, level);
    list(&mut html, "Next", &entry.next, level);
    list(&mut html, "Risks", &entry.risks, level);

    for section in &entry.sections {
        let _ = writeln!(
            html,
            "<h{level}>{}</h{level}>\n{}",
            render_inline_text(&section.heading),
            paragraphs(&section.body)
        );
    }

    html.push_str("</article>\n");
    html
}

fn milestone_row(milestone: &Milestone) -> String {
    let status = milestone.status;
    // The status word is always shown, so a custom label never leaves colour
    // as the only signal.
    // The word gets its own span so it never breaks when a long label wraps.
    let word = format!("<span class=\"status-word\">{}</span>", status.label());
    let text = match milestone.label.as_deref() {
        Some(label) => format!("{word} · {}", render_inline_text(label)),
        None => word,
    };
    format!(
        "<tr><td>{}</td><td><span class=\"status status-{}\">{text}</span></td></tr>\n",
        render_inline_text(&milestone.name),
        status.as_str(),
    )
}

fn list(html: &mut String, heading: &str, items: &[String], level: usize) {
    if items.is_empty() {
        return;
    }
    let _ = writeln!(html, "<h{level}>{heading}</h{level}>");
    html.push_str("<ul>\n");
    for item in items {
        let _ = writeln!(html, "<li>{}</li>", render_inline_text(item));
    }
    html.push_str("</ul>\n");
}

/// Splits a body into paragraphs on blank lines; single newlines become `<br>`.
fn paragraphs(body: &str) -> String {
    let mut out = String::new();
    let mut lines: Vec<&str> = Vec::new();
    for line in body.lines().chain([""]) {
        if line.trim().is_empty() {
            if !lines.is_empty() {
                out.push_str("<p>");
                for (index, line) in lines.iter().enumerate() {
                    if index > 0 {
                        out.push_str("<br>\n");
                    }
                    out.push_str(&render_inline_text(line));
                }
                out.push_str("</p>\n");
                lines.clear();
            }
        } else {
            lines.push(line);
        }
    }
    out
}

/// Escapes text for a context that accepts no markup, such as `<title>`.
fn escape_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
