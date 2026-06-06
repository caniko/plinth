#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AudienceGrid {
    pub id: Option<String>,
    pub heading: String,
    pub intro: String,
    pub audiences: Vec<Audience>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Audience {
    pub label: String,
    pub description: String,
}
