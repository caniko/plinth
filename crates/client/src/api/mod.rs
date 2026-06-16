pub mod activity;
pub mod blog;
pub mod common;
pub mod portfolio;
pub mod todo;

pub use common::get_site_config;
pub use common::get_site_content;

#[cfg(feature = "brick-blog")]
pub use blog::{
    get_all_series, get_blog_post_by_slug, get_blog_posts, get_blog_posts_by_tag, get_series_nav,
    get_series_posts,
};

#[cfg(feature = "brick-portfolio")]
pub use portfolio::{get_portfolio_item_by_slug, get_portfolio_items};

#[cfg(feature = "brick-todo")]
pub use todo::{get_todo_by_slug, get_todos, get_todos_by_tag};

#[cfg(feature = "brick-activity")]
pub use activity::{get_activity_item_by_id, get_activity_list};
