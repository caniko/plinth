/// Activity (external contributions) server function wrappers.
pub mod activity;
/// Blog server function wrappers (posts, series, tags).
pub mod blog;
/// Core server function wrappers (site config, site content).
pub mod common;
/// Portfolio server function wrappers.
pub mod portfolio;
/// Todo / bucket list server function wrappers.
pub mod todo;

/// Fetch the full [`SiteConfig`](plinth_shared::SiteConfig) from the server.
pub use common::get_site_config;
/// Fetch a [`SiteContent`](plinth_shared::SiteContent) value by its key.
pub use common::get_site_content;

/// Fetch all blog series list items.
#[cfg(feature = "brick-blog")]
pub use blog::get_all_series;
/// Fetch a single blog post by slug.
#[cfg(feature = "brick-blog")]
pub use blog::get_blog_post_by_slug;
/// Fetch all published blog posts.
#[cfg(feature = "brick-blog")]
pub use blog::get_blog_posts;
/// Fetch blog posts matching a tag.
#[cfg(feature = "brick-blog")]
pub use blog::get_blog_posts_by_tag;
/// Fetch series navigation for a given post slug.
#[cfg(feature = "brick-blog")]
pub use blog::get_series_nav;
/// Fetch posts belonging to a series.
#[cfg(feature = "brick-blog")]
pub use blog::get_series_posts;

/// Fetch a single portfolio item by slug.
#[cfg(feature = "brick-portfolio")]
pub use portfolio::get_portfolio_item_by_slug;
/// Fetch all portfolio items.
#[cfg(feature = "brick-portfolio")]
pub use portfolio::get_portfolio_items;

/// Fetch a single todo item by slug.
#[cfg(feature = "brick-todo")]
pub use todo::get_todo_by_slug;
/// Fetch all todo items.
#[cfg(feature = "brick-todo")]
pub use todo::get_todos;
/// Fetch todo items matching a tag.
#[cfg(feature = "brick-todo")]
pub use todo::get_todos_by_tag;

/// Fetch a single activity item by ID.
#[cfg(feature = "brick-activity")]
pub use activity::get_activity_item_by_id;
/// Fetch the activity list (ranked contributions).
#[cfg(feature = "brick-activity")]
pub use activity::get_activity_list;
