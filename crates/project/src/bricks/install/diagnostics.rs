use serde::Serialize;

use crate::diagnostics::{Diagnostic, DiagnosticReport};

use super::InstallSection;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct InstallUxReport {
    pub section_id: String,
    pub primary_route_count: usize,
    pub secondary_route_count: usize,
    pub recommended_routes: Vec<String>,
    pub routes_with_commands: usize,
    pub routes_missing_anchors: Vec<String>,
    pub findings: Vec<InstallRouteUxFinding>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct InstallRouteUxFinding {
    pub route: String,
    pub code: &'static str,
    pub message: String,
}

pub fn build_install_ux_report(install: &InstallSection) -> InstallUxReport {
    let routes = install
        .primary_routes
        .iter()
        .chain(&install.secondary_routes)
        .collect::<Vec<_>>();
    let mut findings = Vec::new();

    for route in &routes {
        if route.command.is_none() && route.recommended {
            findings.push(InstallRouteUxFinding {
                route: route.label.clone(),
                code: "install.recommended_without_command",
                message: "recommended install route should expose a copyable command".into(),
            });
        }

        if route
            .command
            .as_ref()
            .is_some_and(|command| command.len() > 96)
        {
            findings.push(InstallRouteUxFinding {
                route: route.label.clone(),
                code: "install.long_command",
                message: "route command is long enough to risk horizontal overflow".into(),
            });
        }

        if !route.href.contains('#') {
            findings.push(InstallRouteUxFinding {
                route: route.label.clone(),
                code: "install.route_without_anchor",
                message: "route should link to a focused install-guide anchor".into(),
            });
        }

        if route.audience.trim().is_empty() {
            findings.push(InstallRouteUxFinding {
                route: route.label.clone(),
                code: "install.missing_audience",
                message: "route should name the user segment or package channel".into(),
            });
        }
    }

    InstallUxReport {
        section_id: install.id.clone(),
        primary_route_count: install.primary_routes.len(),
        secondary_route_count: install.secondary_routes.len(),
        recommended_routes: routes
            .iter()
            .filter(|route| route.recommended)
            .map(|route| route.label.clone())
            .collect(),
        routes_with_commands: routes
            .iter()
            .filter(|route| route.command.is_some())
            .count(),
        routes_missing_anchors: routes
            .iter()
            .filter(|route| !route.href.contains('#'))
            .map(|route| route.label.clone())
            .collect(),
        findings,
    }
}

pub fn validate_install_section(install: &InstallSection, report: &mut DiagnosticReport) {
    let recommended = install
        .primary_routes
        .iter()
        .chain(&install.secondary_routes)
        .filter(|route| route.recommended)
        .count();

    if recommended == 0 {
        report.diagnostics.push(Diagnostic::error(
            "install.no_recommended_route",
            "install section must mark one route as recommended",
        ));
    }

    if recommended > 1 {
        report.diagnostics.push(Diagnostic::error(
            "install.too_many_recommended_routes",
            "install section must not mark multiple routes as recommended",
        ));
    }

    if install.primary_routes.len() > 4 {
        report.diagnostics.push(Diagnostic::error(
            "install.choice_overload",
            format!(
                "install section has {} primary routes; use at most 4",
                install.primary_routes.len()
            ),
        ));
    }

    for route in install
        .primary_routes
        .iter()
        .chain(&install.secondary_routes)
    {
        if route.command.as_deref().unwrap_or_default().lines().count() > 2 {
            report.diagnostics.push(Diagnostic::error(
                "install.overloaded_route",
                format!(
                    "`{}` includes too many commands; route cards should show one default action",
                    route.label
                ),
            ));
        }

        let label = route.label.to_ascii_lowercase();
        let docs_link = route.href.contains("/docs/") || route.href.ends_with(".html");
        if docs_link && (label == "install" || label == "install now") {
            report.diagnostics.push(Diagnostic::error(
                "install.misleading_cta",
                format!(
                    "`{}` points to documentation; use a more specific label",
                    route.label
                ),
            ));
        }

        if route.href == install.guide_href {
            report.diagnostics.push(Diagnostic::warning(
                "install.unanchored_guide_link",
                format!(
                    "`{}` links to the full guide without an anchor; prefer a route-specific anchor",
                    route.label
                ),
            ));
        }
    }
}
