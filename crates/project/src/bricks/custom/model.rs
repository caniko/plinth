use std::sync::Arc;

/// A project section whose HTML is produced by a caller-supplied closure.
///
/// No built-in template — the `render` closure is called each time the
/// section is emitted.  Use for widgets, embeds, or one-off sections
/// that don't warrant a dedicated brick type.
#[derive(Clone)]
pub struct CustomSection {
    /// Optional `id` attribute on the wrapping element.
    pub id: Option<String>,
    /// Closure that produces the full HTML for this section.
    pub render: Arc<dyn Fn() -> String + Send + Sync>,
}

impl CustomSection {
    /// Create a new custom section with the given render closure.
    ///
    /// The section starts with `id: None`; use [`Self::id`] to set one.
    #[must_use]
    pub fn new(render: impl Fn() -> String + Send + Sync + 'static) -> Self {
        Self {
            id: None,
            render: Arc::new(render),
        }
    }

    /// Set the optional `id` attribute on this section (builder style).
    #[must_use]
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
}
