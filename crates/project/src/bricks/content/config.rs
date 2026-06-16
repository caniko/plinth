use super::ContentSection;

pub fn build_content(id: String, heading: Option<String>, html: String) -> ContentSection {
    ContentSection { id, heading, html }
}
