#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComparisonSection {
    pub id: Option<String>,
    pub heading: String,
    pub intro: String,
    pub rows: Vec<ComparisonRow>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComparisonRow {
    pub area: String,
    pub status: String,
    pub notes: String,
}
