mod about;
mod home;
mod not_found;
mod support;

#[cfg(feature = "brick-activity")]
mod activity;
#[cfg(feature = "brick-activity")]
mod activity_detail;

#[cfg(feature = "brick-blog")]
mod blog_list;
#[cfg(feature = "brick-blog")]
mod blog_post;
#[cfg(feature = "brick-blog")]
mod blog_tag;
#[cfg(feature = "brick-blog")]
mod series_detail;
#[cfg(feature = "brick-blog")]
mod series_list;

#[cfg(feature = "brick-portfolio")]
mod portfolio;
#[cfg(feature = "brick-portfolio")]
mod portfolio_detail;

#[cfg(feature = "brick-todo")]
mod todo_detail;
#[cfg(feature = "brick-todo")]
mod todo_list;
#[cfg(feature = "brick-todo")]
mod todo_tag;

/// About page — renders editable site content with a default fallback bio.
pub use about::AboutPage;
/// Home page — animated background, tagline, recent posts, projects, activity.
pub use home::HomePage;
/// 404 page — shown when no route matches.
pub use not_found::NotFound;
/// Support page — donation platform cards and custom support content.
pub use support::SupportPage;

/// Activity listing — curated external contributions ranked by impact.
#[cfg(feature = "brick-activity")]
pub use activity::ActivityPage;
/// Activity detail — single contribution with full body and metadata.
#[cfg(feature = "brick-activity")]
pub use activity_detail::ActivityDetailPage;

/// Blog listing — all published posts with series info and tags.
#[cfg(feature = "brick-blog")]
pub use blog_list::BlogListPage;
/// Blog post detail — full article with series nav, tags, and SEO meta.
#[cfg(feature = "brick-blog")]
pub use blog_post::BlogPostPage;
/// Blog posts filtered by a tag.
#[cfg(feature = "brick-blog")]
pub use blog_tag::BlogTagPage;
/// Blog series detail — ordered list of posts in a series.
#[cfg(feature = "brick-blog")]
pub use series_detail::SeriesDetailPage;
/// Blog series listing — all series with post count and total reading time.
#[cfg(feature = "brick-blog")]
pub use series_list::SeriesListPage;

/// Portfolio listing — project cards with tech stack and links.
#[cfg(feature = "brick-portfolio")]
pub use portfolio::PortfolioPage;
/// Portfolio detail — single project with full description and links.
#[cfg(feature = "brick-portfolio")]
pub use portfolio_detail::PortfolioDetailPage;

/// Todo detail — single bucket list item with completion status.
#[cfg(feature = "brick-todo")]
pub use todo_detail::TodoDetailPage;
/// Todo listing — bucket list items with completion state.
#[cfg(feature = "brick-todo")]
pub use todo_list::TodoListPage;
/// Todo items filtered by a tag.
#[cfg(feature = "brick-todo")]
pub use todo_tag::TodoTagPage;
