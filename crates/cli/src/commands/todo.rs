use anyhow::{Context, Result};
use std::path::Path;

use crate::api_client::ApiClient;
use crate::typst_processor;
use crate::ui;
use plinth_shared::{CreateTodoRequest, UpdateTodoRequest};

/// Create a new TODO item
pub async fn create_todo(
    title: &str,
    description: &str,
    content_file: Option<&str>,
    tags: Option<&str>,
    order: i32,
    api_client: &ApiClient,
) -> Result<()> {
    // Parse optional content file (Typst)
    let (content, html_content) = if let Some(file_path) = content_file {
        let path = Path::new(file_path);
        if !path.exists() {
            anyhow::bail!("File not found: {}", file_path);
        }
        let sp = ui::spinner("Compiling Typst content...");
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", file_path))?;
        let stripped = typst_processor::strip_typst_frontmatter(&raw);
        let html = typst_processor::compile_typst_to_html(&stripped)?;
        sp.finish_and_clear();
        (Some(raw), Some(html))
    } else {
        (None, None)
    };

    // Parse tags
    let tag_list: Vec<String> = tags
        .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    let request = CreateTodoRequest {
        title: title.to_string(),
        slug: None,
        description: description.to_string(),
        content,
        html_content,
        tags: tag_list,
        completed: false,
        order,
    };

    let sp = ui::spinner("Creating TODO...");
    api_client.create_todo(request).await?;
    sp.finish_and_clear();
    ui::success(&format!("TODO created: {title}"));
    Ok(())
}

/// Update an existing TODO item
#[allow(clippy::too_many_arguments)]
pub async fn update_todo(
    slug: &str,
    complete: bool,
    uncomplete: bool,
    title: Option<&str>,
    description: Option<&str>,
    content_file: Option<&str>,
    tags: Option<&str>,
    order: Option<i32>,
    api_client: &ApiClient,
) -> Result<()> {
    // Parse optional content file
    let (content, html_content) = if let Some(file_path) = content_file {
        let path = Path::new(file_path);
        if !path.exists() {
            anyhow::bail!("File not found: {}", file_path);
        }
        let sp = ui::spinner("Compiling Typst content...");
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", file_path))?;
        let stripped = typst_processor::strip_typst_frontmatter(&raw);
        let html = typst_processor::compile_typst_to_html(&stripped)?;
        sp.finish_and_clear();
        (Some(raw), Some(html))
    } else {
        (None, None)
    };

    let completed = if complete {
        Some(true)
    } else if uncomplete {
        Some(false)
    } else {
        None
    };

    let tag_list = tags.map(|t| t.split(',').map(|s| s.trim().to_string()).collect());

    let request = UpdateTodoRequest {
        title: title.map(|s| s.to_string()),
        description: description.map(|s| s.to_string()),
        content,
        html_content,
        tags: tag_list,
        completed,
        order,
    };

    let sp = ui::spinner(&format!("Updating TODO: {slug}..."));
    api_client.update_todo(slug, request).await?;
    sp.finish_and_clear();
    ui::success(&format!("TODO updated: {slug}"));
    Ok(())
}

/// Delete a TODO item
pub async fn delete_todo(slug: &str, api_client: &ApiClient) -> Result<()> {
    let sp = ui::spinner(&format!("Deleting TODO: {slug}..."));
    api_client.delete_todo(slug).await?;
    sp.finish_and_clear();
    ui::success(&format!("TODO deleted: {slug}"));
    Ok(())
}

/// Interactive TODO creation — prompts for all fields.
pub async fn interactive_create_todo(api_client: &ApiClient) -> Result<()> {
    use crate::prompts::{self, ContentSource, TEMPLATE_BUCKET_LIST};

    let title = prompts::prompt_text("Title:", None)?;
    let description = prompts::prompt_text("Description:", None)?;
    let tags = prompts::prompt_tags("Tags:")?;
    let order_str = prompts::prompt_text("Display order:", Some("0"))?;
    let order: i32 = order_str
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid number: {}", order_str))?;

    let (content, html_content) = match prompts::prompt_content(&[TEMPLATE_BUCKET_LIST])? {
        ContentSource::EditorContent(text) => {
            if text.trim().is_empty() {
                (None, None)
            } else {
                let sp = ui::spinner("Compiling Typst content...");
                let html = typst_processor::compile_typst_to_html(&text)?;
                sp.finish_and_clear();
                (Some(text), Some(html))
            }
        }
        ContentSource::ExistingFile(file_path) => {
            let path = Path::new(&file_path);
            if !path.exists() {
                anyhow::bail!("File not found: {}", file_path);
            }
            let sp = ui::spinner("Compiling Typst content...");
            let raw = std::fs::read_to_string(path)
                .with_context(|| format!("Failed to read file: {}", file_path))?;
            let stripped = typst_processor::strip_typst_frontmatter(&raw);
            let html = typst_processor::compile_typst_to_html(&stripped)?;
            sp.finish_and_clear();
            (Some(raw), Some(html))
        }
        ContentSource::Skip => (None, None),
    };

    let request = CreateTodoRequest {
        title: title.clone(),
        slug: None,
        description,
        content,
        html_content,
        tags,
        completed: false,
        order,
    };

    let sp = ui::spinner("Creating TODO...");
    api_client.create_todo(request).await?;
    sp.finish_and_clear();
    ui::success(&format!("TODO created: {title}"));
    Ok(())
}

/// List all TODO items
pub async fn list_todos(api_client: &ApiClient) -> Result<()> {
    let items = api_client.list_todos().await?;

    if items.is_empty() {
        ui::status("Info", "No TODO items found.");
        return Ok(());
    }

    ui::status("Found", &format!("{} item(s)", items.len()));
    println!();
    for item in items {
        let check = if item.completed {
            format!("{}", ui::success_style().apply_to("[x]"))
        } else {
            format!("{}", ui::dim_style().apply_to("[ ]"))
        };
        let tags = if item.tags.is_empty() {
            String::new()
        } else {
            format!(
                " {}",
                ui::dim_style().apply_to(format!("[{}]", item.tags.join(", ")))
            )
        };
        println!(
            "  {} {}{}",
            check,
            ui::bold_style().apply_to(&item.title),
            tags
        );
        if !item.description.is_empty() {
            println!("      {}", ui::dim_style().apply_to(&item.description));
        }
    }

    Ok(())
}
