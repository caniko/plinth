use super::ProjectGrid;

/// Build a [`ProjectGrid`] model from deserialized config values.
///
/// Template markers: `<section class="project-grid">`,
/// `<div class="project-list">`.
pub fn build_project_grid(id: Option<String>, heading: String, intro: String) -> ProjectGrid {
    ProjectGrid { id, heading, intro }
}
