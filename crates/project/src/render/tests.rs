use crate::{Page, ProjectSite, ProjectTheme, RenderOptions, render_static};

#[test]
fn scalable_text_does_not_change_other_pages() {
    let mut site = ProjectSite::new("Example", "Example").page(Page::new("index", "Home"));
    let options = RenderOptions::new("unused");
    let html = super::render_page(&site, &site.pages[0], &options);
    let css = super::stylesheet(&site);
    let mut enabled = Page::new("readable", "Readable");
    enabled.scalable_text = true;
    site.pages.push(enabled);
    assert_eq!(super::render_page(&site, &site.pages[0], &options), html);
    assert!(super::stylesheet(&site).starts_with(&css));
    assert!(
        super::render_page(&site, &site.pages[1], &options).contains("class=\"scalable-text\"")
    );
}

#[cfg(feature = "brick-company")]
#[test]
fn scalable_text_does_not_change_company_output() {
    let config = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../website/company-examples/north.toml");
    let mut site = crate::load_project_site(config).unwrap();
    let options = RenderOptions::new("unused");
    let html = super::render_page(&site, &site.pages[0], &options);
    let css = super::stylesheet(&site);
    site.pages[0].scalable_text = true;
    assert_eq!(super::render_page(&site, &site.pages[0], &options), html);
    assert_eq!(super::stylesheet(&site), css);
}

#[cfg(unix)]
#[test]
fn output_cleanup_preserves_source_symlink_entries() {
    use std::os::unix::fs::symlink;
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source");
    let output = dir.path().join("output");
    std::fs::create_dir(&source).unwrap();
    std::fs::create_dir(&output).unwrap();
    let asset = source.join("logo.svg");
    std::fs::write(&asset, "source asset").unwrap();
    symlink(&asset, output.join("logo.svg")).unwrap();
    symlink(&source, output.join("assets")).unwrap();
    for input in [output.join("logo.svg"), output.join("assets/logo.svg")] {
        let site = ProjectSite::new("example", "example site")
            .asset(crate::Asset::new(&input, "/logo.svg"));
        assert!(render_static(&site, &RenderOptions::new(&output)).is_err());
        assert_eq!(std::fs::read_to_string(&input).unwrap(), "source asset");
    }
    let config = source.join("plinth-project.toml");
    std::fs::write(&config, "[site]\ntitle = 'Example'\ndescription = 'Example site'\n[[pages]]\nslug = 'index'\ntitle = 'Example'\n").unwrap();
    symlink(&config, output.join("config.toml")).unwrap();
    let site = crate::load_project_site(output.join("config.toml")).unwrap();
    assert!(render_static(&site, &RenderOptions::new(&output)).is_err());
    assert!(output.join("config.toml").is_symlink());
    assert!(asset.exists());
}

#[cfg(unix)]
#[test]
fn output_cleanup_preserves_links_reached_through_another_link_target() {
    use std::os::unix::fs::symlink;
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("output");
    std::fs::create_dir(&output).unwrap();
    let asset = dir.path().join("logo.svg");
    std::fs::write(&asset, "source asset").unwrap();
    symlink(&asset, output.join("logo.svg")).unwrap();
    let alias = dir.path().join("alias.svg");
    symlink(output.join("logo.svg"), &alias).unwrap();
    let site =
        ProjectSite::new("example", "example site").asset(crate::Asset::new(&alias, "/logo.svg"));
    assert!(render_static(&site, &RenderOptions::new(&output)).is_err());
    assert_eq!(std::fs::read_to_string(alias).unwrap(), "source asset");
}

#[test]
fn output_cleanup_preserves_inputs_and_resolves_symlinks() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source");
    std::fs::create_dir(&source).unwrap();
    let asset = source.join("logo.svg");
    std::fs::write(&asset, "source asset").unwrap();
    let site =
        ProjectSite::new("example", "example site").asset(crate::Asset::new(&asset, "/logo.svg"));
    assert!(render_static(&site, &RenderOptions::new(&source)).is_err());
    assert_eq!(std::fs::read_to_string(&asset).unwrap(), "source asset");
    #[cfg(unix)]
    {
        let alias = dir.path().join("alias");
        std::os::unix::fs::symlink(&source, &alias).unwrap();
        assert!(render_static(&site, &RenderOptions::new(&alias)).is_err());
        assert!(render_static(&site, &RenderOptions::new(alias.join("new/../.."))).is_err());
        assert!(asset.exists());
    }
    render_static(&site, &RenderOptions::new(source.join("public"))).unwrap();
    assert!(asset.exists());
}

#[test]
fn output_cleanup_preserves_the_broker_stylesheet() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("public");
    std::fs::create_dir(&output).unwrap();
    let broker = output.join("broker-theme.css");
    std::fs::write(&broker, "theme").unwrap();
    let mut site = ProjectSite::new("example", "example site");
    site.theme.broker_css = Some(broker.clone());
    assert!(render_static(&site, &RenderOptions::new(&output)).is_err());
    assert_eq!(std::fs::read_to_string(broker).unwrap(), "theme");
}

#[test]
fn loaded_config_is_protected_even_without_assets() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("plinth-project.toml");
    std::fs::write(&config, "[site]\ntitle = 'Example'\ndescription = 'Example site'\n[[pages]]\nslug = 'index'\ntitle = 'Example'\n").unwrap();
    let site = crate::load_project_site(&config).unwrap();
    assert!(render_static(&site, &RenderOptions::new(dir.path())).is_err());
    assert!(config.exists());
    render_static(&site, &RenderOptions::new(dir.path().join("public"))).unwrap();
}

#[cfg(feature = "brick-custom")]
use crate::CustomSection;

#[cfg(all(feature = "brick-hero", feature = "brick-person-mention"))]
use crate::{ExternalLink, Hero, LinkKind, PersonMention, PersonReference, ProjectSection};

#[cfg(feature = "brick-screenshot-grid")]
use crate::{Screenshot, ScreenshotGrid};

#[cfg(feature = "brick-feature-grid")]
use crate::{Feature, FeatureGrid};

#[cfg(all(
    feature = "brick-workflow-steps",
    feature = "brick-audience-grid",
    feature = "brick-trust-panel"
))]
use crate::{Audience, AudienceGrid, TrustItem, TrustPanel, WorkflowStep, WorkflowSteps};

#[cfg(feature = "brick-custom")]
#[test]
fn renders_custom_section() {
    let dir = tempfile::tempdir().unwrap();
    let site = ProjectSite::new("example", "example site").page(
        Page::new("index", "example").section(crate::ProjectSection::Custom(CustomSection::new(
            || "<section id=\"custom\">custom slot</section>".into(),
        ))),
    );

    render_static(&site, &RenderOptions::new(dir.path())).unwrap();
    let html = std::fs::read_to_string(dir.path().join("index.html")).unwrap();
    assert!(html.contains("custom slot"));
    assert!(html.contains("plinth-project"));
}

#[test]
fn dev_reload_script_is_opt_in() {
    let dir = tempfile::tempdir().unwrap();
    let site = ProjectSite::new("example", "example site").page(Page::new("index", "example"));

    render_static(&site, &RenderOptions::new(dir.path())).unwrap();
    let html = std::fs::read_to_string(dir.path().join("index.html")).unwrap();
    assert!(!html.contains("__plinth_project_reload"));

    render_static(
        &site,
        &RenderOptions::new(dir.path()).with_dev_reload("/__plinth_project_reload"),
    )
    .unwrap();
    let html = std::fs::read_to_string(dir.path().join("index.html")).unwrap();
    assert!(html.contains("__plinth_project_reload"));
}

#[test]
fn renders_theme_css_variables_when_configured() {
    let dir = tempfile::tempdir().unwrap();
    let site = ProjectSite {
        theme: ProjectTheme {
            paper: Some("#fbf8f4".into()),
            ink: Some("#2a2724".into()),
            accent: Some("#c9a0a6".into()),
            ..ProjectTheme::default()
        },
        ..ProjectSite::new("example", "example site")
    }
    .page(Page::new("index", "example"));

    render_static(&site, &RenderOptions::new(dir.path())).unwrap();
    let css = std::fs::read_to_string(dir.path().join("style.css")).unwrap();
    assert!(css.contains("--pp-paper:#fbf8f4"));
    assert!(css.contains("--pp-ink:#2a2724"));
    assert!(css.contains("--pp-accent:#c9a0a6"));
    assert!(css.contains("var(--pp-paper"));
}

#[cfg(feature = "brick-feature-grid")]
#[test]
fn feature_grid_cards_follow_page_heading() {
    let dir = tempfile::tempdir().unwrap();
    let site = ProjectSite::new("example", "example site").page(
        Page::new("index", "example").section(crate::ProjectSection::FeatureGrid(FeatureGrid {
            id: Some("overview".into()),
            features: vec![Feature::new("Rust", "A reliable toolchain.")],
            ..FeatureGrid::default()
        })),
    );

    render_static(&site, &RenderOptions::new(dir.path())).unwrap();
    let html = std::fs::read_to_string(dir.path().join("index.html")).unwrap();
    assert!(html.contains("<h2>Rust</h2>"));
    assert!(!html.contains("<h3>Rust</h3>"));
}

#[cfg(feature = "brick-screenshot-grid")]
#[test]
fn screenshot_grid_images_are_lightbox_triggers() {
    let dir = tempfile::tempdir().unwrap();
    let site =
        ProjectSite::new("example", "example site").page(Page::new("index", "example").section(
            crate::ProjectSection::ScreenshotGrid(ScreenshotGrid {
                id: "shots".into(),
                heading: "Screenshots".into(),
                intro: "Generated from the app.".into(),
                scroll_sequence: false,
                screenshots: vec![Screenshot {
                    src: "/screenshots/main.png".into(),
                    alt: "Main app view".into(),
                    caption: "Main view".into(),
                    still: None,
                }],
            }),
        ));

    render_static(&site, &RenderOptions::new(dir.path())).unwrap();
    let html = std::fs::read_to_string(dir.path().join("index.html")).unwrap();
    assert!(html.contains("data-lightbox-image=\"/screenshots/main.png\""));
    assert!(html.contains("data-lightbox-caption=\"Main view\""));
    assert!(html.contains("image-lightbox"));
    assert!(html.matches("class=\\\"image-lightbox\\\"").count() <= 1);
}

#[cfg(all(
    feature = "brick-hero",
    feature = "brick-person-mention",
    feature = "brick-screenshot-grid"
))]
#[test]
fn identity_images_do_not_become_lightbox_triggers() {
    let dir = tempfile::tempdir().unwrap();
    let person = PersonReference {
        id: "maintainer".into(),
        name: "Maintainer".into(),
        url: "https://person.example".into(),
        role: Some("Project lead".into()),
        avatar_url: Some("/avatar.png".into()),
        links: Vec::new(),
    };
    let site = ProjectSite::new("example", "example site").page(
        Page::new("index", "example")
            .section(ProjectSection::Hero(Hero {
                logo_src: Some("/logo.svg".into()),
                title: "Example".into(),
                tagline: "Built plainly".into(),
                subtitle: "A test site".into(),
                person: None,
                ctas: Vec::new(),
                ..Hero::default()
            }))
            .section(ProjectSection::PersonMention(PersonMention {
                id: Some("maintainer".into()),
                heading: "Maintainer".into(),
                intro: "Who keeps this project moving.".into(),
                person,
            })),
    );

    render_static(&site, &RenderOptions::new(dir.path())).unwrap();
    let html = std::fs::read_to_string(dir.path().join("index.html")).unwrap();
    assert!(html.contains("class=\"hero-logo\""));
    assert!(html.contains("class=\"person-avatar\""));
    assert!(!html.contains("data-lightbox-image=\"/logo.svg\""));
    assert!(!html.contains("data-lightbox-image=\"/avatar.png\""));
}

#[cfg(all(
    feature = "brick-workflow-steps",
    feature = "brick-audience-grid",
    feature = "brick-trust-panel"
))]
#[test]
fn renders_product_brick_markers() {
    let dir = tempfile::tempdir().unwrap();
    let site = ProjectSite::new("example", "example site").page(
        Page::new("index", "example")
            .section(crate::ProjectSection::WorkflowSteps(WorkflowSteps {
                eyebrow: None,
                id: Some("flow".into()),
                heading: "Workflow".into(),
                intro: "How the work moves.".into(),
                scroll_sequence: false,
                steps: vec![WorkflowStep {
                    title: "Discover".into(),
                    description: "Find relevant precedent records.".into(),
                }],
            }))
            .section(crate::ProjectSection::AudienceGrid(AudienceGrid {
                id: Some("roles".into()),
                heading: "Roles".into(),
                intro: "Who uses it.".into(),
                audiences: vec![Audience {
                    label: "Curator".into(),
                    description: "Reviews and organizes records.".into(),
                }],
            }))
            .section(crate::ProjectSection::TrustPanel(TrustPanel {
                eyebrow: None,
                id: Some("trust".into()),
                heading: "Trust".into(),
                intro: "How safety stays visible.".into(),
                items: vec![TrustItem {
                    title: "Rights visible".into(),
                    description: "Rights travel with each record.".into(),
                }],
            })),
    );

    render_static(&site, &RenderOptions::new(dir.path())).unwrap();
    let html = std::fs::read_to_string(dir.path().join("index.html")).unwrap();
    assert!(html.contains("workflow-steps"));
    assert!(html.contains("audience-grid"));
    assert!(html.contains("trust-panel"));
}

#[cfg(all(feature = "brick-hero", feature = "brick-person-mention"))]
#[test]
fn renders_primary_person_links_and_metadata() {
    let dir = tempfile::tempdir().unwrap();
    let person = PersonReference {
        id: "maintainer".into(),
        name: "Maintainer".into(),
        url: "https://person.example".into(),
        role: Some("Project lead".into()),
        avatar_url: None,
        links: vec![ExternalLink::new(
            "Contact",
            "https://person.example/contact",
            LinkKind::Contact,
        )],
    };
    let site = ProjectSite {
        primary_person: Some("maintainer".into()),
        people: vec![person.clone()],
        ..ProjectSite::new("example", "example site")
    }
    .page(
        Page::new("index", "example")
            .section(ProjectSection::Hero(Hero {
                logo_src: None,
                title: "Example".into(),
                tagline: "Built plainly".into(),
                subtitle: "A test site".into(),
                person: Some("maintainer".into()),
                ctas: Vec::new(),
                ..Hero::default()
            }))
            .section(ProjectSection::PersonMention(PersonMention {
                id: Some("maintainer".into()),
                heading: "Maintainer".into(),
                intro: "Who keeps this project moving.".into(),
                person,
            })),
    );

    render_static(&site, &RenderOptions::new(dir.path())).unwrap();
    let html = std::fs::read_to_string(dir.path().join("index.html")).unwrap();
    assert!(html.contains("hero-byline"));
    assert!(html.contains("Maintained by"));
    assert!(html.contains("application/ld+json"));
    assert!(html.contains("person-mention"));
    assert!(html.contains("link-contact"));
}

#[test]
fn renders_machine_artifacts() {
    use crate::ProjectReference;

    let dir = tempfile::tempdir().unwrap();
    let mut site = ProjectSite::new("Example", "An example site.");
    site.canonical_domain = Some("example.com".into());
    site.projects = vec![ProjectReference {
        title: "Demo".into(),
        url: "https://demo.example/".into(),
        description: Some("A demo project.".into()),
        source_url: Some("https://github.com/example/demo".into()),
        demo_url: None,
        links: vec![crate::ExternalLink::new(
            "Docs",
            "https://demo.example/docs",
            crate::LinkKind::Docs,
        )],
    }];
    let site = site
        .page(Page::new("index", "Home").description("Welcome."))
        .page(Page::new("about", "About"));

    render_static(&site, &RenderOptions::new(dir.path())).unwrap();

    let sitemap = std::fs::read_to_string(dir.path().join("sitemap.xml")).unwrap();
    assert!(sitemap.contains("<loc>https://example.com/</loc>"));
    assert!(sitemap.contains("<loc>https://example.com/about/</loc>"));

    let robots = std::fs::read_to_string(dir.path().join("robots.txt")).unwrap();
    assert!(robots.contains("Allow: /"));
    assert!(robots.contains("Sitemap: https://example.com/sitemap.xml"));

    let llms = std::fs::read_to_string(dir.path().join("llms.txt")).unwrap();
    assert!(llms.contains("# Example"));
    assert!(llms.contains("[Home](https://example.com/)"));
    assert!(llms.contains("[Demo](https://demo.example/)"));
    assert!(llms.contains("Source: https://github.com/example/demo"));

    let projects = std::fs::read_to_string(dir.path().join("projects.json")).unwrap();
    assert!(projects.contains("\"title\": \"Demo\""));

    let cname = std::fs::read_to_string(dir.path().join("CNAME")).unwrap();
    assert_eq!(cname, "example.com\n");
}

#[test]
fn canonical_link_follows_canonical_domain() {
    let page = |site: ProjectSite| {
        let dir = tempfile::tempdir().unwrap();
        let site = site
            .page(Page::new("index", "Home"))
            .page(Page::new("about", "About"));
        render_static(&site, &RenderOptions::new(dir.path())).unwrap();
        let read = |f: &str| std::fs::read_to_string(dir.path().join(f)).unwrap();
        (read("index.html"), read("about/index.html"))
    };
    let mut with = ProjectSite::new("Example", "An example site.");
    with.canonical_domain = Some("example.com".into());
    let (index, about) = page(with);
    assert!(index.contains("<link rel=\"canonical\" href=\"https://example.com/\">"));
    assert!(about.contains("<link rel=\"canonical\" href=\"https://example.com/about/\">"));

    let (index, about) = page(ProjectSite::new("Example", "An example site."));
    assert!(!index.contains("rel=\"canonical\""));
    assert!(!about.contains("rel=\"canonical\""));
}

#[test]
fn omits_cname_without_canonical_domain() {
    let dir = tempfile::tempdir().unwrap();
    let site = ProjectSite::new("Example", "An example site.").page(Page::new("index", "Home"));

    render_static(&site, &RenderOptions::new(dir.path())).unwrap();

    assert!(!dir.path().join("CNAME").exists());
    let sitemap = std::fs::read_to_string(dir.path().join("sitemap.xml")).unwrap();
    assert!(sitemap.contains("<loc>/</loc>"));
}

fn broker_site(dir: &std::path::Path) -> ProjectSite {
    let css = dir.join("src-theme.css");
    std::fs::write(&css, ":root{--tb-ui-background:#fff}").unwrap();
    ProjectSite {
        theme: ProjectTheme {
            accent: Some("#c9a0a6".into()),
            broker_css: Some(css),
            ..ProjectTheme::default()
        },
        ..ProjectSite::new("example", "example site")
    }
    .page(Page::new("index", "example"))
}

#[test]
fn broker_css_is_copied_linked_before_style_and_aliased() {
    let src = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();
    render_static(&broker_site(src.path()), &RenderOptions::new(out.path())).unwrap();

    assert_eq!(
        std::fs::read_to_string(out.path().join("broker-theme.css")).unwrap(),
        ":root{--tb-ui-background:#fff}"
    );
    let html = std::fs::read_to_string(out.path().join("index.html")).unwrap();
    let broker = html.find("href=\"/broker-theme.css\"").unwrap();
    let style = html.find("href=\"/style.css\"").unwrap();
    assert!(broker < style);
    assert!(html.contains("<body class=\"pp-minimal\">"));

    let css = std::fs::read_to_string(out.path().join("style.css")).unwrap();
    assert!(css.contains("--pp-paper:var(--tb-ui-background);"));
    assert!(css.contains("--pp-ink-soft:var(--tb-ui-foreground-muted);"));
    // Configured colour is the fallback for its alias.
    assert!(css.contains("--pp-accent:var(--tb-ui-accent,#c9a0a6);"));
    assert!(css.contains(
        "--pp-accent-soft:var(--tb-ui-accent-muted,color-mix(in srgb,var(--pp-accent) 16%,transparent));"
    ));
}

#[test]
fn broker_theme_aliases_cover_ui_state_tokens_generically() {
    let src = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();
    render_static(&broker_site(src.path()), &RenderOptions::new(out.path())).unwrap();
    let css = std::fs::read_to_string(out.path().join("style.css")).unwrap();
    let start = css.find(":root{").unwrap();
    let theme = &css[start..start + css[start..].find('}').unwrap() + 1];

    for alias in [
        "--pp-accent:var(--tb-ui-accent,#c9a0a6);",
        "--pp-accent-hover:var(--tb-ui-accent-hover);",
        "--pp-accent-active:var(--tb-ui-accent-active,var(--pp-accent-hover));",
        "--pp-accent-soft:var(--tb-ui-accent-muted,color-mix(in srgb,var(--pp-accent) 16%,transparent));",
        "--pp-on-accent:var(--tb-ui-on-accent,var(--pp-paper));",
        "--pp-focus:var(--tb-ui-focus,var(--pp-accent));",
        "--pp-link:var(--tb-ui-link);",
        "--pp-selection-background:var(--tb-ui-selection-background,var(--pp-accent));",
        "--pp-selection-foreground:var(--tb-ui-selection-foreground,var(--pp-paper));",
        "--pp-panel:var(--tb-ui-background-alt,var(--pp-paper));",
        "--pp-surface:var(--tb-ui-surface);",
        "--pp-surface-alt:var(--tb-ui-surface-alt,var(--pp-surface));",
        "--pp-ink-soft:var(--tb-ui-foreground-muted);",
        "--pp-ink-subtle:var(--tb-ui-foreground-subtle,var(--pp-ink-soft));",
        "--pp-ink-bright:var(--tb-ui-foreground-bright);",
        "--pp-line-subtle:var(--tb-ui-border-subtle,var(--pp-line));",
    ] {
        assert!(theme.contains(alias), "{alias}");
    }

    // Generic by construction: the configured accent is the only literal, so
    // no palette-specific colour can leak into the aliases.
    assert_eq!(theme.matches('#').count(), 1, "{theme}");
}

#[test]
fn older_broker_state_tokens_fall_back_to_the_previous_look() {
    let src = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();
    render_static(&broker_site(src.path()), &RenderOptions::new(out.path())).unwrap();
    let css = std::fs::read_to_string(out.path().join("style.css")).unwrap();

    // Bright ink is optional in the light/older sheets. Keep the fallback at
    // each use site: ordinary links, toggle hover and selection differ.
    assert!(css.contains("--pp-ink-bright:var(--tb-ui-foreground-bright);"));
    assert!(css.contains("color: var(--pp-ink-bright, var(--pp-ink-soft));"));
    assert!(css.contains("color: var(--pp-ink-bright, var(--pp-accent));"));

    // A broker stylesheet that only carries the older basic roles still
    // resolves every state through the old semantic colours.
    for expected in [
        "--pp-accent-hover:var(--tb-ui-accent-hover);",
        "--pp-accent-active:var(--tb-ui-accent-active,var(--pp-accent-hover));",
        "--pp-on-accent:var(--tb-ui-on-accent,var(--pp-paper));",
        "--pp-focus:var(--tb-ui-focus,var(--pp-accent));",
        "--pp-link:var(--tb-ui-link);",
    ] {
        assert!(css.contains(expected), "{expected}");
    }
    assert!(
        css.contains(
            "--pp-selection-background:var(--tb-ui-selection-background,var(--pp-accent));"
        )
    );
    assert!(
        css.contains(
            "--pp-selection-foreground:var(--tb-ui-selection-foreground,var(--pp-paper));"
        )
    );
    assert!(css.contains(
        "--pp-accent-soft:var(--tb-ui-accent-muted,color-mix(in srgb,var(--pp-accent) 16%,transparent));"
    ));
}

#[test]
fn non_broker_sites_keep_the_legacy_theme_and_skip_state_aliases() {
    let dir = tempfile::tempdir().unwrap();
    let site = ProjectSite {
        theme: ProjectTheme {
            paper: Some("#fbf8f4".into()),
            accent: Some("#c9a0a6".into()),
            ..ProjectTheme::default()
        },
        ..ProjectSite::new("example", "example site")
    }
    .page(Page::new("index", "example"));
    render_static(&site, &RenderOptions::new(dir.path())).unwrap();
    let css = std::fs::read_to_string(dir.path().join("style.css")).unwrap();

    assert!(!css.contains("var(--tb-"));
    assert!(!css.contains("--pp-accent-hover:"));
    assert!(!css.contains("--pp-selection-background:"));
    assert!(css.contains("--pp-paper:#fbf8f4"));
    assert!(css.contains("--pp-accent:#c9a0a6"));
    // Use sites keep the pre-broker values when no broker role exists.
    assert!(css.contains("color: var(--pp-link, var(--pp-accent, #58a6ff));"));
    assert!(css.contains("background: var(--pp-accent, #58a6ff);"));
}

#[test]
fn minimal_ui_states_use_semantic_state_tokens() {
    let src = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();
    render_static(&broker_site(src.path()), &RenderOptions::new(out.path())).unwrap();
    let css = std::fs::read_to_string(out.path().join("style.css")).unwrap();
    let minimal = &css[css.find("@layer minimal {").unwrap()..];

    for rule in [
        ".pp-minimal::selection,\n  .pp-minimal ::selection {",
        "background: var(--pp-accent-hover, var(--pp-accent));",
        "background: var(--pp-accent-active, var(--pp-accent-hover, var(--pp-accent)));",
        "outline: 2px solid var(--pp-focus, var(--pp-accent));",
        "background: var(--pp-selection-background, var(--pp-accent));",
        "color: var(--pp-selection-foreground, var(--pp-paper));",
        "color: var(--pp-on-accent, var(--pp-paper));",
        "background: var(--pp-panel, transparent);",
        "background: var(--pp-surface-alt, var(--pp-surface));",
        "color: var(--pp-ink-subtle, var(--pp-ink-soft));",
        "color: var(--pp-ink-bright, var(--pp-ink-soft));",
        "color: var(--pp-link, currentColor);",
        "color: var(--pp-accent-hover, var(--pp-ink-soft));",
        ":not(:disabled):not([aria-disabled=\"true\"]):hover",
        ":not(:disabled):not([aria-disabled=\"true\"]):active",
        "--tartan-focus: var(--pp-focus, var(--pp-accent));",
    ] {
        assert!(minimal.contains(rule), "{rule}");
    }
}

#[test]
fn theme_toggle_is_accessible_and_only_with_broker_css() {
    let src = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();
    render_static(&broker_site(src.path()), &RenderOptions::new(out.path())).unwrap();
    let html = std::fs::read_to_string(out.path().join("index.html")).unwrap();
    assert!(html.contains(
        "<div class=\"theme-toggle\" role=\"group\" aria-label=\"Theme\" data-theme-toggle hidden>"
    ));
    for mode in ["system", "light", "dark"] {
        assert!(html.contains(&format!(
            "data-theme-mode=\"{mode}\" aria-label=\"{} theme\"",
            mode[..1].to_uppercase() + &mode[1..]
        )));
    }
    assert_eq!(
        html.matches("<svg viewBox=\"0 0 24 24\" width=\"16\" height=\"16\"")
            .count(),
        3
    );
    assert_eq!(html.matches("stroke-width=\"1.5\"").count(), 3);
    assert!(html.contains("aria-pressed"));
    assert!(html.contains("localStorage.getItem('tartan-theme')"));
    assert!(html.contains("localStorage.setItem('tartan-theme', mode)"));
    assert!(html.contains("dataset.theme"));

    let plain = tempfile::tempdir().unwrap();
    let site = ProjectSite::new("example", "example site").page(Page::new("index", "example"));
    render_static(&site, &RenderOptions::new(plain.path())).unwrap();
    let html = std::fs::read_to_string(plain.path().join("index.html")).unwrap();
    assert!(!html.contains("broker-theme.css"));
    assert!(!html.contains("theme-toggle"));
    assert!(!html.contains("tartan-theme"));
    assert!(!html.contains("pp-minimal"));
    assert!(!plain.path().join("broker-theme.css").exists());
    let css = std::fs::read_to_string(plain.path().join("style.css")).unwrap();
    assert!(!css.contains("var(--tb-"));
}

#[test]
fn missing_broker_css_names_the_source_path() {
    let out = tempfile::tempdir().unwrap();
    let site = ProjectSite {
        theme: ProjectTheme {
            broker_css: Some("/nonexistent/theme.css".into()),
            ..ProjectTheme::default()
        },
        ..ProjectSite::new("example", "example site")
    }
    .page(Page::new("index", "example"));
    let error = render_static(&site, &RenderOptions::new(out.path()))
        .unwrap_err()
        .to_string();
    assert!(error.contains("/nonexistent/theme.css"));
}

#[test]
fn stylesheet_carries_display_size_minimal_rules_and_scroll_sequence() {
    let src = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();
    render_static(&broker_site(src.path()), &RenderOptions::new(out.path())).unwrap();
    let css = std::fs::read_to_string(out.path().join("style.css")).unwrap();
    assert!(css.contains("--pp-display: 40px;"));
    // Parity rules live only in the minimal layer, so sites without
    // `broker_css` (no `pp-minimal` body class) are unaffected.
    let minimal = &css[css.find("@layer minimal {").unwrap()..];
    for rule in [
        ".pp-minimal .nav-brand::before { content: none; }",
        ".pp-minimal .screenshot-slot {",
        "counter(pp-step, decimal-leading-zero)",
        "--pc-width: 1200px;",
        "max-height: min(60vh, 33rem);",
        "grid-template-columns: minmax(0, 1fr) fit-content(55%);",
    ] {
        assert!(minimal.contains(rule), "{rule}");
        assert!(
            !css[..css.find("@layer minimal {").unwrap()].contains(rule),
            "{rule}"
        );
    }
    assert!(css.contains("@media (max-width: 480px) {\n    .pp-minimal { --pp-display: 32px; }"));
    assert!(css.contains("font-size: var(--pp-display);"));
    assert!(css.contains("@supports (animation-timeline: view())"));
    assert!(css.contains("animation-range: entry 0% cover 40%;"));
    assert!(css.contains("@media (prefers-reduced-motion: reduce)"));
    assert!(css.contains("@keyframes pp-sequence-in"));
    assert!(css.contains("translateY(16px)"));
}

#[cfg(all(feature = "brick-screenshot-grid", feature = "brick-workflow-steps"))]
#[test]
fn scroll_sequence_hook_is_rendered_only_when_enabled() {
    let dir = tempfile::tempdir().unwrap();
    let section = |on: bool| {
        (
            crate::ProjectSection::ScreenshotGrid(ScreenshotGrid {
                id: "shots".into(),
                heading: "Shots".into(),
                intro: "Intro".into(),
                scroll_sequence: on,
                screenshots: Vec::new(),
            }),
            crate::ProjectSection::WorkflowSteps(crate::WorkflowSteps {
                eyebrow: None,
                id: Some("flow".into()),
                heading: "Flow".into(),
                intro: "Intro".into(),
                scroll_sequence: on,
                steps: Vec::new(),
            }),
        )
    };
    for (on, expected) in [(true, 2), (false, 0)] {
        let (grid, flow) = section(on);
        let site = ProjectSite::new("example", "example site")
            .page(Page::new("index", "example").section(grid).section(flow));
        render_static(&site, &RenderOptions::new(dir.path())).unwrap();
        let html = std::fs::read_to_string(dir.path().join("index.html")).unwrap();
        assert_eq!(html.matches("data-scroll-sequence").count(), expected);
    }
}

#[test]
fn lightbox_trigger_cancels_anchor_navigation() {
    let script = super::interaction_script();
    let trigger = script
        .find("closest('[data-lightbox-image]')")
        .expect("lightbox click handler finds the trigger");
    let cancel = script
        .find("event.preventDefault()")
        .expect("lightbox handler cancels the trigger's navigation");
    let open = script
        .find("dialog.classList.add('open')")
        .expect("lightbox handler opens the dialog");
    assert!(
        trigger < cancel,
        "navigation is cancelled once the trigger is found"
    );
    assert!(
        cancel < open,
        "navigation is cancelled before the dialog opens"
    );
}

#[test]
fn lightbox_ignores_modified_anchor_clicks() {
    let script = super::interaction_script()
        .trim()
        .trim_start_matches("<script>")
        .trim_end_matches("</script>");
    let mut harness = String::new();
    harness.push_str(
        r#"
const document = {
  created: null,
  listeners: [],
  addEventListener(type, fn) { if (type === 'click') this.listeners.push(fn); },
  createElement() {
    const el = {
      className: '',
      innerHTML: '',
      attributes: {},
      classList: {
        values: [],
        add(name) { this.values.push(name); },
        remove(name) { this.values = this.values.filter((value) => value !== name); },
        contains(name) { return this.values.includes(name); },
      },
      setAttribute(name, value) { this.attributes[name] = value; },
      removeAttribute(name) { delete this.attributes[name]; },
      appendChild() {},
      addEventListener() {},
      querySelector(selector) {
        if (selector === 'img') return { src: '', alt: '', removeAttribute() {} };
        if (selector === 'p') return { textContent: '' };
        return { focus() {}, addEventListener() {} };
      },
    };
    this.created = el;
    return el;
  },
  body: { appendChild() {}, classList: { add() {}, remove() {} } },
};
const window = { matchMedia: () => ({ matches: false }) };
"#,
    );
    harness.push_str(script);
    harness.push_str(
        r#"
let prevented = 0;
function trigger(isAnchor) {
  return {
    dataset: { lightboxImage: '/a.webp', lightboxAlt: 'a', lightboxCaption: 'c' },
    closest(selector) {
      return selector === 'a[href]' && isAnchor ? {} : null;
    },
  };
}
function click(element, modifiers) {
  prevented = 0;
  document.created.classList.remove('open');
  const event = Object.assign({
    target: {
      closest(selector) {
        return selector === '[data-lightbox-image]' ? element : null;
      },
    },
    preventDefault() { prevented += 1; },
    ctrlKey: false,
    metaKey: false,
    shiftKey: false,
    altKey: false,
    button: 0,
  }, modifiers || {});
  document.listeners.forEach((listener) => listener(event));
  return { prevented, open: document.created.classList.contains('open') };
}
function expect(actual, expected, label) {
  if (actual !== expected) {
    console.error(`FAIL ${label}: ${actual} !== ${expected}`);
    process.exit(1);
  }
}
const anchor = trigger(true);
const button = trigger(false);
for (const modifier of ['ctrlKey', 'metaKey', 'shiftKey', 'altKey']) {
  expect(click(anchor, { [modifier]: true }).prevented, 0, `${modifier} leaves the navigation alone`);
  expect(click(anchor, { [modifier]: true }).open, false, `${modifier} does not open the lightbox`);
}
expect(click(anchor, { button: 1 }).prevented, 0, 'non-primary button leaves the navigation alone');
expect(click(anchor, { button: 1 }).open, false, 'non-primary button does not open the lightbox');
expect(click(anchor).prevented, 1, 'plain anchor click cancels navigation');
expect(click(anchor).open, true, 'plain anchor click opens the lightbox');
expect(click(button).open, true, 'button triggers still open');
console.log('lightbox modifier checks passed');
"#,
    );
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("lightbox-check.mjs");
    std::fs::write(&path, harness).unwrap();
    let output = std::process::Command::new("node")
        .arg(&path)
        .output()
        .expect("node runs the lightbox harness (the dev shell provides it)");
    assert!(
        output.status.success(),
        "node lightbox harness failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
