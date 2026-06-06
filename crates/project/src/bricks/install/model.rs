#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstallSection {
    pub id: String,
    pub heading: String,
    pub intro: String,
    pub guide_href: String,
    pub primary_routes: Vec<InstallRoute>,
    pub secondary_routes: Vec<InstallRoute>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstallRoute {
    pub label: String,
    pub audience: String,
    pub command: Option<String>,
    pub href: String,
    pub recommended: bool,
}

impl InstallRoute {
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
            recommended: false,
        }
    }

    #[must_use]
    pub fn command(mut self, command: impl Into<String>) -> Self {
        self.command = Some(command.into());
        self
    }

    #[must_use]
    pub fn recommended(mut self) -> Self {
        self.recommended = true;
        self
    }
}
