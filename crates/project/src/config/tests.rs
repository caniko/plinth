use super::{load_project_site, project_watch_paths};
#[cfg(feature = "brick-capability-matrix")]
use crate::ProjectSection;
use plinth_person::LinkKind;

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

    let paths = project_watch_paths(&config).unwrap();
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
