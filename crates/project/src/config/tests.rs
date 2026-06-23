use super::{load_project_site, project_watch_paths};
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
