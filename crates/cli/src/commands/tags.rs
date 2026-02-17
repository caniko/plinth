use anyhow::Result;

use crate::api_client::ApiClient;
use crate::ui;

pub async fn list_tags(api_client: &ApiClient) -> Result<()> {
    let tags = api_client.list_tags().await?;

    if tags.is_empty() {
        ui::status("Info", "No tags found.");
        return Ok(());
    }

    ui::table_header(&[("TAG", 20), ("POSTS", 5)]);
    for tag in tags {
        println!("{:<20}  {:<5}", tag.name, tag.post_count);
    }

    Ok(())
}

pub async fn add_tag(post_slug: &str, tag: &str, api_client: &ApiClient) -> Result<()> {
    api_client.add_tag_to_post(post_slug, tag).await?;
    ui::success(&format!("Tag '{tag}' added to post '{post_slug}'"));
    Ok(())
}

pub async fn remove_tag(post_slug: &str, tag_slug: &str, api_client: &ApiClient) -> Result<()> {
    api_client.remove_tag_from_post(post_slug, tag_slug).await?;
    ui::success(&format!("Tag '{tag_slug}' removed from post '{post_slug}'"));
    Ok(())
}
