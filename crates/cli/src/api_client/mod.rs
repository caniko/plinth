#[cfg(feature = "brick-activity")]
mod activity;
#[cfg(feature = "brick-blog")]
mod blog;
mod client;
mod content;
#[cfg(any(
    feature = "brick-activity",
    feature = "brick-blog",
    feature = "brick-portfolio"
))]
mod error;
#[cfg(feature = "brick-portfolio")]
mod portfolio;
#[cfg(feature = "brick-todo")]
mod todo;

#[cfg(test)]
mod tests;

pub use client::ApiClient;
