use shared::{BlogListItem, BlogPost, PortfolioItem};

/// Stub API functions for data fetching.
/// In production, these would call the server API endpoints.
pub async fn get_blog_posts() -> Result<Vec<BlogListItem>, String> {
    Ok(vec![])
}

pub async fn get_blog_post_by_slug(_slug: String) -> Result<Option<BlogPost>, String> {
    Ok(None)
}

pub async fn get_blog_posts_by_tag(_tag: String) -> Result<Vec<BlogListItem>, String> {
    Ok(vec![])
}

pub async fn get_portfolio_items() -> Result<Vec<PortfolioItem>, String> {
    Ok(vec![])
}

pub async fn get_portfolio_item_by_slug(_slug: String) -> Result<Option<PortfolioItem>, String> {
    Ok(None)
}
