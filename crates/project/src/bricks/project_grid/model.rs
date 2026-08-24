/// Model for a project-card grid section.
///
/// Cards come from `site.projects` at render time.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectGrid {
    /// Optional `id` attribute on the wrapping `<section>`.
    pub id: Option<String>,
    /// Section heading (`<h2>`).
    pub heading: String,
    /// Introductory paragraph below the heading.
    pub intro: String,
}
