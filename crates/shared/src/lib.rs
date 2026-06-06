#[cfg(feature = "brick-activity")]
pub mod activity_item;
#[cfg(feature = "brick-blog")]
pub mod blog_post;
pub mod config;
pub mod content_format;
#[cfg(feature = "brick-portfolio")]
pub mod portfolio_item;
pub(crate) mod serde_helpers;
pub mod site_content;
pub mod tag;
#[cfg(feature = "brick-todo")]
pub mod todo_item;
#[cfg(feature = "config-toml")]
pub mod toml_config;

/// Current API version. Increment when making breaking changes.
pub const API_VERSION: u32 = 1;

// Re-export types for convenient importing
#[cfg(feature = "brick-activity")]
pub use activity_item::{
    ActivityItem, ActivityKind, ActivityListItem, ActivityRefreshHook, ActivityState,
    ActivityValidationError, FetchedActivity, Forge, ParseEnumError, PublishActivityRequest,
    RankingStrategy, validate_activity_fields,
};
#[cfg(feature = "brick-blog")]
pub use blog_post::{
    BlogListItem, BlogPost, PublishArticleRequest, SeriesEntry, SeriesListItem, SeriesNav,
    humanize_slug,
};
pub use config::SiteConfig;
pub use content_format::ContentFormat;
pub use plinth_person::{
    ExternalLink, LinkKind, PersonReference, ProjectReference, normalized_links,
};
#[cfg(feature = "brick-portfolio")]
pub use portfolio_item::{PortfolioItem, PublishPortfolioRequest};
pub use site_content::{SiteContent, UpdateSiteContentRequest};
pub use tag::{AddTagRequest, Tag};
#[cfg(feature = "brick-todo")]
pub use todo_item::{CreateTodoRequest, TodoItem, TodoListItem, UpdateTodoRequest};
