use plinth_person::PersonReference;

use super::PersonMention;

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
