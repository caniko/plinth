#![warn(missing_docs)]

//! Identity and linking data models for people and projects.
//!
//! Provides `PersonReference`, `ProjectReference`, and `ExternalLink` types
//! used by the Plinth project-site generator and shared crate to describe
//! personal identity metadata, canonical project links, and typed external
//! references (source, demo, docs, social).

use serde::{Deserialize, Serialize};

/// A single named external link with a typed kind.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExternalLink {
    /// Display text for the link.
    pub label: String,
    /// Target URL.
    pub href: String,
    /// Semantic category used for icon/label selection.
    #[serde(default)]
    pub kind: LinkKind,
}

impl ExternalLink {
    /// Create a new external link.
    #[must_use]
    pub fn new(label: impl Into<String>, href: impl Into<String>, kind: LinkKind) -> Self {
        Self {
            label: label.into(),
            href: href.into(),
            kind,
        }
    }
}

/// Classifies an external link into a semantic category.
///
/// Used to render appropriate icons and labels on profile/project pages.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkKind {
    /// Personal home page or profile.
    Person,
    /// Landing / project site for the linked project.
    ProjectSite,
    /// Source code repository.
    Source,
    /// Live demo or hosted instance.
    Demo,
    /// Documentation site or API reference.
    Docs,
    /// Contact form, email, or messaging.
    Contact,
    /// Fallback for unrecognized or uncategorized links.
    #[default]
    Other,
}

impl LinkKind {
    /// Human-readable default label for this link kind.
    #[must_use]
    pub fn default_label(&self) -> &'static str {
        match self {
            Self::Person => "Person",
            Self::ProjectSite => "Project site",
            Self::Source => "Source",
            Self::Demo => "Demo",
            Self::Docs => "Docs",
            Self::Contact => "Contact",
            Self::Other => "Link",
        }
    }
}

/// Reference to a person associated with a project or site.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct PersonReference {
    /// Unique identifier for the person (used as anchor ID).
    pub id: String,
    /// Full display name.
    pub name: String,
    /// Canonical URL for the person's landing page.
    pub url: String,
    /// Role or title displayed alongside the name (e.g. "Maintainer").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// URL for an avatar/portrait image.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    /// Additional external links (social, contact, etc.).
    #[serde(default)]
    pub links: Vec<ExternalLink>,
}

impl PersonReference {
    /// Convenience constructor wrapping `(name, url)` as a `LinkKind::Person` link.
    #[must_use]
    pub fn primary_link(&self) -> ExternalLink {
        ExternalLink::new(&self.name, &self.url, LinkKind::Person)
    }
}

/// Reference to a linked project with its canonical + extra links.
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProjectReference {
    /// Project display name.
    pub title: String,
    /// Canonical project URL (the primary landing page).
    pub url: String,
    /// Optional URL to the source code repository.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    /// Optional URL to a live demo / hosted instance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub demo_url: Option<String>,
    /// Additional external links (docs, community, etc.).
    #[serde(default)]
    pub links: Vec<ExternalLink>,
}

impl ProjectReference {
    /// Collect all links, ordering canonical fields (project site, source, demo)
    /// before any free-form [`links`](Self::links).
    #[must_use]
    pub fn links(&self) -> Vec<ExternalLink> {
        let mut links = Vec::new();
        if !self.url.is_empty() {
            links.push(ExternalLink::new(
                "Project site",
                &self.url,
                LinkKind::ProjectSite,
            ));
        }
        if let Some(source_url) = &self.source_url {
            links.push(ExternalLink::new("Source", source_url, LinkKind::Source));
        }
        if let Some(demo_url) = &self.demo_url {
            links.push(ExternalLink::new("Demo", demo_url, LinkKind::Demo));
        }
        links.extend(self.links.clone());
        links
    }
}

/// Trim whitespace from every link's label and href, and drop entries
/// where either field would be empty after trimming.
pub fn normalized_links(links: impl IntoIterator<Item = ExternalLink>) -> Vec<ExternalLink> {
    links
        .into_iter()
        .map(|mut link| {
            link.label = link.label.trim().to_string();
            link.href = link.href.trim().to_string();
            link
        })
        .filter(|link| !link.label.is_empty() && !link.href.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{ExternalLink, LinkKind, ProjectReference, normalized_links};

    #[test]
    fn project_reference_orders_canonical_links_first() {
        let project = ProjectReference {
            title: "Tool".into(),
            url: "https://tool.example".into(),
            source_url: Some("https://source.example".into()),
            demo_url: Some("https://demo.example".into()),
            links: vec![ExternalLink::new(
                "Docs",
                "https://docs.example",
                LinkKind::Docs,
            )],
        };

        let links = project.links();
        assert_eq!(links[0].kind, LinkKind::ProjectSite);
        assert_eq!(links[1].kind, LinkKind::Source);
        assert_eq!(links[2].kind, LinkKind::Demo);
        assert_eq!(links[3].kind, LinkKind::Docs);
    }

    #[test]
    fn normalized_links_drops_incomplete_entries() {
        let links = normalized_links([
            ExternalLink::new(" Docs ", " https://docs.example ", LinkKind::Docs),
            ExternalLink::new("", "https://missing-label.example", LinkKind::Other),
            ExternalLink::new("Missing href", "", LinkKind::Other),
        ]);

        assert_eq!(links.len(), 1);
        assert_eq!(links[0].label, "Docs");
        assert_eq!(links[0].href, "https://docs.example");
    }
}
