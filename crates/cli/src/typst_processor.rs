use anyhow::{Context, Result};
use serde::Deserialize;
use typst_as_lib::TypstEngine;
use typst_html::HtmlDocument;

/// The blog template providing #blog-image(), #hero-image(), etc.
const BLOG_TEMPLATE: &str = include_str!("../templates/blog.typ");

/// YAML frontmatter extracted from Typst comments.
#[derive(Debug, Default, Deserialize)]
pub struct TypstFrontmatter {
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub author: Option<String>,
    pub published: Option<bool>,
    pub featured: Option<bool>,
}

/// Extract YAML frontmatter from Typst comments.
///
/// Looks for a block starting with `// ---` and ending with `// ---`,
/// strips the `// ` prefix from each line, then parses as YAML.
///
/// Example:
/// ```typst
/// // ---
/// // title: My Post
/// // tags: ["rust", "typst"]
/// // ---
/// ```
pub fn extract_typst_frontmatter(content: &str) -> Result<Option<TypstFrontmatter>> {
    let lines: Vec<&str> = content.lines().collect();

    // Find the opening `// ---`
    let start = lines.iter().position(|line| line.trim() == "// ---");

    let start = match start {
        Some(s) => s,
        None => return Ok(None),
    };

    // Find the closing `// ---` after the opening
    let end = lines[start + 1..]
        .iter()
        .position(|line| line.trim() == "// ---")
        .map(|i| i + start + 1);

    let end = match end {
        Some(e) => e,
        None => return Ok(None),
    };

    // Extract the YAML content, stripping `// ` prefix
    let yaml_content: String = lines[start + 1..end]
        .iter()
        .map(|line| {
            let trimmed = line.trim();
            if let Some(stripped) = trimmed.strip_prefix("// ") {
                stripped
            } else if trimmed == "//" {
                ""
            } else {
                trimmed
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    let fm: TypstFrontmatter =
        serde_yaml::from_str(&yaml_content).context("Failed to parse Typst frontmatter YAML")?;

    Ok(Some(fm))
}

/// Strip the frontmatter comment block from Typst source.
pub fn strip_typst_frontmatter(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();

    let start = lines.iter().position(|line| line.trim() == "// ---");

    let start = match start {
        Some(s) => s,
        None => return content.to_string(),
    };

    let end = lines[start + 1..]
        .iter()
        .position(|line| line.trim() == "// ---")
        .map(|i| i + start + 1);

    let end = match end {
        Some(e) => e,
        None => return content.to_string(),
    };

    // Remove lines start..=end and rejoin
    let mut result_lines: Vec<&str> = Vec::new();
    result_lines.extend_from_slice(&lines[..start]);
    result_lines.extend_from_slice(&lines[end + 1..]);

    // Trim leading empty lines
    let result = result_lines.join("\n");
    result.trim_start_matches('\n').to_string()
}

/// Compile a Typst document to HTML.
///
/// The blog template is automatically prepended so authors can use
/// `#blog-image()`, `#hero-image()`, and `#gallery()` functions.
pub fn compile_typst_to_html(content: &str) -> Result<String> {
    // Prepend blog template import
    let full_source = format!("{}\n\n{}", BLOG_TEMPLATE, content);

    let engine = TypstEngine::builder().main_file(full_source).build();

    let result = engine.compile::<HtmlDocument>();

    // Check for warnings
    for warning in &result.warnings {
        crate::ui::warn(&format!("Typst: {:?}", warning));
    }

    let doc = result
        .output
        .map_err(|e| anyhow::anyhow!("Typst compilation failed: {:?}", e))?;

    let html = typst_html::html(&doc)
        .map_err(|e| anyhow::anyhow!("Typst HTML generation failed: {:?}", e))?;

    Ok(html)
}

/// Extract plain text from Typst content for embedding generation.
///
/// Strips Typst markup, keeping only text content. This is a simple
/// heuristic that removes common Typst syntax.
pub fn extract_text_for_embedding(content: &str) -> String {
    let stripped = strip_typst_frontmatter(content);

    stripped
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            // Skip pure markup lines
            !trimmed.starts_with("#import")
                && !trimmed.starts_with("#let ")
                && !trimmed.starts_with("#set ")
                && !trimmed.starts_with("#show ")
                && !trimmed.starts_with("#blog-image")
                && !trimmed.starts_with("#hero-image")
                && !trimmed.starts_with("#gallery")
        })
        .map(|line| {
            // Strip inline markup: *bold*, _italic_, `code`, heading markers
            let mut s = line.to_string();
            // Remove heading markers (= , == , etc.)
            if let Some(rest) = s.strip_prefix("= ") {
                s = rest.to_string();
            } else if let Some(rest) = s.strip_prefix("== ") {
                s = rest.to_string();
            } else if let Some(rest) = s.strip_prefix("=== ") {
                s = rest.to_string();
            }
            s
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_frontmatter() {
        let content = r#"// ---
// title: My Post
// tags: ["rust", "typst"]
// description: A test post
// author: Can
// published: true
// featured: false
// ---

= My Post

Some content here."#;

        let fm = extract_typst_frontmatter(content).unwrap().unwrap();
        assert_eq!(fm.title.as_deref(), Some("My Post"));
        assert_eq!(fm.tags, Some(vec!["rust".to_string(), "typst".to_string()]));
        assert_eq!(fm.description.as_deref(), Some("A test post"));
        assert_eq!(fm.author.as_deref(), Some("Can"));
        assert_eq!(fm.published, Some(true));
        assert_eq!(fm.featured, Some(false));
    }

    #[test]
    fn test_extract_frontmatter_no_frontmatter() {
        let content = "= Just a heading\n\nSome content.";
        let fm = extract_typst_frontmatter(content).unwrap();
        assert!(fm.is_none());
    }

    #[test]
    fn test_extract_frontmatter_incomplete() {
        let content = "// ---\n// title: Test\nNo closing delimiter";
        let fm = extract_typst_frontmatter(content).unwrap();
        assert!(fm.is_none());
    }

    #[test]
    fn test_strip_frontmatter() {
        let content = "// ---\n// title: Test\n// ---\n\n= Content\n\nHello!";
        let stripped = strip_typst_frontmatter(content);
        assert!(stripped.starts_with("= Content"));
        assert!(!stripped.contains("title: Test"));
    }

    #[test]
    fn test_strip_frontmatter_no_frontmatter() {
        let content = "= Just content\n\nHello world.";
        let stripped = strip_typst_frontmatter(content);
        assert_eq!(stripped, content);
    }

    #[test]
    fn test_extract_text_for_embedding() {
        let content = r#"// ---
// title: Test
// ---

#import "something": *

= My Heading

Some paragraph text here.

#blog-image("photo.jpg", alt: "Test")

More text content.

#let x = 5"#;

        let text = extract_text_for_embedding(content);
        assert!(text.contains("My Heading"));
        assert!(text.contains("Some paragraph text here."));
        assert!(text.contains("More text content."));
        assert!(!text.contains("title: Test"));
        assert!(!text.contains("#import"));
        assert!(!text.contains("#blog-image"));
        assert!(!text.contains("#let x"));
    }
}
