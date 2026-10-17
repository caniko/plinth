pub(crate) mod serde;
mod types;

#[cfg(test)]
mod tests;

#[cfg(feature = "brick-capability-matrix")]
pub(crate) use serde::resolve_path;
pub use serde::{load_project_site, project_watch_paths};
pub use types::*;
