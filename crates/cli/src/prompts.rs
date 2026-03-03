use anyhow::Result;
use inquire::{Confirm, Editor, InquireError, Select, Text};

const POST_TEMPLATE: &str = include_str!("../templates/post.typ");
const BUCKET_LIST_TEMPLATE: &str = include_str!("../templates/bucket-list.typ");

/// Available templates for the editor pre-fill.
#[derive(Clone)]
pub struct Template {
    pub label: &'static str,
    pub content: &'static str,
}

impl std::fmt::Display for Template {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.label)
    }
}

pub const TEMPLATE_POST: Template = Template {
    label: "Blog post template",
    content: POST_TEMPLATE,
};

pub const TEMPLATE_BUCKET_LIST: Template = Template {
    label: "Bucket list template",
    content: BUCKET_LIST_TEMPLATE,
};

const TEMPLATE_EMPTY: Template = Template {
    label: "Empty file",
    content: "",
};

/// How the user chose to provide content.
pub enum ContentSource {
    /// Content written in an editor.
    EditorContent(String),
    /// Path to an existing file.
    ExistingFile(String),
    /// No content.
    Skip,
}

/// Convert an inquire error to anyhow, with a friendly cancellation message.
fn handle_inquire_err(e: InquireError) -> anyhow::Error {
    match e {
        InquireError::OperationCanceled | InquireError::OperationInterrupted => {
            anyhow::anyhow!("Cancelled.")
        }
        other => other.into(),
    }
}

/// Prompt for a required text field.
pub fn prompt_text(message: &str, default: Option<&str>) -> Result<String> {
    let mut prompt = Text::new(message);
    if let Some(d) = default {
        prompt = prompt.with_default(d);
    }
    prompt.prompt().map_err(handle_inquire_err)
}

/// Prompt for an optional text field (empty string → None).
pub fn prompt_optional_text(message: &str) -> Result<Option<String>> {
    let value = Text::new(message)
        .with_help_message("Leave empty to skip")
        .prompt()
        .map_err(handle_inquire_err)?;
    if value.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

/// Prompt for comma-separated tags.
pub fn prompt_tags(message: &str) -> Result<Vec<String>> {
    let value = Text::new(message)
        .with_help_message("Comma-separated, e.g. rust, typst, web")
        .prompt()
        .map_err(handle_inquire_err)?;
    let tags: Vec<String> = value
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    Ok(tags)
}

/// Prompt for a boolean with a default value.
pub fn prompt_bool(message: &str, default: bool) -> Result<bool> {
    Confirm::new(message)
        .with_default(default)
        .prompt()
        .map_err(handle_inquire_err)
}

/// Prompt the user to provide content via editor, file path, or skip.
///
/// `templates` should contain the context-relevant templates (e.g. post template
/// for publish, bucket-list template for todo). An "Empty file" option is always
/// appended.
pub fn prompt_content(templates: &[Template]) -> Result<ContentSource> {
    let choices = vec!["Open in editor", "Use existing file", "Skip"];
    let selection = Select::new("How would you like to provide content?", choices)
        .prompt()
        .map_err(handle_inquire_err)?;

    match selection {
        "Open in editor" => {
            let mut options: Vec<Template> = templates.to_vec();
            options.push(TEMPLATE_EMPTY.clone());

            let template = if options.len() == 1 {
                // Only "Empty file" — no need to ask
                TEMPLATE_EMPTY.clone()
            } else {
                Select::new("Start from template?", options)
                    .prompt()
                    .map_err(handle_inquire_err)?
            };

            let content = Editor::new("Edit content (save and close to continue)")
                .with_predefined_text(template.content)
                .with_file_extension(".typ")
                .prompt()
                .map_err(handle_inquire_err)?;

            if content.trim().is_empty() {
                Ok(ContentSource::Skip)
            } else {
                Ok(ContentSource::EditorContent(content))
            }
        }
        "Use existing file" => {
            let path = Text::new("File path:")
                .with_help_message("Path to a .typ or .md file")
                .prompt()
                .map_err(handle_inquire_err)?;
            Ok(ContentSource::ExistingFile(path))
        }
        _ => Ok(ContentSource::Skip),
    }
}
