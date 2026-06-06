#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeatureGrid {
    pub id: Option<String>,
    pub features: Vec<Feature>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Feature {
    pub title: String,
    pub description: String,
    pub highlight: bool,
}

impl Feature {
    #[must_use]
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            highlight: false,
        }
    }

    #[must_use]
    pub fn highlight(mut self) -> Self {
        self.highlight = true;
        self
    }
}
