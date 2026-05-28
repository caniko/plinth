use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use fastembed::{EmbeddingModel, TextEmbedding};
use plinth_shared::{ContentFormat, PublishArticleRequest};

use crate::api_client::ApiClient;
use crate::image_scanner::{self, ImageMapping};
use crate::immich_client::ImmichClient;
use crate::typst_processor;
use crate::ui;

/// Publish an article from a markdown or typst file
pub async fn publish_article(
    file_path: &str,
    api_client: &ApiClient,
    immich_client: Option<&ImmichClient>,
    skip_images: bool,
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
        ContentFormat::Markdown => {
            publish_markdown(content, path, api_client, immich_client, skip_images).await
        }
        ContentFormat::Typst => {
            publish_typst(content, path, api_client, immich_client, skip_images).await
        }
    }
}

/// Publish a markdown article with optional image upload support.
async fn publish_markdown(
    content: String,
    file_path: &Path,
    api_client: &ApiClient,
    immich_client: Option<&ImmichClient>,
    skip_images: bool,
) -> Result<()> {
    // Scan for local image references and upload to Immich
    let refs = image_scanner::scan_markdown_image_references(&content);
    let mut image_mapping: HashMap<String, ImageMapping> = HashMap::new();

    if !refs.is_empty() {
        let base_dir = file_path.parent().unwrap_or(Path::new("."));

        match immich_client {
            Some(immich) => {
                let canonical_base = base_dir
                    .canonicalize()
                    .context("Failed to resolve base directory")?;
                for img_ref in &refs {
                    let image_path = base_dir.join(&img_ref.src);
                    if !image_path.exists() {
                        anyhow::bail!(
                            "Image file not found: {} (resolved to {})",
                            img_ref.src,
                            image_path.display()
                        );
                    }
                    let canonical_image = image_path
                        .canonicalize()
                        .context("Failed to resolve image path")?;
                    if !canonical_image.starts_with(&canonical_base) {
                        anyhow::bail!("Image path escapes base directory: {}", img_ref.src);
                    }

                    let sp = ui::spinner(&format!("Uploading image: {}", img_ref.src));
                    match immich.upload_asset(&image_path).await {
                        Ok(result) => {
                            sp.finish_and_clear();
                            ui::status(
                                "Upload",
                                &format!(
                                    "{} -> {}",
                                    img_ref.src,
                                    &result.asset_id[..8.min(result.asset_id.len())]
                                ),
                            );
                            image_mapping.insert(
                                img_ref.src.clone(),
                                ImageMapping {
                                    asset_id: result.asset_id,
                                    width: result.width,
                                    height: result.height,
                                },
                            );
                        }
                        Err(e) if skip_images => {
                            sp.finish_and_clear();
                            ui::warn(&format!(
                                "Failed to upload {}, skipping: {}",
                                img_ref.src, e
                            ));
                        }
                        Err(e) => return Err(e),
                    }
                }
            }
            None if skip_images => {
                for img_ref in &refs {
                    ui::warn(&format!(
                        "Skipping image upload: {} (Immich not configured)",
                        img_ref.src
                    ));
                }
            }
            None => {
                anyhow::bail!(
                    "File references {} local image(s), but Immich is not configured.\n\
                     Set IMMICH_API_URL and IMMICH_API_KEY to enable image uploads,\n\
                     or use --skip-images to publish without uploading.",
                    refs.len()
                );
            }
        }
    }

    // Replace local paths with proxy URLs (with dimension query params)
    let resolved_content = if image_mapping.is_empty() {
        content
    } else {
        image_scanner::replace_markdown_image_references(&content, &image_mapping)
    };

    // Generate vector embedding
    let sp = ui::spinner("Generating vector embedding...");
    let text_for_embedding = strip_frontmatter(&resolved_content);
    let embedding = generate_embedding(&text_for_embedding).await?;
    sp.finish_and_clear();
    ui::status("Embed", &format!("{} dimensions", embedding.len()));

    let request = PublishArticleRequest {
        title: None,
        content: resolved_content,
        slug: None,
        description: None,
        author: None,
        tags: None,
        featured: None,
        published: None,
        embedding: Some(embedding),
        content_format: None, // Default: Markdown
        html_content: None,   // Server renders markdown
        series: None,         // Extracted from frontmatter server-side
        series_title: None,
        series_position: None,
    };

    send_request(request, api_client).await
}

/// Publish a Typst article with image upload and HTML compilation.
async fn publish_typst(
    content: String,
    file_path: &Path,
    api_client: &ApiClient,
    immich_client: Option<&ImmichClient>,
    skip_images: bool,
) -> Result<()> {
    // 1. Extract frontmatter metadata
    let frontmatter = typst_processor::extract_typst_frontmatter(&content)?;
    let stripped = typst_processor::strip_typst_frontmatter(&content);

    // 2. Scan for local image references and upload to Immich
    let refs = image_scanner::scan_image_references(&stripped);
    let mut image_mapping: HashMap<String, ImageMapping> = HashMap::new();

    if !refs.is_empty() {
        let base_dir = file_path.parent().unwrap_or(Path::new("."));

        match immich_client {
            Some(immich) => {
                let canonical_base = base_dir
                    .canonicalize()
                    .context("Failed to resolve base directory")?;
                for img_ref in &refs {
                    let image_path = base_dir.join(&img_ref.src);
                    if !image_path.exists() {
                        anyhow::bail!(
                            "Image file not found: {} (resolved to {})",
                            img_ref.src,
                            image_path.display()
                        );
                    }
                    let canonical_image = image_path
                        .canonicalize()
                        .context("Failed to resolve image path")?;
                    if !canonical_image.starts_with(&canonical_base) {
                        anyhow::bail!("Image path escapes base directory: {}", img_ref.src);
                    }

                    let sp = ui::spinner(&format!("Uploading image: {}", img_ref.src));
                    match immich.upload_asset(&image_path).await {
                        Ok(result) => {
                            sp.finish_and_clear();
                            ui::status(
                                "Upload",
                                &format!(
                                    "{} -> {}",
                                    img_ref.src,
                                    &result.asset_id[..8.min(result.asset_id.len())]
                                ),
                            );
                            image_mapping.insert(
                                img_ref.src.clone(),
                                ImageMapping {
                                    asset_id: result.asset_id,
                                    width: result.width,
                                    height: result.height,
                                },
                            );
                        }
                        Err(e) if skip_images => {
                            sp.finish_and_clear();
                            ui::warn(&format!(
                                "Failed to upload {}, skipping: {}",
                                img_ref.src, e
                            ));
                        }
                        Err(e) => return Err(e),
                    }
                }
            }
            None if skip_images => {
                for img_ref in &refs {
                    ui::warn(&format!(
                        "Skipping image upload: {} (Immich not configured)",
                        img_ref.src
                    ));
                }
            }
            None => {
                anyhow::bail!(
                    "File references {} local image(s), but Immich is not configured.\n\
                     Set IMMICH_API_URL and IMMICH_API_KEY to enable image uploads,\n\
                     or use --skip-images to publish without uploading.",
                    refs.len()
                );
            }
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
        series: frontmatter.as_ref().and_then(|fm| fm.series.clone()),
        series_title: frontmatter.as_ref().and_then(|fm| fm.series_title.clone()),
        series_position: frontmatter.as_ref().and_then(|fm| fm.series_position),
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
    // Truncate content if too long (model has token limits), snapping to a
    // char boundary so a multi-byte codepoint at byte 5000 cannot panic.
    let mut end = content.len().min(5000);
    while end > 0 && !content.is_char_boundary(end) {
        end -= 1;
    }
    let truncated = content[..end].to_string();

    // Model init and inference are blocking/CPU-heavy — keep them off the async
    // runtime so they don't stall the executor.
    tokio::task::spawn_blocking(move || -> Result<Vec<f32>> {
        let mut init_options = fastembed::TextInitOptions::default();
        init_options.model_name = EmbeddingModel::AllMiniLML6V2;
        init_options.show_download_progress = false;
        let mut model = TextEmbedding::try_new(init_options)?;

        let embeddings = model.embed(vec![truncated], None)?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("Failed to generate embedding"))
    })
    .await?
}

/// Interactive publish flow — prompts for metadata, opens editor, writes file, optionally publishes.
pub async fn interactive_publish(
    api_client: &ApiClient,
    immich_client: Option<&ImmichClient>,
) -> Result<()> {
    use crate::prompts::{self, ContentSource, TEMPLATE_POST};
    use plinth_shared::BlogPost;

    let title = prompts::prompt_text("Title:", None)?;
    let description = prompts::prompt_optional_text("Description:")?;
    let tags = prompts::prompt_tags("Tags:")?;
    let author = prompts::prompt_optional_text("Author:")?;
    let published = prompts::prompt_bool("Published?", true)?;
    let featured = prompts::prompt_bool("Featured?", false)?;

    // Content body
    let body = match prompts::prompt_content(&[TEMPLATE_POST])? {
        ContentSource::EditorContent(text) => {
            // Strip template frontmatter — we already prompted for metadata
            typst_processor::strip_typst_frontmatter(&text)
        }
        ContentSource::ExistingFile(path) => {
            let p = Path::new(&path);
            if !p.exists() {
                anyhow::bail!("File not found: {}", path);
            }
            let raw = std::fs::read_to_string(p)
                .with_context(|| format!("Failed to read file: {}", path))?;
            typst_processor::strip_typst_frontmatter(&raw)
        }
        ContentSource::Skip => String::new(),
    };

    // Build the .typ file
    let frontmatter = build_typst_frontmatter(
        &title,
        description.as_deref(),
        &tags,
        author.as_deref(),
        published,
        featured,
    );
    let file_content = if body.is_empty() {
        format!("{}\n\n= {}\n", frontmatter, title)
    } else {
        format!("{}\n\n{}", frontmatter, body)
    };

    // Output path
    let default_filename = format!("{}.typ", BlogPost::slugify(&title));
    let output_path = prompts::prompt_text("Output file:", Some(&default_filename))?;

    let dest = Path::new(&output_path);
    if dest.exists() {
        anyhow::bail!("File already exists: {}", output_path);
    }

    if let Some(parent) = dest.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    std::fs::write(dest, &file_content)
        .with_context(|| format!("Failed to write: {}", output_path))?;

    ui::success(&format!("Created {}", output_path));

    // Optionally publish
    if prompts::prompt_bool("Publish now?", false)? {
        publish_article(&output_path, api_client, immich_client, false).await?;
    } else {
        ui::detail(&format!(
            "Run 'plinth publish {}' to publish later",
            output_path
        ));
    }

    Ok(())
}

/// Build a Typst comment-block YAML frontmatter string.
fn build_typst_frontmatter(
    title: &str,
    description: Option<&str>,
    tags: &[String],
    author: Option<&str>,
    published: bool,
    featured: bool,
) -> String {
    let mut lines = vec!["// ---".to_string()];
    lines.push(format!("// title: {}", title));
    if let Some(desc) = description {
        lines.push(format!("// description: {}", desc));
    }
    if !tags.is_empty() {
        let quoted: Vec<String> = tags.iter().map(|t| format!("\"{}\"", t)).collect();
        lines.push(format!("// tags: [{}]", quoted.join(", ")));
    }
    if let Some(a) = author {
        lines.push(format!("// author: {}", a));
    }
    lines.push(format!("// published: {}", published));
    lines.push(format!("// featured: {}", featured));
    lines.push("// ---".to_string());
    lines.join("\n")
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

    #[test]
    fn test_build_typst_frontmatter_full() {
        let fm = build_typst_frontmatter(
            "My Post",
            Some("A description"),
            &["rust".to_string(), "web".to_string()],
            Some("Can"),
            true,
            false,
        );
        assert!(fm.starts_with("// ---"));
        assert!(fm.ends_with("// ---"));
        assert!(fm.contains("// title: My Post"));
        assert!(fm.contains("// description: A description"));
        assert!(fm.contains(r#"// tags: ["rust", "web"]"#));
        assert!(fm.contains("// author: Can"));
        assert!(fm.contains("// published: true"));
        assert!(fm.contains("// featured: false"));
    }

    #[test]
    fn test_build_typst_frontmatter_minimal() {
        let fm = build_typst_frontmatter("Title Only", None, &[], None, false, false);
        assert!(fm.contains("// title: Title Only"));
        assert!(!fm.contains("// description:"));
        assert!(!fm.contains("// tags:"));
        assert!(!fm.contains("// author:"));
        assert!(fm.contains("// published: false"));
    }
}
