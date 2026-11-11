//! Company presentation profiles and typed, independently composable sections.

mod model;
pub(crate) mod render;
mod validation;

pub use model::*;
pub(crate) use validation::validate;

pub(crate) const FONT_URL: &str = "/fonts/plinth-company-inter.woff2";
pub(crate) const FONT_FAMILY: &str = "Inter";
pub(crate) const SECTION_NAMES: &[&str] = &[
    "mission",
    "team",
    "testimonials",
    "partnerships",
    "metrics",
    "case_studies",
    "contact",
];

pub(crate) struct CompanyBrick(pub &'static str);
impl crate::bricks::ProjectBrick for CompanyBrick {
    fn name(&self) -> &'static str {
        self.0
    }
}

/// Produce a complete company starter using the same typed configuration as project sites.
pub fn starter_config(title: &str, description: &str) -> String {
    let escape =
        |value: &str| serde_json::to_string(value).expect("serializing a string is infallible");
    // Substitute template fragments before joining, so placeholder-like user text
    // is never interpreted as another template token.
    include_str!("starter.toml")
        .split("__TITLE__")
        .map(|part| part.replace("__DESCRIPTION__", &escape(description)))
        .collect::<Vec<_>>()
        .join(&escape(title))
}

/// Supply a neutral editorial palette while retaining explicit brand overrides.
pub fn apply_theme_defaults(theme: &mut crate::ProjectTheme) {
    if theme.font_family.is_none() && theme.font_url.is_none() {
        theme.font_family = Some(FONT_FAMILY.into());
        theme.font_url = Some(FONT_URL.into());
    }
    for (field, value) in [
        (&mut theme.paper, "#F7F6F2"),
        (&mut theme.surface, "#EEEDE6"),
        (&mut theme.ink, "#262A25"),
        (&mut theme.ink_soft, "#62685D"),
        (&mut theme.line, "#D9DCD2"),
        (&mut theme.accent, "#3F5E45"),
        (&mut theme.accent_soft, "#E3E8DD"),
        (&mut theme.secondary, "#314A36"),
    ] {
        field.get_or_insert_with(|| value.into());
    }
}
