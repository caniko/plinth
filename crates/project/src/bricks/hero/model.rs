#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Cta {
    pub label: String,
    pub href: String,
    pub primary: bool,
}

impl Cta {
    #[must_use]
    pub fn primary(label: impl Into<String>, href: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            href: href.into(),
            primary: true,
        }
    }

    #[must_use]
    pub fn secondary(label: impl Into<String>, href: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            href: href.into(),
            primary: false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Hero {
    pub logo_src: Option<String>,
    pub title: String,
    pub tagline: String,
    pub subtitle: String,
    pub person: Option<String>,
    pub ctas: Vec<Cta>,
}
