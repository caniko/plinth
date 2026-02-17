use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use fastembed::{EmbeddingModel, TextEmbedding};
use plinth_shared::{ContentFormat, PublishArticleRequest};

use crate::api_client::ApiClient;
use crate::image_scanner;
use crate::immich_client::ImmichClient;
use crate::typst_processor;
use crate::ui;

/// Publish an article from a markdown or typst file
pub async fn publish_article(
    file_path: &str,
    api_client: &ApiClient,
    immich_client: Option<&ImmichClient>,
) -> Result<()> {
    let path = Path::new(file_path);
    if !path.exists() {
        anyhow::bail!("File not found: {}", file_path);
    }

    let content =
        std::fs::read_to_string(path).context(format!("Failed to read file: {}", file_path))?;

    // Detect format from extension
    let format = match path.extension().and_then(|e| e.to_str()) {
        Some("typ") => ContentFormat::Typst,
        _ => ContentFormat::Markdown,
    };

    ui::status("Read", &format!("{} ({} bytes)", file_path, content.len()));
    ui::status("Format", format.as_str());

    match format {
        ContentFormat::Markdown => publish_markdown(content, api_client).await,
        ContentFormat::Typst => publish_typst(content, path, api_client, immich_client).await,
    }
}

/// Publish a markdown article (existing flow).
async fn publish_markdown(content: String, api_client: &ApiClient) -> Result<()> {
    // Generate vector embedding
    let sp = ui::spinner("Generating vector embedding...");
    let text_for_embedding = strip_frontmatter(&content);
    let embedding = generate_embedding(&text_for_embedding).await?;
    sp.finish_and_clear();
    ui::status("Embed", &format!("{} dimensions", embedding.len()));

    let request = PublishArticleRequest {
        title: None,
        content,
        slug: None,
        description: None,
        author: None,
        tags: None,
        featured: None,
        published: None,
        embedding: Some(embedding),
        content_format: None, // Default: Markdown
        html_content: None,   // Server renders markdown
    };

    send_request(request, api_client).await
}

/// Publish a Typst article with image upload and HTML compilation.
async fn publish_typst(
    content: String,
    file_path: &Path,
    api_client: &ApiClient,
    immich_client: Option<&ImmichClient>,
) -> Result<()> {
    // 1. Extract frontmatter metadata
    let frontmatter = typst_processor::extract_typst_frontmatter(&content)?;
    let stripped = typst_processor::strip_typst_frontmatter(&content);

    // 2. Scan for local image references and upload to Immich
    let refs = image_scanner::scan_image_references(&stripped);
    let mut image_mapping: HashMap<String, String> = HashMap::new();

    if !refs.is_empty() {
        let immich = immich_client.ok_or_else(|| {
            anyhow::anyhow!(
                "Typst file references {} local image(s), but Immich is not configured.\n\
                 Set IMMICH_API_URL and IMMICH_API_KEY to enable image uploads.",
                refs.len()
            )
        })?;

        let base_dir = file_path.parent().unwrap_or(Path::new("."));

        for img_ref in &refs {
            let image_path = base_dir.join(&img_ref.src);
            if !image_path.exists() {
                anyhow::bail!(
                    "Image file not found: {} (resolved to {})",
                    img_ref.src,
                    image_path.display()
                );
            }

            let sp = ui::spinner(&format!("Uploading image: {}", img_ref.src));
            let asset_id = immich.upload_asset(&image_path).await?;
            sp.finish_and_clear();
            ui::status(
                "Upload",
                &format!("{} -> {}", img_ref.src, &asset_id[..8.min(asset_id.len())]),
            );
            image_mapping.insert(img_ref.src.clone(), asset_id);
        }
    }

    // 3. Replace local paths with proxy URLs
    let resolved_content = if image_mapping.is_empty() {
        stripped
    } else {
        image_scanner::replace_image_references(&stripped, &image_mapping)
    };

    // 4. Compile Typst to HTML
    let sp = ui::spinner("Compiling Typst to HTML...");
    let html = typst_processor::compile_typst_to_html(&resolved_content)?;
    sp.finish_and_clear();
    ui::status("Typst", &format!("HTML generated ({} bytes)", html.len()));

    // 5. Extract text and generate embedding
    let sp = ui::spinner("Generating vector embedding...");
    let text_for_embedding = typst_processor::extract_text_for_embedding(&content);
    let embedding = generate_embedding(&text_for_embedding).await?;
    sp.finish_and_clear();
    ui::status("Embed", &format!("{} dimensions", embedding.len()));

    // 6. Build request with metadata from frontmatter
    let request = PublishArticleRequest {
        title: frontmatter.as_ref().and_then(|fm| fm.title.clone()),
        content,
        slug: None,
        description: frontmatter.as_ref().and_then(|fm| fm.description.clone()),
        author: frontmatter.as_ref().and_then(|fm| fm.author.clone()),
        tags: frontmatter.as_ref().and_then(|fm| fm.tags.clone()),
        featured: frontmatter.as_ref().and_then(|fm| fm.featured),
        published: frontmatter.as_ref().and_then(|fm| fm.published),
        embedding: Some(embedding),
        content_format: Some(ContentFormat::Typst),
        html_content: Some(html),
    };

    send_request(request, api_client).await
}

/// Send the publish request and display the result.
async fn send_request(request: PublishArticleRequest, api_client: &ApiClient) -> Result<()> {
    let sp = ui::spinner("Sending to API...");
    let response = api_client.publish_article(request).await?;
    sp.finish_and_clear();

    ui::success(&format!("Published: {}", response.slug));
    if let Some(id) = response.id {
        ui::detail(&format!("ID: {}", id));
    }
    ui::detail(&format!("Message: {}", response.message));

    Ok(())
}

/// Generate a vector embedding for the given content using fastembed
async fn generate_embedding(content: &str) -> Result<Vec<f32>> {
    let mut init_options = fastembed::TextInitOptions::default();
    init_options.model_name = EmbeddingModel::AllMiniLML6V2;
    init_options.show_download_progress = false;
    let mut model = TextEmbedding::try_new(init_options)?;

    // Truncate content if too long (model has token limits)
    let truncated = if content.len() > 5000 {
        &content[..5000]
    } else {
        content
    };

    let embeddings = model.embed(vec![truncated.to_string()], None)?;

    embeddings
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Failed to generate embedding"))
}

/// Strip YAML frontmatter from markdown content
fn strip_frontmatter(content: &str) -> String {
    if let Some(stripped) = content.strip_prefix("---")
        && let Some(end) = stripped.find("---")
    {
        return stripped[end + 3..].trim().to_string();
    }
    content.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_frontmatter() {
        let with_frontmatter = r##"---
title: Test
tags: ["rust"]
---

# Content

Hello world!"##;

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

    #[test]
    fn test_strip_frontmatter_with_dashes_in_content() {
        let content = "---\ntitle: Test\n---\n\nContent with --- in the middle.";
        let stripped = strip_frontmatter(content);
        assert!(stripped.starts_with("Content with"));
        assert!(stripped.contains("---"));
    }

    #[test]
    fn test_strip_frontmatter_incomplete() {
        let content = "---\ntitle: Test\nNo closing delimiter";
        let stripped = strip_frontmatter(content);
        assert_eq!(stripped, content);
    }

    #[test]
    fn test_strip_frontmatter_empty() {
        let stripped = strip_frontmatter("");
        assert_eq!(stripped, "");
    }
}
