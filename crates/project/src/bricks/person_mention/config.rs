use plinth_person::PersonReference;

use super::PersonMention;

/// Build a [`PersonMention`] model from config values.
///
/// Template markers: `<section class="person-mention">`,
/// `<article class="person-card">`, `<img class="person-avatar">`.
pub fn build_person_mention(
    id: Option<String>,
    heading: String,
    intro: String,
    person: PersonReference,
) -> PersonMention {
    PersonMention {
        id,
        heading,
        intro,
        person,
    }
}
