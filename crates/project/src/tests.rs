#[test]
fn config_resolution_uses_current_directory_only_order() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("website")).unwrap();
    std::fs::write(dir.path().join("plinth-project.toml"), "").unwrap();
    std::fs::write(dir.path().join(crate::DEFAULT_CONFIG), "").unwrap();

    let resolved = crate::resolve_config_path_in(dir.path(), None).unwrap();
    assert_eq!(resolved, dir.path().join(crate::DEFAULT_CONFIG));
}

#[test]
fn config_resolution_falls_back_to_root_file() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(crate::FALLBACK_CONFIG), "").unwrap();

    let resolved = crate::resolve_config_path_in(dir.path(), None).unwrap();
    assert_eq!(resolved, dir.path().join(crate::FALLBACK_CONFIG));
}

#[test]
fn missing_config_error_names_init_and_config_flag() {
    let dir = tempfile::tempdir().unwrap();
    let error = crate::resolve_config_path_in(dir.path(), None)
        .unwrap_err()
        .to_string();
    assert!(error.contains("plinth-project init"));
    assert!(error.contains("--config"));
    assert!(error.contains(crate::DEFAULT_CONFIG));
}

#[test]
fn diagnostic_human_format_is_stable() {
    let diagnostic = plinth_project::Diagnostic {
        severity: plinth_project::Severity::Error,
        code: "install.no_recommended_route",
        message: "install section must mark one route as recommended".into(),
    };

    assert_eq!(
        crate::format_diagnostic(&diagnostic),
        "error[install.no_recommended_route]: install section must mark one route as recommended"
    );
}

#[test]
fn diagnostic_json_format_is_stable() {
    let report = plinth_project::DiagnosticReport {
        diagnostics: vec![plinth_project::Diagnostic {
            severity: plinth_project::Severity::Warning,
            code: "install.unanchored_guide_link",
            message: "prefer a route-specific anchor".into(),
        }],
    };
    let json = serde_json::to_string(&report).unwrap();

    assert_eq!(
        json,
        r#"{"diagnostics":[{"severity":"warning","code":"install.unanchored_guide_link","message":"prefer a route-specific anchor"}]}"#
    );
}

#[test]
fn publish_manifest_includes_generated_files_and_metadata() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("index.html"), "home").unwrap();
    std::fs::write(dir.path().join("style.css"), "body{}").unwrap();
    let site = plinth_project::ProjectSite::new("Example", "Example site")
        .page(plinth_project::Page::new("index", "Example"));

    let manifest = crate::cmd::publish::publish_manifest(&site, dir.path()).unwrap();

    assert_eq!(manifest.generator, "plinth-project");
    assert_eq!(manifest.title, "Example");
    assert_eq!(manifest.pages[0].path, "index.html");
    assert_eq!(manifest.files, vec!["index.html", "style.css"]);
}

#[test]
fn init_template_escapes_toml_strings() {
    let template =
        crate::cmd::init::initial_config_template("A \"quoted\" site", "Path \\\\ ready");
    toml::from_str::<plinth_project::ProjectConfig>(&template).unwrap();
    assert!(template.contains("A \\\"quoted\\\" site"));
}
