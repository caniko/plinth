use anyhow::{Context, Result};
use std::path::Path;

use crate::api_client::ApiClient;
use crate::typst_processor;
use crate::ui;
use plinth_shared::UpdateSiteContentRequest;

/// Set site content from a Typst file
pub async fn set_content(key: &str, file_path: &str, api_client: &ApiClient) -> Result<()> {
    let path = Path::new(file_path);
    if !path.exists() {
        anyhow::bail!("File not found: {}", file_path);
    }

    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", file_path))?;

    let sp = ui::spinner(&format!("Compiling and uploading '{key}'..."));

    // Extract optional frontmatter for title
    let frontmatter = typst_processor::extract_typst_frontmatter(&content)?;
    let title = frontmatter.and_then(|fm| fm.title);

    // Strip frontmatter before compiling
    let stripped = typst_processor::strip_typst_frontmatter(&content);

    // Compile Typst to HTML
    let html_content = typst_processor::compile_typst_to_html(&stripped)?;

    let request = UpdateSiteContentRequest {
        content,
        title,
        html_content,
    };

    api_client.update_site_content(key, request).await?;
    sp.finish_and_clear();

    ui::success(&format!("Site content '{key}' updated"));
    Ok(())
}

/// Get current site content by key
pub async fn get_content(key: &str, api_client: &ApiClient) -> Result<()> {
    let content = api_client.get_site_content(key).await?;
    match content {
        Some(c) => {
            ui::status("Key", &c.key);
            if let Some(title) = &c.title {
                ui::status("Title", title);
            }
            ui::status("Updated", &c.updated_at.to_string());
            println!("{}", ui::dim_style().apply_to("───"));
            println!("{}", c.content);
        }
        None => {
            ui::warn(&format!("No content found for key '{key}'"));
        }
    }
    Ok(())
}
