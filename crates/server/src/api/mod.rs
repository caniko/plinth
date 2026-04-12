// API endpoints for publishing and management

pub mod admin;
pub mod feeds;
pub mod health;
pub mod images;

#[cfg(feature = "brick-blog")]
pub mod search;
