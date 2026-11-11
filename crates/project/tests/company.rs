#![cfg(feature = "brick-company")]

use plinth_project::{RenderOptions, load_project_site, render_static, validate_site};

const COMPANY: &str = r##"
[site]
title = "North & Co"
description = "An independent studio."
[company]
location = "Copenhagen · Working everywhere"
[company.motion]
motif = "contour"
duration_secs = 32
intensity = 0.3
[[members]]
id = "maya"
name = "Maya Holm"
role = "Founder"
bio = "Builds things that last."
[[pages]]
slug = "index"
title = "North & Co"
[[pages.sections]]
type = "mission"
id = "mission"
heading = "Good work, with good people."
statement = "We turn complex ideas into useful experiences."
[[pages.sections]]
type = "team"
id = "team"
heading = "Small team. Shared ambition."
members = ["maya"]
[[pages.sections]]
type = "testimonials"
id = "testimonials"
heading = "In good company."
[[pages.sections.quotes]]
quote = "They understood the problem before proposing the solution."
name = "Alex Reed"
organization = "Example client"
[[pages.sections]]
type = "partnerships"
id = "partners"
heading = "Better, together."
[[pages.sections.partners]]
name = "Example partner"
relationship = "partner"
"##;

fn load(raw: &str) -> Result<plinth_project::ProjectSite, String> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("plinth-company.toml");
    std::fs::write(&path, raw).unwrap();
    load_project_site(path).map_err(|error| error.to_string())
}

#[cfg(feature = "brick-content")]
#[test]
fn composed_sections_and_member_cards_share_an_id_namespace() {
    for id in [
        "mission",
        "team-member-maya",
        "main-content",
        "company-navigation",
    ] {
        let site = load(&format!("{COMPANY}\n[[pages.sections]]\ntype = 'content'\nid = '{id}'\nhtml = '<p>Example prose</p>'\n")).unwrap();
        let report = validate_site(&site);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|d| d.code == "company.section.id"),
            "missing collision diagnostic for {id}"
        );
    }
}

#[cfg(feature = "brick-hero")]
#[test]
fn unsupported_company_hero_fields_have_explicit_diagnostics() {
    let site = load(&format!("{COMPANY}\n[[pages.sections]]\ntype = 'hero'\ntitle = 'Example'\ntagline = 'Example'\nsubtitle = ''\npreview_src = '/preview.svg'\n")).unwrap();
    assert!(
        validate_site(&site)
            .diagnostics
            .iter()
            .any(|d| d.code == "company.hero.unsupported")
    );
}

#[test]
fn company_content_renders_without_member_urls_or_portraits() {
    let site = load(COMPANY).unwrap();
    let out = tempfile::tempdir().unwrap();
    render_static(&site, &RenderOptions::new(out.path())).unwrap();
    let html = std::fs::read_to_string(out.path().join("index.html")).unwrap();
    for copy in [
        "Maya Holm",
        "Builds things that last.",
        "Example client",
        "Example partner",
    ] {
        assert!(html.contains(copy), "missing company content: {copy}");
    }
    assert!(html.contains("North &amp; Co"));
    assert!(html.contains("<h1 class=\"company-page-title\">North &amp; Co</h1>"));
    assert!(html.contains("application/ld+json"));
    assert!(html.contains("Organization"));
    assert!(out.path().join("company.js").exists());
}

#[cfg(feature = "brick-screenshot-grid")]
#[test]
fn company_pages_keep_shared_gallery_interactions() {
    let raw = format!(
        r##"{COMPANY}
[[pages.sections]]
type = "screenshot_grid"
id = "work"
heading = "Recent work"
intro = "Open an image for a closer look."
[[pages.sections.screenshots]]
src = "/assets/work.svg"
alt = "Example work"
caption = "Our work"
"##
    );
    let site = load(&raw).unwrap();
    let out = tempfile::tempdir().unwrap();
    render_static(&site, &RenderOptions::new(out.path())).unwrap();
    let html = std::fs::read_to_string(out.path().join("index.html")).unwrap();
    assert!(html.contains("data-lightbox-image=\"/assets/work.svg\""));
    assert!(html.contains("Close enlarged image"));
    assert!(html.contains("lastTrigger.focus()"));
}

#[test]
fn missing_team_reference_is_a_validation_error() {
    let site = load(&COMPANY.replace("members = [\"maya\"]", "members = [\"missing\"]")).unwrap();
    assert!(validate_site(&site).has_errors());
}

#[test]
fn compact_team_renders_a_strip_without_bios_and_links_to_the_full_team() {
    let compact = COMPANY
        .replace(
            "bio = \"Builds things that last.\"",
            "bio = \"Builds things that last.\"\nlinks = [{ label = \"Profile\", href = \"https://example.com/maya\" }]",
        )
        .replace(
            "members = [\"maya\"]",
            "members = [\"maya\"]\nlayout = \"compact\"\n[pages.sections.link]\nlabel = \"Meet everyone →\"\nhref = \"/people/\"",
        );
    let site = load(&compact).unwrap();
    assert!(!validate_site(&site).has_errors());
    let out = tempfile::tempdir().unwrap();
    render_static(&site, &RenderOptions::new(out.path())).unwrap();
    let html = std::fs::read_to_string(out.path().join("index.html")).unwrap();
    assert!(html.contains("company-section company-team company-team-compact company-team-small"));
    assert!(html.contains("<div class=\"company-team-strip\">"));
    assert!(html.contains("<article class=\"company-member-compact\" id=\"team-member-maya\">"));
    assert!(html.contains("Maya Holm"));
    assert!(html.contains("Founder"));
    assert!(!html.contains("Builds things that last."));
    assert!(!html.contains("example.com/maya"));
    assert!(!html.contains("company-member-bio"));
    assert!(html.contains(
        "<a class=\"company-action company-team-link\" href=\"/people/\">Meet everyone →</a>"
    ));

    for size in ["medium", "large"] {
        let sized = load(&compact.replace(
            "layout = \"compact\"",
            &format!("layout = \"compact\"\nsize = \"{size}\""),
        ))
        .unwrap();
        assert!(!validate_site(&sized).has_errors());
        let out = tempfile::tempdir().unwrap();
        render_static(&sized, &RenderOptions::new(out.path())).unwrap();
        let html = std::fs::read_to_string(out.path().join("index.html")).unwrap();
        assert!(html.contains(&format!(
            "company-section company-team company-team-compact company-team-{size}"
        )));
    }

    let sized = load(&COMPANY.replace(
        "members = [\"maya\"]",
        "members = [\"maya\"]\nsize = \"large\"",
    ))
    .unwrap();
    let report = validate_site(&sized);
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| d.code == "company.team.size")
    );

    let full = tempfile::tempdir().unwrap();
    let plain = load(COMPANY).unwrap();
    render_static(&plain, &RenderOptions::new(full.path())).unwrap();
    let html = std::fs::read_to_string(full.path().join("index.html")).unwrap();
    assert!(html.contains("<div class=\"company-team-grid\">"));
    assert!(html.contains("Builds things that last."));
    assert!(!html.contains("company-team-link"));

    let bad = COMPANY.replace(
        "members = [\"maya\"]",
        "members = [\"maya\"]\n[pages.sections.link]\nlabel = \"Team\"\nhref = \"javascript:x\"",
    );
    let report = validate_site(&load(&bad).unwrap());
    assert!(report.diagnostics.iter().any(|d| d.code == "company.link"));
}

#[test]
fn invalid_motion_settings_cannot_render() {
    for raw in [
        COMPANY.replace("duration_secs = 32", "duration_secs = 0"),
        COMPANY.replace("intensity = 0.3", "intensity = 2.0"),
    ] {
        let site = load(&raw).unwrap();
        let out = tempfile::tempdir().unwrap();
        assert!(render_static(&site, &RenderOptions::new(out.path())).is_err());
    }
}

#[test]
fn unknown_company_fields_and_motifs_are_rejected() {
    assert!(load(&COMPANY.replace("motif = \"contour\"", "motif = \"unknown\"")).is_err());
    assert!(load(&COMPANY.replace("location =", "locaton =")).is_err());
}

#[test]
fn company_rejects_duplicate_routes_and_unsafe_output_paths() {
    let duplicate = format!("{COMPANY}\n[[pages]]\nslug = \"index\"\ntitle = \"Duplicate\"\n");
    assert!(validate_site(&load(&duplicate).unwrap()).has_errors());
    assert!(
        validate_site(
            &load(&COMPANY.replace("slug = \"index\"", "slug = \"../outside\"")).unwrap()
        )
        .has_errors()
    );
}

#[test]
fn starter_preserves_user_text_and_hides_empty_modules() {
    let title = "__DESCRIPTION__ & \"Company\"";
    let description = "__TITLE__\nA thoughtful company.";
    let site = load(&plinth_project::company::starter_config(title, description)).unwrap();
    assert_eq!(site.title, title);
    assert_eq!(site.description, description);
    let out = tempfile::tempdir().unwrap();
    render_static(&site, &RenderOptions::new(out.path())).unwrap();
    let html = std::fs::read_to_string(out.path().join("index.html")).unwrap();
    assert!(!html.contains("href=\"#partners\""));
    assert!(!html.contains("id=\"partners\""));
}

#[test]
fn empty_module_on_another_page_does_not_hide_a_populated_anchor() {
    let raw = format!(
        "{COMPANY}\n[[nav]]\nlabel = \"Local partners\"\nhref = \"#partners\"\n[[nav]]\nlabel = \"Home partners\"\nhref = \"/#partners\"\n[[nav]]\nlabel = \"Other partners\"\nhref = \"/other/#partners\"\n[[footer_links]]\nlabel = \"Other partners\"\nhref = \"/other/#partners\"\n[[pages]]\nslug = \"other\"\ntitle = \"Other\"\n[[pages.sections]]\ntype = \"partnerships\"\nid = \"partners\"\nheading = \"Partners\"\npartners = []\n"
    );
    let site = load(&raw).unwrap();
    let out = tempfile::tempdir().unwrap();
    render_static(&site, &RenderOptions::new(out.path())).unwrap();
    let html = std::fs::read_to_string(out.path().join("index.html")).unwrap();
    assert!(html.contains("href=\"#partners\""));
    assert!(html.contains("id=\"partners\""));
    assert!(html.contains("href=\"/#partners\""));
    assert!(!html.contains("href=\"/other/#partners\""));
    let other = std::fs::read_to_string(out.path().join("other/index.html")).unwrap();
    assert!(!other.contains("href=\"#partners\""));
    assert!(other.contains("href=\"/#partners\""));
    assert!(!other.contains("href=\"/other/#partners\""));
}

#[test]
fn company_links_and_output_collisions_fail_before_writing() {
    for raw in [
        COMPANY.replace(
            "role = \"Founder\"",
            "role = \"Founder\"\nlinks = [{ label = \"Profile\", href = \"javascript:alert(1)\" }]",
        ),
        COMPANY.replace("slug = \"index\"", "slug = \"style.css\""),
        COMPANY.replace("id = \"mission\"", "id = \"main-content\""),
        COMPANY.replace(
            "[company]\n",
            "[company]\nlogo_src = \"//example.com/logo.svg\"\n",
        ),
    ] {
        let site = load(&raw).unwrap();
        let out = tempfile::tempdir().unwrap();
        std::fs::write(out.path().join("preserve.txt"), "keep").unwrap();
        assert!(render_static(&site, &RenderOptions::new(out.path())).is_err());
        assert!(out.path().join("preserve.txt").exists());
    }
}

#[test]
fn company_metadata_uses_canonical_origin_and_script_safe_json() {
    let raw = COMPANY
        .replace("North & Co", "North </script><script>Co")
        .replace(
            "description = \"An independent studio.\"",
            "description = \"An independent studio.\"\ncanonical_domain = \"company.example\"",
        );
    let site = load(&raw).unwrap();
    let out = tempfile::tempdir().unwrap();
    render_static(&site, &RenderOptions::new(out.path())).unwrap();
    let html = std::fs::read_to_string(out.path().join("index.html")).unwrap();
    let json = html
        .split("<script type=\"application/ld+json\">")
        .nth(1)
        .unwrap()
        .split("</script>")
        .next()
        .unwrap();
    let value: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_eq!(value["name"], "North </script><script>Co");
    assert_eq!(value["url"], "https://company.example/");
    assert!(html.contains("<link rel=\"canonical\" href=\"https://company.example/\">"));
}

#[cfg(feature = "brick-hero")]
#[test]
fn company_audit_rejects_report_overwrites_before_rendering() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("company.toml");
    let original = include_str!("../../../website/company-examples/north.toml");
    std::fs::write(&config, original).unwrap();
    let out = dir.path().join("public");
    std::fs::create_dir(&out).unwrap();
    std::fs::write(out.join("sentinel"), "existing output").unwrap();
    for flags in [
        vec!["--report", "company.toml"],
        vec!["--shared-capture", "--capture-manifest", "company.toml"],
        vec!["--shared-capture", "--visual-report", "company.toml"],
        vec!["--shared-capture", "--capture-output", "."],
        vec!["--shared-capture", "--rubric-cache", "."],
    ] {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_plinth-company"))
            .current_dir(dir.path())
            .args([
                "audit",
                "site",
                "--config",
                "company.toml",
                "--out",
                "public",
            ])
            .args(&flags)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("protected input"),
            "{flags:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(std::fs::read_to_string(&config).unwrap(), original);
        assert_eq!(
            std::fs::read_to_string(out.join("sentinel")).unwrap(),
            "existing output"
        );
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&config, dir.path().join("report.json")).unwrap();
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_plinth-company"))
            .current_dir(dir.path())
            .args([
                "audit",
                "site",
                "--config",
                "company.toml",
                "--out",
                "public",
                "--report",
                "report.json",
            ])
            .output()
            .unwrap();
        assert!(String::from_utf8_lossy(&output.stderr).contains("protected input"));
        assert_eq!(std::fs::read_to_string(&config).unwrap(), original);
        assert!(dir.path().join("report.json").is_symlink());
        assert!(out.join("sentinel").exists());
        std::os::unix::fs::symlink(&config, dir.path().join("capture_job.json")).unwrap();
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_plinth-company"))
            .current_dir(dir.path())
            .args([
                "audit",
                "site",
                "--config",
                "company.toml",
                "--out",
                "public",
                "--shared-capture",
                "--capture-manifest",
                "capture_manifest.json",
            ])
            .output()
            .unwrap();
        assert!(String::from_utf8_lossy(&output.stderr).contains("protected input"));
        assert_eq!(std::fs::read_to_string(&config).unwrap(), original);
        assert!(dir.path().join("capture_job.json").is_symlink());
        assert!(out.join("sentinel").exists());
    }
}

#[cfg(feature = "brick-hero")]
#[test]
fn company_audit_rejects_generated_destination_collisions_before_cleanup() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("company.toml"),
        include_str!("../../../website/company-examples/north.toml"),
    )
    .unwrap();
    std::fs::create_dir_all(dir.path().join("generated/public")).unwrap();
    std::fs::write(
        dir.path().join("generated/public/sentinel"),
        "existing output",
    )
    .unwrap();
    for flags in [
        vec!["--screenshots", "generated/public"],
        vec!["--screenshots", "generated"],
        vec!["--screenshots", "generated/public/screenshots"],
        vec!["--report", "generated/public/index.html"],
        vec!["--report", "target/site-audit/index/desktop.png"],
        vec!["--shared-capture", "--capture-output", "generated/public"],
        vec![
            "--shared-capture",
            "--report",
            "target/visual/run_report.json",
        ],
    ] {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_plinth-company"))
            .current_dir(dir.path())
            .args([
                "audit",
                "site",
                "--config",
                "company.toml",
                "--out",
                "generated/public",
            ])
            .args(&flags)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("overlap"),
            "{flags:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            std::fs::read_to_string(dir.path().join("generated/public/sentinel")).unwrap(),
            "existing output"
        );
    }
}

#[cfg(feature = "brick-hero")]
#[test]
fn company_cli_round_trips_init_check_inspect_build_and_publish() {
    let dir = tempfile::tempdir().unwrap();
    let run = |args: &[&str]| {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_plinth-company"))
            .current_dir(dir.path())
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        output.stdout
    };
    run(&[
        "init",
        "--title",
        "Example & Co",
        "--description",
        "A useful company.",
    ]);
    assert!(dir.path().join("website/plinth-company.toml").exists());
    for args in [
        vec!["build", "--out", "website"],
        vec!["build", "--out", "website", "--json"],
        vec!["publish", "--out", "website"],
        vec!["dev", "--out", "website", "--no-open", "--no-watch"],
        vec!["audit", "site", "--out", "website"],
        vec!["audit", "site", "--screenshots", "website"],
    ] {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_plinth-company"))
            .current_dir(dir.path())
            .env("PLINTH_PROJECT_SERVE_ONCE", "1")
            .args(&args)
            .output()
            .unwrap();
        assert!(
            !output.status.success(),
            "{args:?} accepted destructive output"
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("protected input"),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(dir.path().join("website/plinth-company.toml").exists());
        if args.contains(&"--json") {
            assert!(
                output.stdout.is_empty(),
                "rejected output must not emit a successful build report"
            );
        }
    }
    let check: serde_json::Value = serde_json::from_slice(&run(&["check", "--json"])).unwrap();
    assert_eq!(check["ok"], true);
    let inspection: serde_json::Value =
        serde_json::from_slice(&run(&["inspect", "--json"])).unwrap();
    assert_eq!(inspection["company"]["motion"]["motif"], "contour");
    assert_eq!(inspection["members"][0]["id"], "founder");
    run(&["build"]);
    assert!(dir.path().join("website/public/index.html").exists());
    run(&["publish"]);
    let manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read(
            dir.path()
                .join("dist/plinth-company/plinth-company-manifest.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(manifest["generator"], "plinth-company");
    assert!(
        manifest["files"]
            .as_array()
            .unwrap()
            .iter()
            .any(|path| path == "company.js")
    );
}

#[test]
fn company_pages_keep_site_stylesheets_and_reject_broker_themes() {
    let site = load(&COMPANY.replace("[company]\n", "stylesheets = [\"/extra.css\"]\n[company]\n"))
        .unwrap();
    let out = tempfile::tempdir().unwrap();
    render_static(&site, &RenderOptions::new(out.path().join("public"))).unwrap();
    let html = std::fs::read_to_string(out.path().join("public/index.html")).unwrap();
    assert!(html.contains("<link rel=\"stylesheet\" href=\"/extra.css\">"));

    let site = load(&COMPANY.replace(
        "[company]\n",
        "[theme]\nbroker_css = \"broker.css\"\n[company]\n",
    ))
    .unwrap();
    assert!(
        validate_site(&site)
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "company.broker_css")
    );
}

#[test]
fn company_metadata_falls_back_to_site_branding() {
    let site = load(&COMPANY.replace(
        "[company]\n",
        "favicon = \"/favicon.svg\"\nog_image = \"/social.png\"\n[company]\n",
    ))
    .unwrap();
    let out = tempfile::tempdir().unwrap();
    render_static(&site, &RenderOptions::new(out.path().join("public"))).unwrap();
    let html = std::fs::read_to_string(out.path().join("public/index.html")).unwrap();
    assert!(html.contains("<link rel=\"icon\" href=\"/favicon.svg\">"));
    assert!(html.contains("<meta property=\"og:image\" content=\"/social.png\">"));
}

#[cfg(feature = "brick-blog")]
#[test]
fn company_blog_uses_company_chrome_and_owns_its_directory() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("plinth-company.toml");
    std::fs::write(&config, format!("{COMPANY}\n[blog]\n")).unwrap();
    std::fs::create_dir(dir.path().join("posts")).unwrap();
    std::fs::write(
        dir.path().join("posts/hello.md"),
        "---\ntitle: Hello\ndate: \"2026-11-11\"\n---\nFirst post.\n",
    )
    .unwrap();
    let site = load_project_site(&config).unwrap();
    assert!(validate_site(&site).diagnostics.is_empty());
    let out = dir.path().join("public");
    render_static(&site, &RenderOptions::new(&out)).unwrap();
    let post = std::fs::read_to_string(out.join("blog/hello/index.html")).unwrap();
    assert!(post.contains("class=\"company-site\""));
    assert!(post.contains("href=\"/blog/\""));
    let index = std::fs::read_to_string(out.join("index.html")).unwrap();
    assert!(
        index.contains("href=\"/blog/\""),
        "company nav links the blog"
    );

    std::fs::write(dir.path().join("logo.svg"), "<svg/>").unwrap();
    std::fs::write(
        &config,
        format!(
            "{COMPANY}\n[blog]\n[[assets]]\nsource = \"logo.svg\"\ntarget = \"/blog/index.html\"\n"
        ),
    )
    .unwrap();
    let site = load_project_site(&config).unwrap();
    assert!(
        validate_site(&site)
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "company.output.collision")
    );
}

#[cfg(all(feature = "brick-blog", feature = "brick-hero"))]
#[test]
fn company_audit_refuses_to_overwrite_posts() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("company.toml"),
        format!(
            "{}\n[blog]\n",
            include_str!("../../../website/company-examples/north.toml")
        ),
    )
    .unwrap();
    std::fs::create_dir(dir.path().join("posts")).unwrap();
    let post = "---\ntitle: Draft\ndate: \"2026-11-11\"\npublished: false\n---\nKeep me.\n";
    std::fs::write(dir.path().join("posts/draft.md"), post).unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_plinth-company"))
        .current_dir(dir.path())
        .args([
            "audit",
            "site",
            "--config",
            "company.toml",
            "--out",
            "public",
        ])
        .args(["--report", "posts/draft.md"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("protected input"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(dir.path().join("posts/draft.md")).unwrap(),
        post
    );
}

#[test]
fn company_rejects_unsafe_site_branding_urls() {
    let site = load(&COMPANY.replace(
        "[company]\n",
        "og_image = \"//cdn.example/s.png\"\n[company]\n",
    ))
    .unwrap();
    assert!(
        validate_site(&site)
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "company.image")
    );
}
