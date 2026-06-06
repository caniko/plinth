use plinth_person::PersonReference;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersonMention {
    pub id: Option<String>,
    pub heading: String,
    pub intro: String,
    pub person: PersonReference,
}
