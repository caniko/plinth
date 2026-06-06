use std::sync::Arc;

#[derive(Clone)]
pub struct CustomSection {
    pub id: Option<String>,
    pub render: Arc<dyn Fn() -> String + Send + Sync>,
}

impl CustomSection {
    #[must_use]
    pub fn new(render: impl Fn() -> String + Send + Sync + 'static) -> Self {
        Self {
            id: None,
            render: Arc::new(render),
        }
    }

    #[must_use]
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
}
