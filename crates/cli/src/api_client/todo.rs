use anyhow::{Context, Result};
#[cfg(feature = "brick-todo")]
use plinth_shared::{CreateTodoRequest, TodoListItem, UpdateTodoRequest};

use super::client::ApiClient;

#[cfg(feature = "brick-todo")]
impl ApiClient {
    /// Create a new TODO item
    pub async fn create_todo(&self, request: CreateTodoRequest) -> Result<()> {
        let url = format!("{}/api/admin/todos", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send create TODO request")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Create TODO failed (HTTP {status}): {error_text}");
        }

        Ok(())
    }

    /// Update an existing TODO item
    pub async fn update_todo(&self, slug: &str, request: UpdateTodoRequest) -> Result<()> {
        let url = format!("{}/api/admin/todos/{}", self.base_url, slug);

        let response = self
            .client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send update TODO request")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Update TODO '{slug}' failed (HTTP {status}): {error_text}");
        }

        Ok(())
    }

    /// Delete a TODO item
    pub async fn delete_todo(&self, slug: &str) -> Result<()> {
        let url = format!("{}/api/admin/todos/{}", self.base_url, slug);

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .context("Failed to send delete TODO request")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Delete TODO '{slug}' failed (HTTP {status}): {error_text}");
        }

        Ok(())
    }

    /// List all TODO items
    pub async fn list_todos(&self) -> Result<Vec<TodoListItem>> {
        let url = format!("{}/api/admin/todos", self.base_url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .context("Failed to send list TODOs request")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("List TODOs failed (HTTP {status}): {error_text}");
        }

        let items = response.json().await.context("Failed to parse TODO list")?;
        Ok(items)
    }
}
