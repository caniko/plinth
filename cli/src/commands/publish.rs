use anyhow::{Context, Result};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use std::fs;
use std::path::Path;
use shared::PublishArticleRequest;

use crate::api_client::ApiClient;

/// Publish an article from a markdown file
pub async fn publish_article(file_path: &str, api_client: &ApiClient) -> Result<()> {
    println!("📝 Publishing article from: {}", file_path);
    println!();

    // Read markdown file
    let path = Path::new(file_path);
    if !path.exists() {
        anyhow::bail!("File not found: {}", file_path);
    }

    let content = fs::read_to_string(path)
        .context(format!("Failed to read file: {}", file_path))?;

    println!("✅ Read {} bytes from file", content.len());

    // Generate vector embedding
    println!("🔢 Generating vector embedding...");
    let embedding = generate_embedding(&content).await?;
    println!("✅ Generated embedding with {} dimensions", embedding.len());

    // Create publish request
    let request = PublishArticleRequest {
        title: None, // Will be extracted from frontmatter or filename
        content,
        slug: None, // Will be auto-generated
        description: None,
        author: None, // Will use default
        tags: None,  // Will be extracted from frontmatter
        featured: None,
        published: None, // Default to true
        embedding: Some(embedding),
    };

    // Send to API
    println!("🚀 Sending to API...");
    let response = api_client.publish_article(request).await?;

    // Display result
    println!();
    println!("✅ Success!");
    println!("   Slug: {}", response.slug);
    if let Some(id) = response.id {
        println!("   ID: {}", id);
    }
    println!("   Message: {}", response.message);

    Ok(())
}

/// Generate a vector embedding for the given content using fastembed
async fn generate_embedding(content: &str) -> Result<Vec<f32>> {
    // Initialize the embedding model (using the default model)
    let model = TextEmbedding::try_new(InitOptions {
        model_name: EmbeddingModel::AllMiniLML6V2, // 384 dimensions
        show_download_progress: false,
        ..Default::default()
    })?;

    // Strip frontmatter for embedding generation (only embed the actual content)
    let content_for_embedding = strip_frontmatter(content);

    // Truncate content if too long (model has token limits)
    let truncated = if content_for_embedding.len() > 5000 {
        &content_for_embedding[..5000]
    } else {
        &content_for_embedding
    };

    // Generate embeddings (returns Vec<Vec<f32>> for batch)
    let embeddings = model.embed(vec![truncated.to_string()], None)?;

    // Return the first (and only) embedding
    embeddings
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Failed to generate embedding"))
}

/// Strip frontmatter from markdown content
fn strip_frontmatter(content: &str) -> String {
    if content.starts_with("---") {
        // Find the second "---" delimiter
        if let Some(end) = content[3..].find("---") {
            // Return content after the second delimiter
            return content[end + 6..].trim().to_string();
        }
    }
    content.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_frontmatter() {
        let with_frontmatter = r#"---
title: Test
tags: ["rust"]
---

# Content

Hello world!"#;

        let stripped = strip_frontmatter(with_frontmatter);
        assert!(stripped.starts_with("# Content"));
        assert!(!stripped.contains("title: Test"));
    }

    #[test]
    fn test_strip_frontmatter_no_frontmatter() {
        let without = "# Just Content\n\nNo frontmatter.";
        let stripped = strip_frontmatter(without);
        assert_eq!(stripped, without);
    }
}
