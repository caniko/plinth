use anyhow::{Context, Result};
use regex::Regex;
use std::path::{Path, PathBuf};

/// Represents a scanned article with metadata.
#[derive(Debug, Clone)]
pub struct Article {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub content: String,
    pub path: PathBuf,
    images: Vec<ImageRef>,
}

/// Represents an image reference found or to be inserted.
#[derive(Debug, Clone)]
pub struct ImageRef {
    pub url: String,
    pub placement: Placement,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Placement {
    Hero,
    Inline,
    Gallery,
}

impl Article {
    pub fn image_count(&self) -> usize {
        self.images.len()
    }

    pub fn has_images(&self) -> bool {
        !self.images.is_empty()
    }
}

/// Scan a directory for article markdown files (with YAML frontmatter).
pub fn scan_articles(dir: &Path) -> Result<Vec<Article>> {
    let mut articles = Vec::new();

    for entry in std::fs::read_dir(dir).context("Failed to read articles directory")? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        let slug = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {path:?}"))?;

        let (frontmatter, body_content) = parse_frontmatter(&content);
        let images = scan_images(&body_content);

        let title = frontmatter
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or(&slug)
            .to_string();

        let description = frontmatter
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let tags = frontmatter
            .get("tags")
            .and_then(|v| v.as_sequence())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        articles.push(Article {
            slug,
            title,
            description,
            tags,
            content: body_content,
            path,
            images,
        });
    }

    // Sort by slug for consistent ordering
    articles.sort_by(|a, b| a.slug.cmp(&b.slug));
    Ok(articles)
}

fn parse_frontmatter(content: &str) -> (serde_yaml::Value, String) {
    let matter = gray_matter::Matter::new();
    match matter.parse(content) {
        Ok(parsed) => {
            let front = parsed.data.unwrap_or(serde_yaml::Value::Mapping(Default::default()));
            let body = parsed.content;
            (front, body)
        }
        Err(_) => {
            (serde_yaml::Value::Mapping(Default::default()), content.to_string())
        }
    }
}

fn scan_images(_body: &str) -> Vec<ImageRef> {
    let re = Regex::new(r"!\[([^\]]*)\]\((/api/images/[^)]+)\)").unwrap();
    re.captures_iter(_body)
        .map(|cap| {
            let url = cap[2].to_string();
            ImageRef {
                url,
                placement: Placement::Hero,
            }
        })
        .collect()
}

/// Add a hero image reference to an article's markdown content.
pub fn add_hero_image(content: &str, image_url: &str) -> String {
    let hero_line = format!("\n![Hero]({image_url})\n");

    // Find the end of frontmatter
    if let Some(end) = content.find("\n---\n").map(|i| i + 5) {
        // Check if there's a # Title after frontmatter
        let after_fm = &content[end..];
        if let Some(title_end) = after_fm.find('\n') {
            let title_line_end = end + title_end + 1;
            // Insert after the title line
            let mut result = content[..=title_line_end].to_string();
            result.push_str(&hero_line);
            result.push_str(&content[title_line_end + 1..]);
            return result;
        }
        // No title found, insert after frontmatter
        let mut result = content[..end].to_string();
        result.push_str(&hero_line);
        result.push_str(&content[end..]);
        return result;
    }

    // No frontmatter, insert at top
    format!("{hero_line}{content}")
}

/// Write a hero image reference into the article file.
pub fn write_hero_image(path: &Path, image_url: &str) -> Result<()> {
    let content = std::fs::read_to_string(path)?;
    let updated = add_hero_image(&content, image_url);
    std::fs::write(path, &updated)?;
    Ok(())
}

/// Run the format subcommand: write a hero image reference into an article.
pub fn run(slug: &str, image_url: &str, placement: &str) -> Result<()> {
    if placement != "hero" {
        anyhow::bail!("Only hero placement is currently supported, got: {placement}");
    }
    let cwd = std::env::current_dir()?;
    let article_path = cwd.join(format!("{slug}.md"));
    if !article_path.exists() {
        anyhow::bail!("Article not found: {}", article_path.display());
    }
    write_hero_image(&article_path, image_url)
}

/// Determine the topic cluster from the tags.
pub fn detect_cluster(tags: &[String]) -> &'static str {
    let tag_set: Vec<&str> = tags.iter().map(|t| t.as_str()).collect();

    if tag_set.contains(&"gaming") {
        return "gaming";
    }
    if tag_set.contains(&"ai") && !tag_set.contains(&"infrastructure") {
        return "ai_neural";
    }
    if tag_set.contains(&"infrastructure") || tag_set.contains(&"devops") {
        return "nix_infrastructure";
    }
    if tag_set.contains(&"publishing") || tag_set.contains(&"self-hosting") {
        return "publishing";
    }
    if tag_set.contains(&"rust") {
        return "rust_tools";
    }
    "rust_tools"
}

/// Get a base prompt for the cluster, with article-specific substitutions.
pub fn prompt_for_article(article: &Article) -> String {
    let cluster = detect_cluster(&article.tags);
    let concept = &article.description;
    let subject = &article.title;

    match cluster {
        "nix_infrastructure" => format!(
            "Geometric abstract architecture diagram representing \"{}\": {}, \
             clean technical blueprint lines, slate and teal, no text, no typography",
            subject, concept
        ),
        "rust_tools" => format!(
            "Dark tech aesthetic representing \"{}\": {}, \
             glowing code elements, terminal framing, deep purple and cyan, no text",
            subject, concept
        ),
        "publishing" => format!(
            "Warm minimal editorial aesthetic representing \"{}\": {}, \
             paper texture, warm cream and teal, serif typography ambiance, no text",
            subject, concept
        ),
        "ai_neural" => format!(
            "Abstract neural network topology representing \"{}\": {}, \
             glowing connection nodes, gradient purple and gold, knowledge graph aesthetic, no text",
            subject, concept
        ),
        "gaming" => format!(
            "Cyberpunk gaming setup representing \"{}\": {}, \
             controller silhouette, virtual overlay, neon blue and pink, no text",
            subject, concept
        ),
        _ => format!(
            "Clean modern tech illustration representing \"{}\": {}, \
             geometric shapes, warm tones, no text",
            subject, concept
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_hero_image_with_frontmatter() {
        let content = r#"---
title: Test
tags: ["rust"]
---

# My Article

Some content.
"#;
        let result = add_hero_image(content, "/api/images/abc-123?w=1920&h=1080");
        assert!(result.contains("![Hero](/api/images/abc-123?w=1920&h=1080)"));
        assert!(result.contains("# My Article"));
        assert!(result.contains("Some content."));
    }

    #[test]
    fn test_detect_cluster_infrastructure() {
        let tags = vec!["canix-toolbelt".into(), "nix".into(), "infrastructure".into()];
        assert_eq!(detect_cluster(&tags), "nix_infrastructure");
    }

    #[test]
    fn test_detect_cluster_rust() {
        let tags = vec!["simit".into(), "rust".into(), "cli".into()];
        assert_eq!(detect_cluster(&tags), "rust_tools");
    }

    #[test]
    fn test_detect_cluster_ai() {
        let tags = vec!["skillnet".into(), "rust".into(), "ai".into()];
        assert_eq!(detect_cluster(&tags), "ai_neural");
    }

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
title: Hello
tags: ["a", "b"]
---

Body text
"#;
        let (fm, body) = parse_frontmatter(content);
        assert_eq!(fm["title"].as_str(), Some("Hello"));
        assert_eq!(body.trim(), "Body text");
    }
}
