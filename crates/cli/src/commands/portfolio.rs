use std::path::Path;

use anyhow::{Context, Result};
use plinth_shared::{ContentFormat, PortfolioItem, PublishPortfolioRequest};

use crate::{api_client::ApiClient, ui};

/// Publish a portfolio item from a TOML manifest.
pub async fn publish(path: &Path, api_client: &ApiClient) -> Result<()> {
    if !path.exists() {
        anyhow::bail!("File not found: {}", path.display());
    }

    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;
    let mut request: PublishPortfolioRequest = toml::from_str(&content)
        .with_context(|| format!("Failed to parse portfolio manifest: {}", path.display()))?;

    validate_manifest(&mut request)?;

    ui::status(
        "Read",
        &format!("{} ({} bytes)", path.display(), content.len()),
    );
    ui::status("Slug", request.slug.as_deref().unwrap_or("<generated>"));
    ui::status("Format", "markdown");

    let sp = ui::spinner("Publishing portfolio item...");
    let response = api_client.publish_portfolio(request).await?;
    sp.finish_and_clear();

    ui::success(&response.message);
    ui::status("Slug", &response.slug);
    if let Some(id) = response.id {
        ui::status("ID", &id);
    }

    Ok(())
}

fn validate_manifest(request: &mut PublishPortfolioRequest) -> Result<()> {
    request.title = required_text("title", &request.title)?;
    request.description = required_text("description", &request.description)?;

    if request.tech_stack.is_empty() {
        anyhow::bail!("tech_stack is required and must contain at least one value");
    }

    for tech in &mut request.tech_stack {
        *tech = required_text("tech_stack entries", tech)?;
    }

    let slug = request
        .slug
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| PortfolioItem::slugify(&request.title));

    if slug.is_empty() {
        anyhow::bail!("slug is required when title cannot be slugified");
    }
    request.slug = Some(slug);

    let content_format = request
        .content_format
        .clone()
        .unwrap_or(ContentFormat::Markdown);
    if content_format != ContentFormat::Markdown {
        anyhow::bail!("portfolio manifests currently support only content_format = \"markdown\"");
    }
    request.content_format = Some(ContentFormat::Markdown);

    if let Some(content) = &mut request.content {
        *content = content.trim().to_string();
        if content.is_empty() {
            request.content = None;
        }
    }

    Ok(())
}

fn required_text(field: &str, value: &str) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        anyhow::bail!("{field} is required");
    }
    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_generates_slug_and_defaults_markdown() {
        let mut request = PublishPortfolioRequest {
            id: None,
            slug: None,
            title: "My Tool".to_string(),
            description: "A useful thing".to_string(),
            content: None,
            html_content: None,
            tech_stack: vec!["Rust".to_string()],
            link: None,
            demo: None,
            image_url: None,
            date: chrono::Utc::now(),
            featured: false,
            order: 0,
            content_format: None,
        };

        validate_manifest(&mut request).unwrap();

        assert_eq!(request.slug.as_deref(), Some("my-tool"));
        assert_eq!(request.content_format, Some(ContentFormat::Markdown));
    }
}
