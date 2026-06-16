use crate::{Page, ProjectSite, ProjectTheme, RenderOptions, render_static};

#[cfg(feature = "brick-custom")]
use crate::CustomSection;

#[cfg(all(feature = "brick-hero", feature = "brick-person-mention"))]
use crate::{ExternalLink, Hero, LinkKind, PersonMention, PersonReference, ProjectSection};

#[cfg(feature = "brick-screenshot-grid")]
use crate::{Screenshot, ScreenshotGrid};

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
                screenshots: vec![Screenshot {
                    src: "/screenshots/main.png".into(),
                    alt: "Main app view".into(),
                    caption: "Main view".into(),
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
                id: Some("flow".into()),
                heading: "Workflow".into(),
                intro: "How the work moves.".into(),
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
