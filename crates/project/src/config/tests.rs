use super::load_project_site;
#[cfg(feature = "brick-capability-matrix")]
use crate::ProjectSection;
use plinth_person::LinkKind;

#[cfg(all(feature = "brick-hero", feature = "brick-feature-grid"))]
#[test]
fn guided_landing_config_packages_shared_styles_font_and_task_links() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("plinth-project.toml");
    std::fs::write(dir.path().join("preview.png"), b"preview bytes").unwrap();
    std::fs::write(
        &config,
        r#"
[site]
title = "Example"
description = "Guided landing"
[theme]
font_family = 'Readable "Font"'
font_url = '/assets/font.woff2'
[[nav]]
label = "Enter"
href = "/app"
primary = true
[[assets]]
source = "preview.png"
target = "assets/preview.png"
[[pages]]
slug = "index"
title = "Example"
[[pages.sections]]
type = "hero"
title = "Find <records>"
tagline = "Search and compare"
subtitle = ""
eyebrow = "Research"
note = "Rights remain visible"
preview_src = '/assets/preview.png?x="quoted"'
preview_mobile_src = '/assets/preview-mobile.png'
preview_alt = 'A "preview"'
[[pages.sections]]
type = "feature_grid"
heading = "Start a task"
eyebrow = "Pick <one>"
intro = "Choose an authorized destination"
[[pages.sections.features]]
title = "Find"
description = "Search records"
action = { label = "Open <search>", href = "/search?q=a&view=list", primary = true }
"#,
    )
    .unwrap();
    let site = load_project_site(&config).unwrap();
    let out = dir.path().join("out");
    crate::render_static(&site, &crate::RenderOptions::new(&out)).unwrap();
    let html = std::fs::read_to_string(out.join("index.html")).unwrap();
    assert!(html.contains("hero--split"));
    assert!(html.contains("href=\"/app\" class=\"nav-primary\""));
    assert!(html.contains("media=\"(max-width: 700px)\" srcset=\"/assets/preview-mobile.png\""));
    assert!(html.contains("Pick &lt;one&gt;"));
    assert!(html.contains("Find &lt;records&gt;"));
    assert!(html.contains("A &quot;preview&quot;"));
    assert!(html.contains("<h2>Start a task</h2>"));
    assert!(html.contains("<h3>Find</h3>"));
    assert!(html.contains("href=\"/search?q=a&amp;view=list\""));
    assert!(html.contains("Open &lt;search&gt;"));
    assert!(html.contains("tartan-responsive-grid"));
    let css = std::fs::read_to_string(out.join("style.css")).unwrap();
    assert!(css.contains(tartan_ui_assets::LAYOUT_STYLES));
    assert!(css.contains(r#"font-family:"Readable \"Font\"""#));
    assert!(css.contains("font-weight:100 900"));
    assert_eq!(
        std::fs::read(out.join("assets/preview.png")).unwrap(),
        b"preview bytes"
    );
}

#[test]
fn unknown_project_config_field_fails_fast() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("plinth-project.toml");
    std::fs::write(
        &config,
        r#"
[site]
title = "Example"
description = "Example site"
unexpected = "not allowed"

[[pages]]
slug = "index"
title = "Example"
"#,
    )
    .unwrap();

    let error = match load_project_site(&config) {
        Ok(_) => panic!("unknown site field should fail"),
        Err(error) => error.to_string(),
    };
    assert!(error.contains("unknown field"));
    assert!(error.contains("unexpected"));
}

#[test]
fn project_references_parse_and_order_canonical_links_first() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("plinth-project.toml");
    std::fs::write(
        &config,
        r#"
[site]
title = "Example"
description = "Example site"

[[projects]]
title = "Tool"
url = "https://tool.example"
source_url = "https://source.example"
demo_url = "https://demo.example"

[[projects.links]]
label = "Docs"
href = "https://docs.example"
kind = "docs"

[[pages]]
slug = "index"
title = "Example"
"#,
    )
    .unwrap();

    let site = load_project_site(&config).unwrap();
    assert_eq!(site.projects.len(), 1);
    let project = &site.projects[0];
    assert_eq!(project.title, "Tool");
    let links = project.links();
    assert_eq!(links[0].kind, LinkKind::ProjectSite);
    assert_eq!(links[1].kind, LinkKind::Source);
    assert_eq!(links[2].kind, LinkKind::Demo);
    assert_eq!(links[3].kind, LinkKind::Docs);
}

#[test]
fn unknown_project_reference_field_fails_fast() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("plinth-project.toml");
    std::fs::write(
        &config,
        r#"
[site]
title = "Example"
description = "Example site"

[[projects]]
title = "Tool"
url = "https://tool.example"
unexpected = "not allowed"

[[pages]]
slug = "index"
title = "Example"
"#,
    )
    .unwrap();

    let error = match load_project_site(&config) {
        Ok(_) => panic!("unknown project field should fail"),
        Err(error) => error.to_string(),
    };
    assert!(error.contains("unknown field"));
    assert!(error.contains("unexpected"));
}

#[cfg(all(feature = "brick-hero", feature = "brick-person-mention"))]
#[test]
fn person_config_renders_author_metadata_and_links() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("plinth-project.toml");
    let out = dir.path().join("public");
    std::fs::write(
        &config,
        r#"
[site]
title = "Example"
description = "Example site"
primary_person = "maintainer"

[[people]]
id = "maintainer"
name = "Maintainer"
url = "https://person.example"
role = "Project lead"

[[people.links]]
label = "Contact"
href = "https://person.example/contact"
kind = "contact"

[[pages]]
slug = "index"
title = "Example"

[[pages.sections]]
type = "hero"
title = "Example"
tagline = "Built plainly"
subtitle = "A project site"
person = "maintainer"

[[pages.sections]]
type = "person_mention"
id = "maintainer"
heading = "Maintainer"
intro = "Who keeps this project moving."
person = "maintainer"
"#,
    )
    .unwrap();

    let site = load_project_site(&config).unwrap();
    crate::render_static(&site, &crate::RenderOptions::new(&out)).unwrap();
    let html = std::fs::read_to_string(out.join("index.html")).unwrap();
    assert!(html.contains("application/ld+json"));
    assert!(html.contains("hero-byline"));
    assert!(html.contains("person-attribution"));
    assert!(html.contains("person-mention"));
    assert!(html.contains("link-contact"));
}

#[cfg(feature = "brick-capability-matrix")]
#[test]
fn capability_matrix_source_contributes_watch_path() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("plinth-project.toml");
    let matrix_dir = dir.path().join("data");
    std::fs::create_dir_all(&matrix_dir).unwrap();
    std::fs::write(
        &config,
        r#"
[site]
title = "Example"
description = "Example site"

[[pages]]
slug = "index"
title = "Example"

[[pages.sections]]
type = "capability_matrix"
id = "matrix"
heading = "Matrix"
intro_html = "Intro"
source = "data/capability-matrix.toml"
"#,
    )
    .unwrap();

    let paths = super::project_watch_paths(&config).unwrap();
    assert!(paths.contains(&matrix_dir));
}

#[cfg(feature = "brick-capability-matrix")]
#[test]
fn capability_matrix_loads_legacy_games_source() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("plinth-project.toml");
    let matrix_dir = dir.path().join("data");
    let matrix = matrix_dir.join("capability-matrix.toml");
    std::fs::create_dir_all(&matrix_dir).unwrap();
    std::fs::write(
        &matrix,
        r#"
[games.chess]
display_name = "Chess"
overall = "High"
rules = "Complete"
"#,
    )
    .unwrap();
    std::fs::write(
        &config,
        r#"
[site]
title = "Example"
description = "Example site"

[[pages]]
slug = "index"
title = "Example"

[[pages.sections]]
type = "capability_matrix"
id = "matrix"
heading = "Matrix"
intro_html = "Intro"
source = "data/capability-matrix.toml"
"#,
    )
    .unwrap();

    let site = load_project_site(&config).unwrap();
    let ProjectSection::CapabilityMatrix(matrix) = &site.pages[0].sections[0] else {
        panic!("expected capability matrix section");
    };
    assert_eq!(matrix.capabilities[0].slug, "chess");
    assert_eq!(matrix.capabilities[0].display_name, "Chess");
    assert_eq!(
        matrix.capabilities[0].details[0],
        ("Rules".into(), "Complete".into())
    );
}

#[cfg(feature = "brick-capability-matrix")]
#[test]
fn capability_matrix_loads_neutral_items_source() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("plinth-project.toml");
    let matrix_dir = dir.path().join("data");
    let matrix = matrix_dir.join("capability-matrix.toml");
    std::fs::create_dir_all(&matrix_dir).unwrap();
    std::fs::write(
        &matrix,
        r#"
[items.clickhouse]
display_name = "ClickHouse"
overall = "Built-in adapter"
advertised_features = "15"
corpus_outcomes = "29 supported / 5 rejected"
"#,
    )
    .unwrap();
    std::fs::write(
        &config,
        r#"
[site]
title = "Example"
description = "Example site"

[[pages]]
slug = "index"
title = "Example"

[[pages.sections]]
type = "capability_matrix"
id = "matrix"
heading = "Matrix"
intro_html = "Intro"
source = "data/capability-matrix.toml"
"#,
    )
    .unwrap();

    let site = load_project_site(&config).unwrap();
    let ProjectSection::CapabilityMatrix(matrix) = &site.pages[0].sections[0] else {
        panic!("expected capability matrix section");
    };
    assert_eq!(matrix.capabilities[0].slug, "clickhouse");
    assert_eq!(matrix.capabilities[0].display_name, "ClickHouse");
    assert_eq!(matrix.capabilities[0].overall, "Built-in adapter");
    assert_eq!(
        matrix.capabilities[0].details,
        vec![
            ("Advertised Features".into(), "15".into()),
            ("Corpus Outcomes".into(), "29 supported / 5 rejected".into())
        ]
    );
}

#[test]
fn preset_catppuccin_latte_fills_all_theme_colors() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("plinth-project.toml");
    std::fs::write(
        &config,
        r##"
[site]
title = "Example"
description = "Example site"

[theme]
preset = "catppuccin-latte"

[[pages]]
slug = "index"
title = "Example"
"##,
    )
    .unwrap();

    let site = load_project_site(&config).unwrap();
    crate::render_static(&site, &crate::RenderOptions::new(dir.path())).unwrap();
    let css = std::fs::read_to_string(dir.path().join("style.css")).unwrap();
    assert!(css.contains("--pp-paper:#eff1f5"));
    assert!(css.contains("--pp-ink:#4c4f69"));
    assert!(css.contains("--pp-accent:#d20f39"));
}

#[test]
fn preset_gruvbox_hard_dark_matches_pink_raven_theme() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("plinth-project.toml");
    std::fs::write(
        &config,
        r##"
[site]
title = "Example"
description = "Example site"

[theme]
preset = "gruvbox-hard-dark"

[[pages]]
slug = "index"
title = "Example"
"##,
    )
    .unwrap();

    let site = load_project_site(&config).unwrap();
    crate::render_static(&site, &crate::RenderOptions::new(dir.path())).unwrap();
    let css = std::fs::read_to_string(dir.path().join("style.css")).unwrap();
    for variable in [
        "--pp-paper:#1d2021",
        "--pp-surface:#282828",
        "--pp-ink:#fbf1c7",
        "--pp-ink-soft:#d5c4a1",
        "--pp-line:rgba(235, 219, 178, 0.16)",
        "--pp-accent:#fe8019",
        "--pp-accent-soft:rgba(254, 128, 25, 0.16)",
        "--pp-secondary:#b8bb26",
        "--pp-warning:#fabd2f",
        "--pp-rust:#fb4934",
    ] {
        assert!(css.contains(variable), "missing {variable}");
    }
}

#[test]
fn preset_with_individual_override() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("plinth-project.toml");
    std::fs::write(
        &config,
        r##"
[site]
title = "Example"
description = "Example site"

[theme]
preset = "catppuccin-latte"
accent = "#ff00ff"

[[pages]]
slug = "index"
title = "Example"
"##,
    )
    .unwrap();

    let site = load_project_site(&config).unwrap();
    crate::render_static(&site, &crate::RenderOptions::new(dir.path())).unwrap();
    let css = std::fs::read_to_string(dir.path().join("style.css")).unwrap();
    assert!(css.contains("--pp-paper:#eff1f5"));
    assert!(css.contains("--pp-accent:#ff00ff"));
}

#[test]
fn unknown_preset_is_noop() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("plinth-project.toml");
    std::fs::write(
        &config,
        r##"
[site]
title = "Example"
description = "Example site"

[theme]
preset = "nonexistent"

[[pages]]
slug = "index"
title = "Example"
"##,
    )
    .unwrap();

    let site = load_project_site(&config).unwrap();
    crate::render_static(&site, &crate::RenderOptions::new(dir.path())).unwrap();
    let css = std::fs::read_to_string(dir.path().join("style.css")).unwrap();
    assert!(!css.contains("--pp-paper:"));
    assert!(!css.contains("--pp-ink:"));
}

#[cfg(feature = "brick-project-grid")]
#[test]
fn project_grid_renders_site_projects_and_escapes_copy() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("plinth-project.toml");
    let out = dir.path().join("public");
    std::fs::write(
        &config,
        r#"
[site]
title = "Example"
description = "Example site"

[[projects]]
title = "Tool <beta>"
url = "https://tool.example/?q=\"x\""
description = "Useful & <fast>"
source_url = "https://source.example"

[[pages]]
slug = "index"
title = "Example"

[[pages.sections]]
type = "project_grid"
id = "ecosystem"
heading = "Projects"
intro = "What this org ships."
"#,
    )
    .unwrap();

    let site = load_project_site(&config).unwrap();
    assert_eq!(
        site.projects[0].description.as_deref(),
        Some("Useful & <fast>")
    );
    crate::render_static(&site, &crate::RenderOptions::new(&out)).unwrap();
    let html = std::fs::read_to_string(out.join("index.html")).unwrap();
    assert!(html.contains("id=\"ecosystem\""));
    assert!(html.contains("project-grid"));
    assert!(html.contains("project-card"));
    assert!(html.contains("Tool &lt;beta&gt;"));
    assert!(html.contains("Useful &amp; &lt;fast&gt;"));
    assert!(html.contains("href=\"https://tool.example/?q=&quot;x&quot;\""));
    assert!(html.contains("link-source"));
    assert!(!html.contains("Tool <beta>"));
}

#[test]
fn canonical_domain_parses_and_emits_cname() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("plinth-project.toml");
    let out = dir.path().join("public");
    std::fs::write(
        &config,
        r#"
[site]
title = "Example"
description = "Example site"
canonical_domain = "example.com"

[[pages]]
slug = "index"
title = "Example"
"#,
    )
    .unwrap();

    let site = load_project_site(&config).unwrap();
    assert_eq!(site.canonical_domain.as_deref(), Some("example.com"));
    crate::render_static(&site, &crate::RenderOptions::new(&out)).unwrap();
    let cname = std::fs::read_to_string(out.join("CNAME")).unwrap();
    assert_eq!(cname, "example.com\n");
}

#[test]
fn broker_css_resolves_against_config_dir_and_is_watched() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("plinth-project.toml");
    std::fs::write(
        &config,
        r##"
[site]
title = "Example"
description = "Example site"

[theme]
broker_css = "theme/theme.css"
accent = "#123456"

[[pages]]
slug = "index"
title = "Example"
"##,
    )
    .unwrap();

    let site = load_project_site(&config).unwrap();
    let css = dir.path().join("theme/theme.css");
    assert_eq!(site.theme.broker_css.as_deref(), Some(css.as_path()));
    // Configured colours stay available as fallbacks.
    assert_eq!(site.theme.accent.as_deref(), Some("#123456"));
    assert!(
        super::project_watch_paths(&config)
            .unwrap()
            .contains(&dir.path().join("theme"))
    );
}

#[test]
fn omitted_broker_css_keeps_existing_sites_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("plinth-project.toml");
    std::fs::write(
        &config,
        "[site]\ntitle = \"Example\"\ndescription = \"Example site\"\n[theme]\nfont_family = \"Inter\"\n",
    )
    .unwrap();
    assert!(
        load_project_site(&config)
            .unwrap()
            .theme
            .broker_css
            .is_none()
    );
}

#[test]
fn misspelled_broker_css_key_fails_fast() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("plinth-project.toml");
    std::fs::write(
        &config,
        "[site]\ntitle = \"Example\"\ndescription = \"Example site\"\n[theme]\nbroker_cs = \"x.css\"\n",
    )
    .unwrap();
    let error = match load_project_site(&config) {
        Ok(_) => panic!("typo in [theme] should fail"),
        Err(error) => error.to_string(),
    };
    assert!(error.contains("unknown field"));
    assert!(error.contains("broker_cs"));
}

#[cfg(all(feature = "brick-screenshot-grid", feature = "brick-workflow-steps"))]
#[test]
fn scroll_sequence_parses_on_screenshot_grid_and_workflow_steps() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("plinth-project.toml");
    std::fs::write(
        &config,
        r#"
[site]
title = "Example"
description = "Example site"

[[pages]]
slug = "index"
title = "Example"

[[pages.sections]]
type = "screenshot_grid"
id = "shots"
heading = "Shots"
intro = "Intro"
scroll_sequence = true

[[pages.sections]]
type = "workflow_steps"
heading = "Flow"
intro = "Intro"
scroll_sequence = true

[[pages.sections]]
type = "workflow_steps"
heading = "Static"
intro = "Intro"
"#,
    )
    .unwrap();

    let site = load_project_site(&config).unwrap();
    let sections = &site.pages[0].sections;
    assert!(matches!(&sections[0], crate::ProjectSection::ScreenshotGrid(g) if g.scroll_sequence));
    assert!(matches!(&sections[1], crate::ProjectSection::WorkflowSteps(w) if w.scroll_sequence));
    assert!(matches!(&sections[2], crate::ProjectSection::WorkflowSteps(w) if !w.scroll_sequence));
}

#[cfg(feature = "brick-screenshot-grid")]
#[test]
fn misspelled_scroll_sequence_key_fails_fast() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("plinth-project.toml");
    std::fs::write(
        &config,
        r#"
[site]
title = "Example"
description = "Example site"

[[pages]]
slug = "index"
title = "Example"

[[pages.sections]]
type = "screenshot_grid"
id = "shots"
heading = "Shots"
intro = "Intro"
scroll_sequnce = true
"#,
    )
    .unwrap();
    let error = match load_project_site(&config) {
        Ok(_) => panic!("typo in section should fail"),
        Err(error) => error.to_string(),
    };
    assert!(error.contains("scroll_sequnce"));
}

/// Load `toml`, render it, and return `index.html`.
fn render_index(toml: &str) -> String {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("plinth-project.toml");
    let out = dir.path().join("public");
    std::fs::write(&config, toml).unwrap();
    let site = load_project_site(&config).unwrap();
    crate::render_static(&site, &crate::RenderOptions::new(&out)).unwrap();
    std::fs::read_to_string(out.join("index.html")).unwrap()
}

const PAGE: &str = "\n[[pages]]\nslug = \"index\"\ntitle = \"Home\"\n";

#[test]
fn site_stylesheets_follow_builtin_stylesheet_in_order() {
    let html = render_index(&format!(
        "[site]\ntitle = \"E\"\ndescription = \"D\"\nstylesheets = [\"/site.css\", \"/a\\\"b.css\"]\n{PAGE}"
    ));
    assert!(html.contains(
        "<link rel=\"stylesheet\" href=\"/style.css\"><link rel=\"stylesheet\" href=\"/site.css\"><link rel=\"stylesheet\" href=\"/a&quot;b.css\"></head>"
    ));
}

#[test]
fn favicon_adds_svg_type_only_for_svg() {
    let head = |icon: &str| {
        render_index(&format!(
            "[site]\ntitle = \"E\"\ndescription = \"D\"\nfavicon = \"{icon}\"\n{PAGE}"
        ))
    };
    assert!(
        head("/icon.svg").contains("<link rel=\"icon\" href=\"/icon.svg\" type=\"image/svg+xml\">")
    );
    assert!(head("/icon.png").contains("<link rel=\"icon\" href=\"/icon.png\">"));
}

#[test]
fn og_image_emits_social_meta_with_absolute_urls_under_canonical_domain() {
    let html = render_index(&format!(
        "[site]\ntitle = \"E\"\ndescription = \"Site desc\"\ncanonical_domain = \"example.com\"\nog_image = \"/social.png\"\n{PAGE}"
    ));
    for meta in [
        "<meta property=\"og:title\" content=\"Home\">",
        "<meta property=\"og:description\" content=\"Site desc\">",
        "<meta property=\"og:type\" content=\"website\">",
        "<meta property=\"og:url\" content=\"https://example.com/\">",
        "<meta property=\"og:image\" content=\"https://example.com/social.png\">",
        "<meta name=\"twitter:card\" content=\"summary_large_image\">",
    ] {
        assert!(html.contains(meta), "missing {meta}");
    }

    // Without a canonical domain: no og:url, path kept verbatim.
    let html = render_index(&format!(
        "[site]\ntitle = \"E\"\ndescription = \"D\"\nog_image = \"/social.png\"\n{PAGE}"
    ));
    assert!(!html.contains("og:url"));
    assert!(html.contains("<meta property=\"og:image\" content=\"/social.png\">"));

    // Already-absolute images are never rewritten.
    let html = render_index(&format!(
        "[site]\ntitle = \"E\"\ndescription = \"D\"\ncanonical_domain = \"example.com\"\nog_image = \"https://cdn.example/s.png\"\n{PAGE}"
    ));
    assert!(html.contains("<meta property=\"og:image\" content=\"https://cdn.example/s.png\">"));
}

#[cfg(feature = "brick-hero")]
#[test]
fn hero_preview_still_precedes_mobile_source() {
    let html = render_index(
        r#"
[site]
title = "E"
description = "D"

[[pages]]
slug = "index"
title = "Home"
[[pages.sections]]
type = "hero"
title = "T"
tagline = "tag"
subtitle = ""
preview_src = "/p.webp"
preview_mobile_src = "/m.webp"
preview_still = "/still.webp"
"#,
    );
    assert!(html.contains(
        "<picture class=\"hero-media\"><source media=\"(prefers-reduced-motion: reduce)\" srcset=\"/still.webp\"><source media=\"(max-width: 700px)\" srcset=\"/m.webp\"><img class=\"hero-preview\""
    ));
}

#[test]
fn site_without_head_polish_fields_emits_no_extras() {
    let html = render_index(&format!(
        "[site]\ntitle = \"E\"\ndescription = \"D\"\n{PAGE}"
    ));
    assert!(!html.contains("og:"));
    assert!(!html.contains("twitter:"));
    assert!(!html.contains("rel=\"icon\""));
    assert_eq!(html.matches("rel=\"stylesheet\"").count(), 1);
}
