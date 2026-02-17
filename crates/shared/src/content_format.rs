use serde::{Deserialize, Serialize};

/// The format of a blog post's source content.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ContentFormat {
    #[default]
    Markdown,
    Typst,
}

impl ContentFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentFormat::Markdown => "markdown",
            ContentFormat::Typst => "typst",
        }
    }
}

impl std::fmt::Display for ContentFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_markdown() {
        assert_eq!(ContentFormat::default(), ContentFormat::Markdown);
    }

    #[test]
    fn test_serde_roundtrip() {
        let md = ContentFormat::Markdown;
        let json = serde_json::to_string(&md).unwrap();
        assert_eq!(json, r#""markdown""#);
        let back: ContentFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ContentFormat::Markdown);

        let typst = ContentFormat::Typst;
        let json = serde_json::to_string(&typst).unwrap();
        assert_eq!(json, r#""typst""#);
        let back: ContentFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ContentFormat::Typst);
    }

    #[test]
    fn test_display() {
        assert_eq!(ContentFormat::Markdown.to_string(), "markdown");
        assert_eq!(ContentFormat::Typst.to_string(), "typst");
    }
}
