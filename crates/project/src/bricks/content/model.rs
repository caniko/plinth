#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentSection {
    pub id: String,
    pub heading: Option<String>,
    pub html: String,
}
