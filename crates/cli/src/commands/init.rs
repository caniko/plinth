use std::path::Path;

use anyhow::Result;

use crate::ui;

const POST_TEMPLATE: &str = include_str!("../../templates/post.typ");
const BUCKET_LIST_TEMPLATE: &str = include_str!("../../templates/bucket-list.typ");

/// Create a new file from a built-in template.
pub fn create_from_template(template: &str, output: Option<&str>) -> Result<()> {
    let (content, default_name) = match template {
        "post" => (POST_TEMPLATE, "post.typ"),
        "bucket-list" => (BUCKET_LIST_TEMPLATE, "bucket-list.typ"),
        other => anyhow::bail!(
            "Unknown template '{}'. Available templates: post, bucket-list",
            other
        ),
    };

    let dest = output.unwrap_or(default_name);
    let path = Path::new(dest);

    if path.exists() {
        anyhow::bail!("File already exists: {}", dest);
    }

    // Create parent directories if needed
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent)
            .map_err(|e| anyhow::anyhow!("Failed to create directory {}: {}", parent.display(), e))?;
    }

    std::fs::write(path, content)
        .map_err(|e| anyhow::anyhow!("Failed to write {}: {}", dest, e))?;

    ui::success(&format!("Created {} from '{}' template", dest, template));
    Ok(())
}
