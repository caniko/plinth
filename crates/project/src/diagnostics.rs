#[cfg(feature = "brick-install")]
use crate::ProjectSection;
use crate::ProjectSite;
use serde::Serialize;

#[cfg(feature = "brick-install")]
pub use crate::bricks::install::{InstallRouteUxFinding, InstallUxReport};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: &'static str,
    pub message: String,
}

impl Diagnostic {
    #[cfg(feature = "brick-install")]
    pub(crate) fn error(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            code,
            message: message.into(),
        }
    }

    #[cfg(feature = "brick-install")]
    pub(crate) fn warning(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            code,
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct DiagnosticReport {
    pub diagnostics: Vec<Diagnostic>,
}

impl DiagnosticReport {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
    }
}

pub fn validate_site(site: &ProjectSite) -> DiagnosticReport {
    #[allow(unused_mut)]
    let mut report = DiagnosticReport::default();

    for page in &site.pages {
        #[allow(unused_variables)]
        for section in &page.sections {
            #[cfg(feature = "brick-install")]
            if let ProjectSection::Install(install) = section {
                crate::bricks::install::validate_install_section(install, &mut report);
            }
        }
    }

    report
}

#[cfg(feature = "brick-install")]
pub fn install_ux_report(site: &ProjectSite) -> Option<InstallUxReport> {
    site.pages
        .iter()
        .flat_map(|page| &page.sections)
        .find_map(|section| match section {
            ProjectSection::Install(install) => {
                Some(crate::bricks::install::build_install_ux_report(install))
            }
            _ => None,
        })
}

pub fn assert_valid(site: &ProjectSite) -> Result<(), DiagnosticReport> {
    let report = validate_site(site);
    if report.has_errors() {
        Err(report)
    } else {
        Ok(())
    }
}

#[cfg(test)]
#[cfg(feature = "brick-install")]
mod tests {
    use crate::{
        InstallRoute, InstallSection, Page, ProjectSection, ProjectSite, assert_valid,
        install_ux_report, validate_site,
    };

    fn site_with_install(install: InstallSection) -> ProjectSite {
        ProjectSite::new("example", "example site")
            .page(Page::new("index", "example").section(ProjectSection::Install(install)))
    }

    #[test]
    fn rejects_choice_overload_and_missing_recommendation() {
        let install = InstallSection {
            id: "install".into(),
            heading: "Install".into(),
            intro: String::new(),
            guide_href: "/docs/install.html".into(),
            primary_routes: (0..5)
                .map(|idx| InstallRoute::new(format!("Route {idx}"), "Users", "#"))
                .collect(),
            secondary_routes: Vec::new(),
        };

        let report = validate_site(&site_with_install(install));
        let codes: Vec<_> = report
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect();
        assert!(codes.contains(&"install.choice_overload"));
        assert!(codes.contains(&"install.no_recommended_route"));
    }

    #[test]
    fn rejects_overloaded_route_and_misleading_label() {
        let install = InstallSection {
            id: "install".into(),
            heading: "Install".into(),
            intro: String::new(),
            guide_href: "/docs/install.html".into(),
            primary_routes: vec![
                InstallRoute::new("Install now", "Desktop users", "/docs/install.html#linux")
                    .command("one\ntwo\nthree")
                    .recommended(),
            ],
            secondary_routes: Vec::new(),
        };

        let report = validate_site(&site_with_install(install));
        let codes: Vec<_> = report
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect();
        assert!(codes.contains(&"install.overloaded_route"));
        assert!(codes.contains(&"install.misleading_cta"));
    }

    #[test]
    fn accepts_guided_install_section() {
        let install = InstallSection {
            id: "install".into(),
            heading: "Install".into(),
            intro: String::new(),
            guide_href: "/docs/install.html".into(),
            primary_routes: vec![
                InstallRoute::new("Linux desktop", "GUI users", "/docs/install.html#flatpak")
                    .command("flatpak install flathub com.example.App")
                    .recommended(),
                InstallRoute::new("macOS", "Homebrew users", "/docs/install.html#homebrew")
                    .command("brew install example"),
            ],
            secondary_routes: vec![InstallRoute::new(
                "Build from source",
                "Developers",
                "/docs/install.html#build-from-source",
            )],
        };

        assert!(assert_valid(&site_with_install(install)).is_ok());
    }

    #[test]
    fn reports_install_ux_facts() {
        let install = InstallSection {
            id: "install".into(),
            heading: "Install".into(),
            intro: String::new(),
            guide_href: "/docs/install.html".into(),
            primary_routes: vec![
                InstallRoute::new("Linux desktop", "GUI users", "/docs/install.html#flatpak")
                    .command("flatpak install flathub com.example.App")
                    .recommended(),
                InstallRoute::new("Loose docs", "Docs", "/docs/install.html"),
            ],
            secondary_routes: Vec::new(),
        };

        let report = install_ux_report(&site_with_install(install)).unwrap();
        assert_eq!(report.primary_route_count, 2);
        assert_eq!(report.recommended_routes, vec!["Linux desktop"]);
        assert_eq!(report.routes_missing_anchors, vec!["Loose docs"]);
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.code == "install.route_without_anchor")
        );
    }
}
