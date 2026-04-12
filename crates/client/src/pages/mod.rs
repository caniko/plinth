mod about;
mod home;
mod not_found;
mod support;

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

// Core pages (always present)
pub use about::AboutPage;
pub use home::HomePage;
pub use not_found::NotFound;
pub use support::SupportPage;

// Blog pages
#[cfg(feature = "brick-blog")]
pub use blog_list::BlogListPage;
#[cfg(feature = "brick-blog")]
pub use blog_post::BlogPostPage;
#[cfg(feature = "brick-blog")]
pub use blog_tag::BlogTagPage;
#[cfg(feature = "brick-blog")]
pub use series_detail::SeriesDetailPage;
#[cfg(feature = "brick-blog")]
pub use series_list::SeriesListPage;

// Portfolio pages
#[cfg(feature = "brick-portfolio")]
pub use portfolio::PortfolioPage;
#[cfg(feature = "brick-portfolio")]
pub use portfolio_detail::PortfolioDetailPage;

// Todo pages
#[cfg(feature = "brick-todo")]
pub use todo_detail::TodoDetailPage;
#[cfg(feature = "brick-todo")]
pub use todo_list::TodoListPage;
#[cfg(feature = "brick-todo")]
pub use todo_tag::TodoTagPage;
