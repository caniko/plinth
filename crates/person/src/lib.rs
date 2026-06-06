use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExternalLink {
    pub label: String,
    pub href: String,
    #[serde(default)]
    pub kind: LinkKind,
}

impl ExternalLink {
    #[must_use]
    pub fn new(label: impl Into<String>, href: impl Into<String>, kind: LinkKind) -> Self {
        Self {
            label: label.into(),
            href: href.into(),
            kind,
        }
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkKind {
    Person,
    ProjectSite,
    Source,
    Demo,
    Docs,
    Contact,
    #[default]
    Other,
}

impl LinkKind {
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

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct PersonReference {
    pub id: String,
    pub name: String,
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub links: Vec<ExternalLink>,
}

impl PersonReference {
    #[must_use]
    pub fn primary_link(&self) -> ExternalLink {
        ExternalLink::new(&self.name, &self.url, LinkKind::Person)
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProjectReference {
    pub title: String,
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub demo_url: Option<String>,
    #[serde(default)]
    pub links: Vec<ExternalLink>,
}

impl ProjectReference {
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
