use super::CompanySection;
#[cfg(feature = "brick-hero")]
use super::MotionMotif;
use super::TeamLayout;
use crate::render::{escape_attr, escape_text, external_attrs};
use crate::{Page, ProjectSection, ProjectSite};

pub(crate) fn metadata(site: &ProjectSite, page: &Page, url: &str) -> String {
    let company = site
        .company
        .as_ref()
        .expect("company renderer requires a profile");
    let mut organization = serde_json::json!({"@context":"https://schema.org", "@type":"Organization", "name":site.title, "description":site.description});
    if url.starts_with("http") {
        organization["url"] = serde_json::json!(absolute_url(url, "/"));
    }
    if let Some(name) = &company.legal_name {
        organization["legalName"] = serde_json::json!(name);
    }
    if let Some(src) = &company.logo_src {
        organization["logo"] = serde_json::json!(absolute_url(url, src));
    }
    let json = organization
        .to_string()
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026");
    let description = if page.description.is_empty() {
        &site.description
    } else {
        &page.description
    };
    let mut head = format!(
        "<meta name=\"generator\" content=\"plinth-company\"><meta property=\"og:type\" content=\"website\"><meta property=\"og:title\" content=\"{}\"><meta property=\"og:description\" content=\"{}\"><script type=\"application/ld+json\">{json}</script>",
        escape_attr(&page.title),
        escape_attr(description)
    );
    if url.starts_with("http") {
        head.push_str(&format!(
            "<link rel=\"canonical\" href=\"{}\"><meta property=\"og:url\" content=\"{}\">",
            escape_attr(url),
            escape_attr(url)
        ));
    }
    if let Some(src) = company.social_image.as_ref().or(site.og_image.as_ref()) {
        head.push_str(&format!(
            "<meta property=\"og:image\" content=\"{}\">",
            escape_attr(&absolute_url(url, src))
        ));
    }
    if let Some(src) = company.favicon_src.as_ref().or(site.favicon.as_ref()) {
        head.push_str(&format!(
            "<link rel=\"icon\" href=\"{}\">",
            escape_attr(src)
        ));
    }
    head.push_str("<script src=\"/company.js\" defer></script>");
    head
}

fn absolute_url(page_url: &str, src: &str) -> String {
    if src.starts_with('/')
        && page_url.starts_with("http")
        && let Some((scheme, rest)) = page_url.split_once("://")
    {
        return format!(
            "{scheme}://{}{}",
            rest.split('/').next().unwrap_or(rest),
            src
        );
    }
    src.into()
}

pub(crate) fn navigation(site: &ProjectSite, page: &Page) -> String {
    let company = site
        .company
        .as_ref()
        .expect("company renderer requires a profile");
    let logo = company.logo_src.as_ref().map_or_else(String::new, |src| {
        format!(
            "<img src=\"{}\" alt=\"\" class=\"company-logo\">",
            escape_attr(src)
        )
    });
    let links = site
        .nav
        .iter()
        .filter(|link| !empty_anchor(site, page, &link.href))
        .map(|link| {
            format!(
                "<a href=\"{}\"{}>{}</a>",
                escape_attr(&link.href),
                if link.primary {
                    " class=\"company-nav-primary\""
                } else {
                    ""
                },
                escape_text(&link.label)
            )
        })
        .collect::<String>();
    format!(
        "<a class=\"company-skip\" href=\"#main-content\">Skip to content</a><header class=\"company-header\"><nav class=\"company-nav\" aria-label=\"Main navigation\"><a class=\"company-brand\" href=\"{}\">{logo}{}</a><button type=\"button\" class=\"company-menu\" aria-controls=\"company-navigation\" aria-expanded=\"false\" hidden>Menu</button><div class=\"company-nav-links\" id=\"company-navigation\">{links}</div></nav></header>",
        escape_attr(if site.base_url.is_empty() {
            "/"
        } else {
            &site.base_url
        }),
        escape_text(&site.title)
    )
}

pub(crate) fn empty_anchor(site: &ProjectSite, page: &Page, href: &str) -> bool {
    let Some((path, id)) = href.split_once('#') else {
        return false;
    };
    let target = if path.is_empty() {
        Some(page)
    } else if path.starts_with('/') {
        let route = path.split('?').next().unwrap_or(path).trim_end_matches('/');
        site.pages.iter().find(|page| {
            if page.slug == "index" {
                route.is_empty()
            } else {
                route.strip_prefix('/') == Some(page.slug.as_str())
            }
        })
    } else {
        None
    };
    target.is_some_and(|page| {
        page.sections.iter().any(|section| {
            matches!(section, ProjectSection::Company(section) if section.id() == id && !has_content(site, section))
        })
    })
}

pub(crate) fn has_content(site: &ProjectSite, section: &CompanySection) -> bool {
    match section {
        CompanySection::Team(s) => {
            if s.members.is_empty() {
                !site.members.is_empty()
            } else {
                !s.members.is_empty()
            }
        }
        CompanySection::Testimonials(s) => !s.quotes.is_empty(),
        CompanySection::Partnerships(s) => !s.partners.is_empty(),
        CompanySection::Metrics(s) => !s.items.is_empty(),
        CompanySection::CaseStudies(s) => !s.studies.is_empty(),
        _ => true,
    }
}

#[cfg(feature = "brick-hero")]
pub(crate) fn hero(site: &ProjectSite, page: &Page, hero: &crate::Hero) -> String {
    let company = site
        .company
        .as_ref()
        .expect("company renderer requires a profile");
    let eyebrow = hero.eyebrow.as_ref().map_or_else(String::new, |s| {
        format!("<p class=\"company-eyebrow\">{}</p>", escape_text(s))
    });
    let ctas = hero
        .ctas
        .iter()
        .filter(|cta| !empty_anchor(site, page, &cta.href))
        .map(|cta| {
            format!(
                "<a class=\"company-action{}\" href=\"{}\">{}</a>",
                if cta.primary {
                    " company-action-primary"
                } else {
                    ""
                },
                escape_attr(&cta.href),
                escape_text(&cta.label)
            )
        })
        .collect::<String>();
    let subtitle = if hero.subtitle.trim().is_empty() || hero.subtitle == hero.tagline {
        String::new()
    } else {
        format!(
            "<p class=\"company-subtitle\">{}</p>",
            escape_text(&hero.subtitle)
        )
    };
    let location = company
        .location
        .as_ref()
        .map_or_else(String::new, |s| escape_text(s));
    let control = if company.motion.enabled && company.motion.intensity > 0.0 {
        "<button class=\"company-motion-toggle\" type=\"button\" aria-pressed=\"false\" hidden>Pause motion <span aria-hidden=\"true\">Ⅱ</span></button>"
    } else {
        ""
    };
    let note = hero.note.as_ref().map_or_else(String::new, |s| {
        format!("<p class=\"company-hero-note\">{}</p>", escape_text(s))
    });
    format!(
        "<section class=\"company-hero\" aria-label=\"Introduction\"><div class=\"company-hero-copy\">{eyebrow}<h1>{}</h1><p class=\"company-tagline\">{}</p>{subtitle}<div class=\"company-actions\">{ctas}</div>{note}</div><div class=\"company-art company-art-{}\" aria-hidden=\"true\">{}</div><div class=\"company-hero-meta\"><p>{location}</p>{control}</div></section>",
        escape_text(&hero.title),
        escape_text(&hero.tagline),
        company.motion.motif.name(),
        artwork(company.motion.motif)
    )
}

#[cfg(feature = "brick-hero")]
fn artwork(motif: MotionMotif) -> String {
    let mut body = String::new();
    match motif {
        MotionMotif::Contour => {
            for i in 0..18 {
                let rx = 100.0 + f64::from(i) * 6.8;
                let ry = 75.0 + f64::from(i) * 8.0;
                let angle = 24.0 + f64::from(i) * 1.7;
                body.push_str(&format!("<ellipse cx=\"260\" cy=\"260\" rx=\"{rx}\" ry=\"{ry}\" transform=\"rotate({angle} 260 260)\"/>"));
            }
        }
        MotionMotif::Orbit => {
            for i in 0..9 {
                body.push_str(&format!("<ellipse cx=\"260\" cy=\"260\" rx=\"{}\" ry=\"{}\" transform=\"rotate({} 260 260)\"/>", 90 + i * 16, 44 + i * 9, i * 20));
            }
            body.push_str("<circle cx=\"260\" cy=\"260\" r=\"5\" class=\"company-orbit-point\"/><circle cx=\"404\" cy=\"148\" r=\"4\" class=\"company-orbit-point\"/>");
        }
        MotionMotif::Ribbon => {
            for i in 0..16 {
                let y = 106 + i * 16;
                body.push_str(&format!(
                    "<path d=\"M 20 {y} C 180 {} 230 {} 500 {}\"/>",
                    y - 100,
                    y + 160,
                    y - 20
                ));
            }
        }
    }
    format!(
        "<svg viewBox=\"0 0 520 520\" fill=\"none\" xmlns=\"http://www.w3.org/2000/svg\"><g class=\"company-motion-layer\">{body}</g></svg>"
    )
}

fn heading(eyebrow: &str, title: &str, intro: &str) -> String {
    let eyebrow = if eyebrow.is_empty() {
        String::new()
    } else {
        format!("<p class=\"company-eyebrow\">{}</p>", escape_text(eyebrow))
    };
    let intro = if intro.is_empty() {
        String::new()
    } else {
        format!(
            "<p class=\"company-section-intro\">{}</p>",
            escape_text(intro)
        )
    };
    format!(
        "<div class=\"company-section-heading\">{eyebrow}<div><h2>{}</h2>{intro}</div></div>",
        escape_text(title)
    )
}

pub(crate) fn section(site: &ProjectSite, section: &CompanySection) -> String {
    if !has_content(site, section) {
        return String::new();
    }
    let content = match section {
        CompanySection::Mission(s) => {
            let vision = s.vision.as_ref().map_or_else(String::new, |v| {
                format!("<p class=\"company-vision\">{}</p>", escape_text(v))
            });
            let values = s
                .values
                .iter()
                .map(|v| {
                    format!(
                        "<article><h3>{}</h3><p>{}</p></article>",
                        escape_text(&v.title),
                        escape_text(&v.description)
                    )
                })
                .collect::<String>();
            format!(
                "{}<div class=\"company-mission-body\"><p class=\"company-statement\">{}</p>{vision}<div class=\"company-values\">{values}</div></div>",
                heading(&s.eyebrow, &s.heading, ""),
                escape_text(&s.statement)
            )
        }
        CompanySection::Team(s) => {
            let members = if s.members.is_empty() {
                site.members.iter().collect::<Vec<_>>()
            } else {
                s.members
                    .iter()
                    .filter_map(|id| site.members.iter().find(|m| &m.id == id))
                    .collect()
            };
            let compact = matches!(s.layout, TeamLayout::Compact);
            let cards = members.into_iter().map(|m| {
                let initials = m.name.split_whitespace().filter_map(|word| word.chars().next()).take(2).collect::<String>();
                let portrait = |alt: &str| m.portrait_src.as_ref().map_or_else(
                    || format!("<div class=\"company-initials\" aria-hidden=\"true\"><span>{}</span></div>", escape_text(&initials)),
                    |src| format!("<img class=\"company-portrait\" src=\"{}\" alt=\"{}\" width=\"640\" height=\"480\" loading=\"lazy\">", escape_attr(src), escape_attr(alt)),
                );
                if compact {
                    return format!("<article class=\"company-member-compact\" id=\"{}-member-{}\">{}<h3>{}</h3><p>{}</p></article>", escape_attr(&s.id), escape_attr(&m.id), portrait(""), escape_text(&m.name), escape_text(&m.role));
                }
                let links = m.links.iter().map(|l| format!("<a href=\"{}\"{}>{} ↗</a>", escape_attr(&l.href), external_attrs(&l.href), escape_text(&l.label))).collect::<String>();
                format!("<article class=\"company-member\" id=\"{}-member-{}\">{}<div class=\"company-member-name\"><h3>{}</h3><p>{}</p></div><p class=\"company-member-bio\">{}</p><div class=\"company-member-links\">{links}</div></article>", escape_attr(&s.id), escape_attr(&m.id), portrait(&m.name), escape_text(&m.name), escape_text(&m.role), escape_text(&m.bio))
            }).collect::<String>();
            let grid = format!(
                "<div class=\"{}\">{cards}</div>",
                if compact {
                    "company-team-strip"
                } else {
                    "company-team-grid"
                }
            );
            let link = s.link.as_ref().map_or_else(String::new, |l| {
                format!(
                    "<a class=\"company-action company-team-link\" href=\"{}\"{}>{}</a>",
                    escape_attr(&l.href),
                    external_attrs(&l.href),
                    escape_text(&l.label)
                )
            });
            format!("{}{grid}{link}", heading(&s.eyebrow, &s.heading, &s.intro))
        }
        CompanySection::Testimonials(s) => {
            let quotes = s.quotes.iter().map(|q| {
                let attribution = [q.role.as_deref(), q.organization.as_deref()].into_iter().flatten().map(escape_text).collect::<Vec<_>>().join(" · ");
                let source = q.source.as_ref().map_or_else(String::new, |url| format!("<a href=\"{}\"{}>Read the source ↗</a>", escape_attr(url), external_attrs(url)));
                format!("<figure class=\"company-quote\"><blockquote><p>“{}”</p></blockquote><figcaption><strong>{}</strong><span>{attribution}</span>{source}</figcaption></figure>", escape_text(&q.quote), escape_text(&q.name))
            }).collect::<String>();
            format!(
                "{}<div class=\"company-quotes\">{quotes}</div>",
                heading(&s.eyebrow, &s.heading, &s.intro)
            )
        }
        CompanySection::Partnerships(s) => {
            let partners = s.partners.iter().map(|p| {
                let logo = p.logo_src.as_ref().map_or_else(|| format!("<span class=\"company-partner-wordmark\">{}</span>", escape_text(&p.name)), |src| format!("<img src=\"{}\" alt=\"{}\" width=\"180\" height=\"64\" loading=\"lazy\">", escape_attr(src), escape_attr(&p.name)));
                let identity = p.href.as_ref().map_or_else(|| logo.clone(), |url| format!("<a href=\"{}\"{}>{logo}</a>", escape_attr(url), external_attrs(url)));
                format!("<article class=\"company-partner\">{identity}<p class=\"company-partner-type\">{}</p><p>{}</p></article>", p.relationship.label(), escape_text(&p.description))
            }).collect::<String>();
            format!(
                "{}<div class=\"company-partners\">{partners}</div>",
                heading(&s.eyebrow, &s.heading, &s.intro)
            )
        }
        CompanySection::Metrics(s) => {
            let items = s
                .items
                .iter()
                .map(|m| {
                    format!(
                        "<div><dt>{}</dt><dd><strong>{}</strong><span>{}</span></dd></div>",
                        escape_text(&m.label),
                        escape_text(&m.value),
                        escape_text(&m.context)
                    )
                })
                .collect::<String>();
            format!(
                "{}<dl class=\"company-metric-grid\">{items}</dl>",
                heading(&s.eyebrow, &s.heading, "")
            )
        }
        CompanySection::CaseStudies(s) => {
            let studies = s.studies.iter().map(|c| {
                let image = c.image_src.as_ref().map_or_else(String::new, |src| format!("<img src=\"{}\" alt=\"{}\" width=\"640\" height=\"400\" loading=\"lazy\">", escape_attr(src), escape_attr(&c.title)));
                let link = c.href.as_ref().map_or_else(String::new, |url| format!("<a href=\"{}\"{}>Explore the work ↗</a>", escape_attr(url), external_attrs(url)));
                format!("<article class=\"company-case\">{image}<p class=\"company-eyebrow\">{}</p><h3>{}</h3><dl><dt>The challenge</dt><dd>{}</dd><dt>Our approach</dt><dd>{}</dd><dt>The outcome</dt><dd>{}</dd></dl>{link}</article>", escape_text(&c.client), escape_text(&c.title), escape_text(&c.problem), escape_text(&c.approach), escape_text(&c.outcome))
            }).collect::<String>();
            format!(
                "{}<div class=\"company-cases\">{studies}</div>",
                heading(&s.eyebrow, &s.heading, &s.intro)
            )
        }
        CompanySection::Contact(s) => format!(
            "{}<a class=\"company-action company-action-primary\" href=\"{}\">{}</a>",
            heading(&s.eyebrow, &s.heading, &s.intro),
            escape_attr(&s.href),
            escape_text(&s.label)
        ),
    };
    let team_size = match section {
        CompanySection::Team(s) if matches!(s.layout, TeamLayout::Compact) => {
            Some(s.size.unwrap_or_default())
        }
        _ => None,
    };
    format!(
        "<section id=\"{}\" class=\"company-section company-{}{}\">{content}</section>",
        escape_attr(section.id()),
        section.name().replace('_', "-"),
        team_size.map_or_else(String::new, |size| format!(
            " company-team-compact company-team-{}",
            size.name()
        ))
    )
}
