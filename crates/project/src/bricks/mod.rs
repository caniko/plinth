//! Project brick system — modular, optional page elements for static project sites.

/// Trait implemented by every project page-element brick.
pub trait ProjectBrick: Send + Sync + 'static {
    /// Unique identifier for this project brick.
    fn name(&self) -> &'static str;
}

#[cfg(feature = "brick-capability-matrix")]
pub mod capability_matrix;
#[cfg(feature = "brick-comparison")]
pub mod comparison;
#[cfg(feature = "brick-custom")]
pub mod custom;
#[cfg(feature = "brick-feature-grid")]
pub mod feature_grid;
#[cfg(feature = "brick-hero")]
pub mod hero;
#[cfg(feature = "brick-install")]
pub mod install;
#[cfg(feature = "brick-person-mention")]
pub mod person_mention;
#[cfg(feature = "brick-screenshot-grid")]
pub mod screenshot_grid;

/// Collect all enabled project bricks for composition and diagnostics.
#[allow(clippy::vec_init_then_push)]
#[allow(unused_mut)]
pub fn enabled_bricks() -> Vec<Box<dyn ProjectBrick>> {
    let mut bricks: Vec<Box<dyn ProjectBrick>> = Vec::new();

    #[cfg(feature = "brick-hero")]
    bricks.push(Box::new(hero::HeroBrick));

    #[cfg(feature = "brick-feature-grid")]
    bricks.push(Box::new(feature_grid::FeatureGridBrick));

    #[cfg(feature = "brick-install")]
    bricks.push(Box::new(install::InstallBrick));

    #[cfg(feature = "brick-screenshot-grid")]
    bricks.push(Box::new(screenshot_grid::ScreenshotGridBrick));

    #[cfg(feature = "brick-capability-matrix")]
    bricks.push(Box::new(capability_matrix::CapabilityMatrixBrick));

    #[cfg(feature = "brick-comparison")]
    bricks.push(Box::new(comparison::ComparisonBrick));

    #[cfg(feature = "brick-custom")]
    bricks.push(Box::new(custom::CustomBrick));

    #[cfg(feature = "brick-person-mention")]
    bricks.push(Box::new(person_mention::PersonMentionBrick));

    bricks
}

#[cfg(test)]
mod tests {
    use super::enabled_bricks;

    #[test]
    fn registry_lists_enabled_project_bricks() {
        let names = enabled_bricks()
            .into_iter()
            .map(|brick| brick.name())
            .collect::<Vec<_>>();

        #[cfg(feature = "brick-hero")]
        assert!(names.contains(&"hero"));
        #[cfg(feature = "brick-feature-grid")]
        assert!(names.contains(&"feature_grid"));
        #[cfg(feature = "brick-install")]
        assert!(names.contains(&"install"));
        #[cfg(feature = "brick-screenshot-grid")]
        assert!(names.contains(&"screenshot_grid"));
        #[cfg(feature = "brick-capability-matrix")]
        assert!(names.contains(&"capability_matrix"));
        #[cfg(feature = "brick-comparison")]
        assert!(names.contains(&"comparison"));
        #[cfg(feature = "brick-custom")]
        assert!(names.contains(&"custom"));
        #[cfg(feature = "brick-person-mention")]
        assert!(names.contains(&"person_mention"));

        #[cfg(not(any(
            feature = "brick-hero",
            feature = "brick-feature-grid",
            feature = "brick-install",
            feature = "brick-screenshot-grid",
            feature = "brick-capability-matrix",
            feature = "brick-comparison",
            feature = "brick-custom",
            feature = "brick-person-mention",
        )))]
        assert!(names.is_empty());
    }
}
