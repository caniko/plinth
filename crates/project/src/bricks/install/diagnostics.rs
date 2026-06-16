use serde::Serialize;

use crate::diagnostics::{Diagnostic, DiagnosticReport};

use super::InstallSection;

/// UX diagnostics report for an [`InstallSection`].
///
/// Produced by [`build_install_ux_report`]; surfaced in project validation
/// output to catch common usability issues like missing anchors or long commands.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct InstallUxReport {
    /// The `id` of the install section this report covers.
    pub section_id: String,
    /// Number of primary install routes.
    pub primary_route_count: usize,
    /// Number of secondary install routes.
    pub secondary_route_count: usize,
    /// Labels of routes marked as recommended.
    pub recommended_routes: Vec<String>,
    /// Count of routes that expose a copyable command.
    pub routes_with_commands: usize,
    /// Labels of routes whose `href` lacks an anchor fragment.
    pub routes_missing_anchors: Vec<String>,
    /// Detailed findings per-route.
    pub findings: Vec<InstallRouteUxFinding>,
}

/// A single UX finding for an install route.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct InstallRouteUxFinding {
    /// The route label this finding applies to.
    pub route: String,
    /// Machine-readable diagnostic code (e.g. `"install.long_command"`).
    pub code: &'static str,
    /// Human-readable explanation of the issue.
    pub message: String,
}

/// Build a UX diagnostics report for an [`InstallSection`].
///
/// Checks every route (primary and secondary) for:
/// - `install.recommended_without_command` — recommended route missing a command
/// - `install.long_command` — command exceeds 96 characters
/// - `install.route_without_anchor` — href lacks an anchor fragment
/// - `install.missing_audience` — audience field is empty or whitespace-only
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

/// Validate an [`InstallSection`] and append diagnostics to `report`.
///
/// Checks:
/// - `install.no_recommended_route` — no route marked recommended
/// - `install.too_many_recommended_routes` — more than one marked recommended
/// - `install.choice_overload` — more than 4 primary routes
/// - `install.overloaded_route` — command spans more than 2 lines
/// - `install.misleading_cta` — doc link with a generic label
/// - `install.unanchored_guide_link` — route href equals the full guide href
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
