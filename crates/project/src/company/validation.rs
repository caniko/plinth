use std::collections::HashSet;

use super::CompanySection;
use crate::{Diagnostic, DiagnosticReport, ProjectSection, ProjectSite};

pub(crate) fn validate(site: &ProjectSite, report: &mut DiagnosticReport) {
    let has_company_sections = site
        .pages
        .iter()
        .flat_map(|p| &p.sections)
        .any(|s| matches!(s, ProjectSection::Company(_)));
    let Some(company) = &site.company else {
        if has_company_sections {
            error(
                report,
                "company.profile",
                "Company sections require a [company] profile.",
            );
        }
        return;
    };
    let motion = &company.motion;
    if site.theme.broker_css.is_some() {
        error(
            report,
            "company.broker_css",
            "Company sites use their own theme; remove [theme] broker_css.",
        );
    }
    required(report, &site.title, "Site title");
    required(report, &site.description, "Site description");
    if site.theme.font_family.is_some() != site.theme.font_url.is_some() {
        error(
            report,
            "company.font",
            "Custom fonts require both font_family and font_url.",
        );
    }
    if let Some(url) = &site.theme.font_url {
        image_url(report, url);
    }
    if !motion.duration_secs.is_finite() || !(8.0..=120.0).contains(&motion.duration_secs) {
        error(
            report,
            "company.motion.duration",
            "Motion duration_secs must be between 8 and 120.",
        );
    }
    if !motion.intensity.is_finite() || !(0.0..=0.6).contains(&motion.intensity) {
        error(
            report,
            "company.motion.intensity",
            "Motion intensity must be between 0 and 0.6.",
        );
    }
    if company.language.is_empty()
        || !company
            .language
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        error(
            report,
            "company.language",
            "Company language must be a language tag such as en or da-DK.",
        );
    }
    let mut members = HashSet::new();
    for member in &site.members {
        if !identifier(&member.id) || !members.insert(member.id.as_str()) {
            error(
                report,
                "company.member.id",
                format!("Invalid or duplicate member ID: {}", member.id),
            );
        }
        required(report, &member.name, "Member name");
        required(report, &member.role, "Member role");
        for link in &member.links {
            link_url(report, &link.href);
            required(report, &link.label, "Link label");
        }
        if let Some(src) = &member.portrait_src {
            image_url(report, src);
        }
    }
    for src in [
        &company.logo_src,
        &company.favicon_src,
        &company.social_image,
    ]
    .into_iter()
    .flatten()
    {
        image_url(report, src);
    }
    let mut pages = HashSet::new();
    if site.pages.is_empty() {
        error(
            report,
            "company.pages",
            "A company site requires at least one page.",
        );
    }
    for page in &site.pages {
        required(report, &page.title, "Page title");
        if !safe_path(&page.slug) || !pages.insert(&page.slug) {
            error(
                report,
                "company.page.slug",
                format!("Unsafe or duplicate page slug: {}", page.slug),
            );
        }
        validate_ids(site, page, report);
        for section in &page.sections {
            #[cfg(feature = "brick-hero")]
            if let ProjectSection::Hero(hero) = section {
                required(report, &hero.title, "Hero title");
                if hero.preview_src.is_some()
                    || hero.preview_mobile_src.is_some()
                    || !hero.preview_alt.is_empty()
                    || hero.logo_src.is_some()
                    || hero.person.is_some()
                {
                    error(
                        report,
                        "company.hero.unsupported",
                        "Company heroes use ambient artwork and company branding; preview images, hero logos, and person bylines are unsupported. Use company.logo_src, team, or content sections instead.",
                    );
                }
                for cta in &hero.ctas {
                    required(report, &cta.label, "CTA label");
                    link_url(report, &cta.href);
                }
            }
            // With only brick-company enabled, the fallback arm is unreachable.
            #[allow(unreachable_patterns)]
            let section = match section {
                ProjectSection::Company(section) => section,
                _ => continue,
            };
            required(report, section.heading(), "Section heading");
            match section {
                CompanySection::Mission(s) => {
                    required(report, &s.statement, "Mission statement");
                    for value in &s.values {
                        required(report, &value.title, "Value title");
                        required(report, &value.description, "Value description");
                    }
                }
                CompanySection::Team(s) => {
                    for id in &s.members {
                        if !members.contains(id.as_str()) {
                            error(
                                report,
                                "company.team.reference",
                                format!("Unknown team member: {id}"),
                            );
                        }
                    }
                    let unique = s.members.iter().collect::<HashSet<_>>();
                    if unique.len() != s.members.len() {
                        error(
                            report,
                            "company.team.duplicate",
                            "Team references must be unique.",
                        );
                    }
                }
                CompanySection::Testimonials(s) => {
                    for q in &s.quotes {
                        required(report, &q.quote, "Testimonial quote");
                        required(report, &q.name, "Testimonial attribution");
                        if let Some(href) = &q.source {
                            link_url(report, href);
                        }
                    }
                }
                CompanySection::Partnerships(s) => {
                    for p in &s.partners {
                        required(report, &p.name, "Partner name");
                        if let Some(href) = &p.href {
                            link_url(report, href);
                        }
                        if let Some(src) = &p.logo_src {
                            image_url(report, src);
                        }
                    }
                }
                CompanySection::Metrics(s) => {
                    for m in &s.items {
                        required(report, &m.value, "Metric value");
                        required(report, &m.label, "Metric label");
                        required(report, &m.context, "Metric context");
                    }
                }
                CompanySection::CaseStudies(s) => {
                    for c in &s.studies {
                        for text in [&c.title, &c.client, &c.problem, &c.approach, &c.outcome] {
                            required(report, text, "Case study content");
                        }
                        if let Some(href) = &c.href {
                            link_url(report, href);
                        }
                        if let Some(src) = &c.image_src {
                            image_url(report, src);
                        }
                    }
                }
                CompanySection::Contact(s) => {
                    required(report, &s.label, "Contact label");
                    link_url(report, &s.href);
                }
            }
        }
    }
    let mut generated = vec![
        "style.css".to_owned(),
        "company.js".to_owned(),
        "sitemap.xml".to_owned(),
        "robots.txt".to_owned(),
        "llms.txt".to_owned(),
        "projects.json".to_owned(),
        "CNAME".to_owned(),
        "plinth-company-manifest.json".to_owned(),
        "plinth-project-manifest.json".to_owned(),
    ];
    // The blog owns its whole directory once it has a published post.
    #[cfg(feature = "brick-blog")]
    if site
        .blog
        .as_ref()
        .is_some_and(|blog| !blog.posts.is_empty())
    {
        generated.push("blog".to_owned());
    }
    generated.extend(site.pages.iter().map(|page| {
        if page.slug == "index" {
            "index.html".into()
        } else {
            format!("{}/index.html", page.slug)
        }
    }));
    if site.theme.font_url.as_deref() == Some(super::FONT_URL) {
        generated.extend([
            super::FONT_URL.trim_start_matches('/').into(),
            "fonts/Inter-LICENSE.txt".into(),
        ]);
    }
    let mut outputs = Vec::new();
    for path in generated
        .iter()
        .map(String::as_str)
        .chain(site.assets.iter().map(|a| a.target.trim_start_matches('/')))
    {
        if outputs.iter().any(|other: &&str| {
            path == *other
                || path.starts_with(&format!("{other}/"))
                || other.starts_with(&format!("{path}/"))
        }) {
            error(
                report,
                "company.output.collision",
                format!("Colliding output path: {path}"),
            );
        }
        outputs.push(path);
    }
    for asset in &site.assets {
        if !safe_path(asset.target.trim_start_matches('/')) {
            error(
                report,
                "company.asset.target",
                format!("Unsafe or reserved asset target: {}", asset.target),
            );
        }
        if !asset.source.is_file() {
            error(
                report,
                "company.asset.source",
                format!("Asset does not exist: {}", asset.source.display()),
            );
        }
    }
    for link in site.nav.iter().chain(&site.footer_links) {
        required(report, &link.label, "Navigation label");
        link_url(report, &link.href);
    }
}

fn validate_ids(site: &ProjectSite, page: &crate::Page, report: &mut DiagnosticReport) {
    let mut anchors = HashSet::from(["main-content".to_owned(), "company-navigation".to_owned()]);
    let mut register = |id: &str| {
        if !identifier(id) || !anchors.insert(id.to_owned()) {
            error(
                report,
                "company.section.id",
                format!("Invalid or duplicate page ID on {}: {id}", page.slug),
            );
        }
    };
    for section in &page.sections {
        let id = match section {
            ProjectSection::Company(section) => Some(section.id()),
            #[cfg(feature = "brick-hero")]
            ProjectSection::Hero(_) => None,
            #[cfg(feature = "brick-feature-grid")]
            ProjectSection::FeatureGrid(s) => s.id.as_deref(),
            #[cfg(feature = "brick-install")]
            ProjectSection::Install(s) => Some(s.id.as_str()),
            #[cfg(feature = "brick-person-mention")]
            ProjectSection::PersonMention(s) => s.id.as_deref(),
            #[cfg(feature = "brick-project-grid")]
            ProjectSection::ProjectGrid(s) => s.id.as_deref(),
            #[cfg(feature = "brick-workflow-steps")]
            ProjectSection::WorkflowSteps(s) => s.id.as_deref(),
            #[cfg(feature = "brick-audience-grid")]
            ProjectSection::AudienceGrid(s) => s.id.as_deref(),
            #[cfg(feature = "brick-trust-panel")]
            ProjectSection::TrustPanel(s) => s.id.as_deref(),
            #[cfg(feature = "brick-screenshot-grid")]
            ProjectSection::ScreenshotGrid(s) => Some(s.id.as_str()),
            #[cfg(feature = "brick-capability-matrix")]
            ProjectSection::CapabilityMatrix(s) => Some(s.id.as_str()),
            #[cfg(feature = "brick-comparison")]
            ProjectSection::Comparison(s) => s.id.as_deref(),
            #[cfg(feature = "brick-content")]
            ProjectSection::Content(s) => (!s.id.is_empty()).then_some(s.id.as_str()),
            #[cfg(feature = "brick-custom")]
            ProjectSection::Custom(_) => None,
        };
        if let Some(id) = id {
            register(id);
        }
        if let ProjectSection::Company(CompanySection::Team(team)) = section {
            for member in &site.members {
                if team.members.is_empty() || team.members.contains(&member.id) {
                    register(&format!("{}-member-{}", team.id, member.id));
                }
            }
        }
    }
}

fn error(report: &mut DiagnosticReport, code: &'static str, message: impl Into<String>) {
    report.diagnostics.push(Diagnostic::error(code, message));
}
fn required(report: &mut DiagnosticReport, text: &str, name: &str) {
    if text.trim().is_empty() {
        error(
            report,
            "company.required",
            format!("{name} must not be empty."),
        );
    }
}
fn identifier(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
}
fn safe_path(text: &str) -> bool {
    !text.is_empty()
        && text.split('/').all(|part| {
            !part.is_empty()
                && part != "."
                && part != ".."
                && part
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        })
}
fn link_url(report: &mut DiagnosticReport, href: &str) {
    if !safe_url(href, true) {
        error(
            report,
            "company.link",
            format!("Unsupported link URL: {href}"),
        );
    }
}
fn image_url(report: &mut DiagnosticReport, src: &str) {
    if !safe_url(src, false) {
        error(
            report,
            "company.image",
            format!("Unsupported image URL: {src}"),
        );
    }
}
fn safe_url(url: &str, link: bool) -> bool {
    !url.chars().any(|c| c.is_control() || c.is_whitespace())
        && !url.starts_with("//")
        && (url.starts_with("https://")
            || url.starts_with("http://")
            || url.starts_with('/')
            || (link
                && (url.starts_with('#') || url.starts_with("mailto:") || url.starts_with("tel:"))))
}
