//! The post draft, its validation, and slug generation.

use serde::{Deserialize, Serialize};

/// Longest accepted slug, in characters.
pub const MAX_SLUG_LEN: usize = 80;
/// Longest accepted description, in characters.
pub const MAX_DESCRIPTION_LEN: usize = 160;

/// An in-progress post.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Draft {
    pub title: String,
    pub slug: String,
    pub description: String,
    pub tags: Vec<String>,
    pub body: String,
    /// `true` publishes immediately; `false` saves a draft post.
    pub published: bool,
    /// The site draft this replaces when sent; set when a draft is opened
    /// from the site's post list.
    pub replaces: Option<Replaces>,
}

/// A site draft as it was when opened. Sending replaces it only if it is
/// still exactly that draft, so a stale editor never overwrites newer work.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Replaces {
    /// The draft's slug when opened.
    pub slug: String,
    /// Personal sites: the server's draft id, which changes on every replace.
    /// Project sites: the file's full text.
    pub version: String,
}

impl Default for Draft {
    /// An empty post that publishes publicly.
    fn default() -> Self {
        Self {
            title: String::new(),
            slug: String::new(),
            description: String::new(),
            tags: Vec::new(),
            body: String::new(),
            published: true,
            replaces: None,
        }
    }
}

/// A form field the workspace can report an error against.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Field {
    Title,
    Slug,
    Description,
    Tags,
    Body,
}

impl Field {
    /// Stable DOM id suffix, also used for `aria-describedby` wiring.
    pub fn id(self) -> &'static str {
        match self {
            Field::Title => "title",
            Field::Slug => "slug",
            Field::Description => "description",
            Field::Tags => "tags",
            Field::Body => "body",
        }
    }
}

/// One validation problem tied to a field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldError {
    pub field: Field,
    pub message: String,
}

impl FieldError {
    pub fn new(field: Field, message: impl Into<String>) -> Self {
        Self {
            field,
            message: message.into(),
        }
    }
}

impl Draft {
    /// Trimmed and tag-normalised copy, suitable for publishing.
    pub fn normalized(&self) -> Draft {
        Draft {
            title: self.title.trim().to_string(),
            slug: self.slug.trim().to_string(),
            description: self.description.trim().to_string(),
            tags: parse_tags(&self.tags.join(",")),
            body: self.body.clone(),
            published: self.published,
            replaces: self.replaces.clone(),
        }
    }

    /// Validate the draft for publishing. Returns every problem at once.
    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut errors = Vec::new();

        if self.title.trim().is_empty() {
            errors.push(FieldError::new(Field::Title, "Add a title."));
        }

        let slug = self.slug.trim();
        if slug.is_empty() {
            errors.push(FieldError::new(Field::Slug, "Add a slug."));
        } else if slug.chars().count() > MAX_SLUG_LEN {
            errors.push(FieldError::new(
                Field::Slug,
                format!("Keep the slug to {MAX_SLUG_LEN} characters or fewer."),
            ));
        } else if !is_valid_slug(slug) {
            errors.push(FieldError::new(
                Field::Slug,
                "Use lowercase letters, digits and single dashes, like my-post.",
            ));
        }

        if self.body.trim().is_empty() {
            errors.push(FieldError::new(Field::Body, "Write something first."));
        }

        let description_len = self.description.trim().chars().count();
        if description_len > MAX_DESCRIPTION_LEN {
            errors.push(FieldError::new(
                Field::Description,
                format!(
                    "Shorten the description to {MAX_DESCRIPTION_LEN} characters or fewer (currently {description_len})."
                ),
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Turn a title into a URL-safe slug (lowercase ASCII alphanumerics and dashes).
pub fn slugify(title: &str) -> String {
    let mut slug = String::with_capacity(title.len());
    for character in title.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
    }
    let slug = slug.trim_matches('-');
    let mut truncated: String = slug.chars().take(MAX_SLUG_LEN).collect();
    // Slugs are ASCII, so byte indexing is safe. A cut inside a word drops the
    // fragment rather than leaving "…-taught-us-a" in a permanent URL.
    if slug.len() > truncated.len()
        && slug.as_bytes()[truncated.len()] != b'-'
        && let Some(cut) = truncated.rfind('-')
    {
        truncated.truncate(cut);
    }
    while truncated.ends_with('-') {
        truncated.pop();
    }
    truncated
}

/// Split a comma-separated tag field into trimmed, lowercased, de-duplicated tags.
pub(crate) fn parse_tags(input: &str) -> Vec<String> {
    let mut tags: Vec<String> = Vec::new();
    for raw in input.split(',') {
        let tag = raw.trim().to_lowercase();
        if tag.is_empty() || tags.contains(&tag) {
            continue;
        }
        tags.push(tag);
    }
    tags
}

fn is_valid_slug(slug: &str) -> bool {
    if slug.starts_with('-') || slug.ends_with('-') {
        return false;
    }
    slug.split('-').all(|part| {
        !part.is_empty()
            && part
                .chars()
                .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft(title: &str, slug: &str, body: &str) -> Draft {
        Draft {
            title: title.to_string(),
            slug: slug.to_string(),
            body: body.to_string(),
            ..Draft::default()
        }
    }

    #[test]
    fn slugify_lowercases_and_collapses_separators() {
        assert_eq!(slugify("Hello, World!"), "hello-world");
        assert_eq!(slugify("  --Rust & Dioxus--  "), "rust-dioxus");
        assert_eq!(slugify("Café release"), "caf-release");
        assert_eq!(slugify(""), "");
        assert_eq!(slugify("!!!"), "");
        assert_eq!(slugify("already-a-slug"), "already-a-slug");
        let long = slugify(&"a".repeat(100));
        assert_eq!(long.chars().count(), MAX_SLUG_LEN);
        // A cut inside a word drops the fragment.
        let words = slugify(&"word ".repeat(30));
        assert!(words.ends_with("-word") && words.chars().count() <= MAX_SLUG_LEN);
    }

    #[test]
    fn validation_accepts_a_complete_draft() {
        assert!(draft("Title", "a-valid-slug-2", "body").validate().is_ok());
    }

    #[test]
    fn validation_reports_every_problem() {
        let broken = Draft {
            title: "  ".to_string(),
            slug: "Not A Slug".to_string(),
            description: "x".repeat(MAX_DESCRIPTION_LEN + 1),
            body: "\n".to_string(),
            ..Draft::default()
        };
        let errors = broken.validate().unwrap_err();
        let fields: Vec<Field> = errors.iter().map(|error| error.field).collect();
        assert_eq!(
            fields,
            vec![Field::Title, Field::Slug, Field::Body, Field::Description]
        );
    }

    #[test]
    fn validation_rejects_double_and_edge_dashes_and_long_slugs() {
        assert!(draft("T", "a--b", "body").validate().is_err());
        assert!(draft("T", "-ab", "body").validate().is_err());
        assert!(draft("T", "ab-", "body").validate().is_err());
        assert!(draft("T", "UPPER", "body").validate().is_err());
        assert!(
            draft("T", &"a".repeat(MAX_SLUG_LEN + 1), "body")
                .validate()
                .is_err()
        );
        assert!(
            draft("T", &"a".repeat(MAX_SLUG_LEN), "body")
                .validate()
                .is_ok()
        );
    }

    #[test]
    fn normalize_trims_fields_and_tags() {
        let messy = Draft {
            title: "  Hello  ".to_string(),
            slug: "  hello  ".to_string(),
            description: "  summary ".to_string(),
            tags: vec![" Rust ".to_string(), "rust".to_string(), "".to_string()],
            body: "body ".to_string(),
            published: true,
            replaces: None,
        };
        let clean = messy.normalized();
        assert_eq!(clean.title, "Hello");
        assert_eq!(clean.slug, "hello");
        assert_eq!(clean.description, "summary");
        assert_eq!(clean.tags, vec!["rust".to_string()]);
        assert_eq!(clean.body, "body ");
        assert!(clean.published);
    }

    #[test]
    fn parse_tags_lowercases_and_dedupes() {
        assert_eq!(
            parse_tags(" Rust, release ,, RUST, typst "),
            vec![
                "rust".to_string(),
                "release".to_string(),
                "typst".to_string()
            ]
        );
    }
}
