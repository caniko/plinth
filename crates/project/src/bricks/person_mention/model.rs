use plinth_person::PersonReference;

/// Model for a person-mention card section.
///
/// Rendered as `<section class="person-mention">` with an
/// avatar, name, role, and external links from a [`PersonReference`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersonMention {
    /// Optional `id` on the wrapping `<section>`.
    pub id: Option<String>,
    /// Section heading (`<h2>`).
    pub heading: String,
    /// Intro paragraph text.
    pub intro: String,
    /// The person to reference — provides name, avatar, role, links.
    pub person: PersonReference,
}
