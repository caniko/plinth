use serde::Deserialize;

/// An operating system an install section can claim support for.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Linux,
    Macos,
    Windows,
}

impl Platform {
    /// Token used in config and in the rendered `data-platforms` attribute.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Linux => "linux",
            Self::Macos => "macos",
            Self::Windows => "windows",
        }
    }
}

/// Model for the install / getting-started section.
///
/// Rendered as `<section class="install-section">` with primary
/// and secondary route groups and a link to the full install guide.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstallSection {
    /// `id` attribute on the wrapping `<section>`.
    pub id: String,
    /// Section heading (`<h2>`).
    pub heading: String,
    /// Intro paragraph text.
    pub intro: String,
    /// Link to the full install guide.
    pub guide_href: String,
    /// Full guide link text; defaults to "Open full install guide".
    pub guide_label: Option<String>,
    /// Platforms this section claims to support. When set, every route must
    /// name its platforms and every claimed platform needs a route.
    pub platforms: Vec<Platform>,
    /// Primary (featured) install routes.
    pub primary_routes: Vec<InstallRoute>,
    /// Secondary (alternative) install routes.
    pub secondary_routes: Vec<InstallRoute>,
}

/// A single install route card within an [`InstallSection`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstallRoute {
    /// Display name for this route.
    pub label: String,
    /// User segment or platform label.
    pub audience: String,
    /// Optional shell command displayed in a copyable block.
    pub command: Option<String>,
    /// Link target for this route.
    pub href: String,
    /// Optional text under the route title.
    pub note: Option<String>,
    /// Route link text; defaults to "Open guide".
    pub link_label: Option<String>,
    /// When `true`, the card shows a "Recommended" badge.
    pub recommended: bool,
    /// Platforms this route installs on; empty means untagged.
    pub platforms: Vec<Platform>,
}

impl InstallRoute {
    /// Create a new install route with the given label, audience, and href.
    ///
    /// `command` and `recommended` default to `None`/`false`;
    /// use [`Self::command`] and [`Self::recommended`] to set them.
    #[must_use]
    pub fn new(
        label: impl Into<String>,
        audience: impl Into<String>,
        href: impl Into<String>,
    ) -> Self {
        Self {
            label: label.into(),
            audience: audience.into(),
            command: None,
            href: href.into(),
            note: None,
            link_label: None,
            recommended: false,
            platforms: Vec::new(),
        }
    }

    /// Set the copyable shell command for this route (builder style).
    #[must_use]
    pub fn command(mut self, command: impl Into<String>) -> Self {
        self.command = Some(command.into());
        self
    }

    /// Set the platforms this route installs on (builder style).
    #[must_use]
    pub fn platforms(mut self, platforms: impl Into<Vec<Platform>>) -> Self {
        self.platforms = platforms.into();
        self
    }

    /// Mark this route as recommended (builder style).
    #[must_use]
    pub fn recommended(mut self) -> Self {
        self.recommended = true;
        self
    }
}
