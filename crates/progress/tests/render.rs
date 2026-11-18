//! Deterministic rendering, escaping and theme loading for the offline page.

use std::path::{Path, PathBuf};

use plinth_progress::{Progress, load_theme, parse, render};
use plinth_project::ProjectTheme;

fn crate_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn example() -> Progress {
    let path = crate_path("examples/progress.toml");
    let source = std::fs::read_to_string(&path).expect("example is readable");
    parse(&source, &path).expect("example parses")
}

fn document(source: &str) -> Progress {
    parse(source, Path::new("progress.toml")).expect("fixture parses")
}

fn theme_fixture(name: &str) -> PathBuf {
    crate_path(&format!("tests/fixtures/theme/{name}"))
}

#[test]
fn render_snapshot() {
    let html = render(&example(), &ProjectTheme::default());
    let path = crate_path("tests/snapshots/example.html");
    if std::env::var("UPDATE_SNAPSHOTS").is_ok_and(|value| value == "1") {
        std::fs::create_dir_all(path.parent().expect("snapshot has a parent")).unwrap();
        std::fs::write(&path, &html).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path)
        .expect("snapshot exists; regenerate it with UPDATE_SNAPSHOTS=1");
    assert_eq!(html, expected);
}

#[test]
fn render_escapes_untrusted_text() {
    let progress = document(
        r##"
title = "Escapes <script>alert(1)</script>"
[[entries]]
version = "1.0.0"
date = "2026-11-17"
summary = "<script>alert(1)</script></style><svg onload=alert(1)>"
done = ["</style><img src=x onerror=alert(1)>"]
next = []
risks = []
sections = []
[[entries.milestones]]
name = "<svg/onload=alert(1)>"
status = "done"
label = "</style><script>alert(1)</script>"
"##,
    );
    let html = render(&progress, &ProjectTheme::default());

    for raw in ["<script", "<svg", "<img"] {
        assert!(!html.contains(raw), "{raw} reached the output");
    }
    assert_eq!(html.matches("</style>").count(), 1, "only the style closes");
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    assert!(html.contains("&lt;svg onload=alert(1)&gt;"));
    assert!(html.contains("&lt;/style&gt;&lt;img src=x onerror=alert(1)&gt;"));
}

#[test]
fn render_has_no_external_resources() {
    let html = render(&example(), &ProjectTheme::default());
    for needle in ["<script", "<link", "<img", "src=", "url(", "@import"] {
        assert!(!html.contains(needle), "{needle} reached the output");
    }
    assert!(html.contains("default-src 'none'"));
}

#[test]
fn render_history_and_statuses() {
    let progress = document(
        r##"
title = "Statuses"
[[entries]]
version = "2.0.0"
date = "2026-11-17"
summary = "Newest."
done = ["One done item."]
next = []
risks = []
sections = []
[[entries.milestones]]
name = "A"
status = "done"
[[entries.milestones]]
name = "B"
status = "in_progress"
label = "In flight"
[[entries.milestones]]
name = "C"
status = "blocked"
[[entries.milestones]]
name = "D"
status = "planned"
[[entries.milestones]]
name = "E"
status = "dropped"
[[entries]]
version = "1.0.0"
date = "2026-10-01"
summary = "Older."
done = []
next = []
risks = []
milestones = []
sections = []
"##,
    );
    let html = render(&progress, &ProjectTheme::default());

    for class in [
        "status-done",
        "status-in_progress",
        "status-blocked",
        "status-planned",
        "status-dropped",
    ] {
        assert!(html.contains(class), "missing {class}");
    }
    assert!(
        html.contains(r#"<span class="status status-in_progress"><span class="status-word">In progress</span> · In flight</span>"#)
    );
    assert!(html.contains(
        r#"<span class="status status-blocked"><span class="status-word">Blocked</span></span>"#
    ));
    assert!(html.contains("<h2>History</h2>"));
    assert!(html.contains("<summary>1.0.0 · 2026-10-01 · Older.</summary>"));
    assert_eq!(html.matches("<details>").count(), 1);
    assert_eq!(html.matches("Older.").count(), 2, "summary and body");
}

#[test]
fn render_inline_links_and_code() {
    let progress = document(
        r##"
title = "Inline"
[[entries]]
version = "1.0.0"
date = "2026-11-17"
summary = "Read [the notes](https://example.com/notes) and run `plinth-progress check`."
done = []
next = []
risks = []
milestones = []
sections = []
"##,
    );
    let html = render(&progress, &ProjectTheme::default());
    assert!(html.contains(
        r#"<a href="https://example.com/notes" target="_blank" rel="noopener noreferrer">the notes</a>"#
    ));
    assert!(html.contains("<code>plinth-progress check</code>"));
}

#[test]
fn render_rejects_javascript_href() {
    let progress = document(
        r##"
title = "Unsafe link"
[[entries]]
version = "1.0.0"
date = "2026-11-17"
summary = "Run [this](javascript:alert(1)) now."
done = []
next = []
risks = []
milestones = []
sections = []
"##,
    );
    let html = render(&progress, &ProjectTheme::default());
    assert!(!html.contains("href=\"javascript:"), "{html}");
    assert!(!html.contains("<a "), "{html}");
    assert!(html.contains("[this](javascript:alert(1))"));
}

#[test]
fn theme_config_reads_only_theme() {
    let (theme, warnings) =
        load_theme(&theme_fixture("incomplete-site.toml")).expect("theme loads");
    assert_eq!(theme.paper.as_deref(), Some("#101418"));
    assert_eq!(theme.ink.as_deref(), Some("#e6edf3"));
    assert_eq!(theme.accent.as_deref(), Some("#58a6ff"));
    assert!(warnings.is_empty());
}

#[test]
fn theme_config_rejects_broker_css() {
    let path = theme_fixture("broker.toml");
    let report = load_theme(&path).unwrap_err();
    assert_eq!(
        report.render(),
        format!(
            "error: {}: $: broker_css is not supported by plinth-progress; set explicit [theme] colours",
            path.display()
        )
    );
}

#[test]
fn theme_config_drops_font_url() {
    let (theme, warnings) = load_theme(&theme_fixture("font.toml")).expect("theme loads");
    assert_eq!(theme.font_family.as_deref(), Some("Example Sans"));
    assert_eq!(theme.font_url, None);
    assert_eq!(
        warnings,
        ["font_url is ignored: the progress page makes no network requests"]
    );

    let html = render(&example(), &theme);
    assert!(!html.contains("url("), "{html}");
    assert!(!html.contains("@font-face"), "{html}");
    assert!(html.contains("--pp-font-family"));
}

#[test]
fn theme_config_rejects_unsafe_values() {
    let dir = std::env::temp_dir().join(format!("plinth-progress-theme-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let config = dir.join("plinth-project.toml");
    let load = |theme: &str| {
        std::fs::write(&config, format!("[theme]\n{theme}\n")).unwrap();
        load_theme(&config)
    };

    for safe in [
        "paper = '#0B1220'",
        "line = 'rgba(148, 163, 184, 0.24)'",
        "accent = 'oklch(70% 0.1 200 / 50%)'",
        "accent_soft = 'color-mix(in srgb, red 20%, transparent)'",
        "ink = 'white'",
        "font_family = 'Inter Variable'",
    ] {
        assert!(load(safe).is_ok(), "{safe} must load");
    }
    for unsafe_value in [
        r#"font_family = "</style><h1>Injected</h1><style>""#,
        r"paper = 'u\rl(https://example.com/x)'",
        r#"paper = 'image-set("https://example.com/x" 1x)'"#,
        "paper = 'url(https://example.com/x)'",
        "paper = 'red/**/'",
        "paper = '</style>'",
        r"paper = '\72 ed'",
    ] {
        assert!(
            load(unsafe_value).is_err(),
            "{unsafe_value} must be rejected"
        );
    }
    std::fs::remove_dir_all(&dir).unwrap();
}
