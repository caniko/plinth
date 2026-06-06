#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustPanel {
    pub id: Option<String>,
    pub heading: String,
    pub intro: String,
    pub items: Vec<TrustItem>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustItem {
    pub title: String,
    pub description: String,
}
