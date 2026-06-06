#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityMatrix {
    pub id: String,
    pub heading: String,
    pub intro_html: String,
    pub capabilities: Vec<Capability>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Capability {
    pub slug: String,
    pub display_name: String,
    pub overall: String,
    pub details: Vec<(String, String)>,
}
