use serde::{Deserialize, Serialize};

/// Company-wide identity and presentation settings. The display name is the site title.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct CompanyProfile {
    pub legal_name: Option<String>,
    pub logo_src: Option<String>,
    pub favicon_src: Option<String>,
    pub social_image: Option<String>,
    pub location: Option<String>,
    #[serde(default = "default_language")]
    pub language: String,
    pub motion: Motion,
}

impl Default for CompanyProfile {
    fn default() -> Self {
        Self {
            legal_name: None,
            logo_src: None,
            favicon_src: None,
            social_image: None,
            location: None,
            language: default_language(),
            motion: Motion::default(),
        }
    }
}

fn default_language() -> String {
    "en".into()
}

/// A bounded ambient animation, progressively enhanced by a small browser script.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Motion {
    pub motif: MotionMotif,
    pub enabled: bool,
    pub duration_secs: f64,
    pub intensity: f64,
}

impl Default for Motion {
    fn default() -> Self {
        Self {
            motif: MotionMotif::Contour,
            enabled: true,
            duration_secs: 32.0,
            intensity: 0.3,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MotionMotif {
    #[default]
    Contour,
    Orbit,
    Ribbon,
}

impl MotionMotif {
    pub fn name(self) -> &'static str {
        match self {
            Self::Contour => "contour",
            Self::Orbit => "orbit",
            Self::Ribbon => "ribbon",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompanyLink {
    pub label: String,
    pub href: String,
}

/// A member need not have a portrait or personal website.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Member {
    pub id: String,
    pub name: String,
    pub role: String,
    #[serde(default)]
    pub bio: String,
    #[serde(default)]
    pub portrait_src: Option<String>,
    #[serde(default)]
    pub links: Vec<CompanyLink>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Value {
    pub title: String,
    pub description: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mission {
    pub id: String,
    pub heading: String,
    pub statement: String,
    #[serde(default)]
    pub eyebrow: String,
    #[serde(default)]
    pub vision: Option<String>,
    #[serde(default)]
    pub values: Vec<Value>,
}

/// A compact team keeps identity and role and links to the full team page.
#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TeamLayout {
    #[default]
    Full,
    Compact,
}

/// Compact teams can occupy less vertical space than the default strip.
#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TeamSize {
    #[default]
    Small,
    Medium,
    Large,
}

impl TeamSize {
    pub fn name(self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Team {
    pub id: String,
    pub heading: String,
    #[serde(default)]
    pub intro: String,
    #[serde(default)]
    pub eyebrow: String,
    /// Ordered references to top-level members; an empty list selects all members.
    #[serde(default)]
    pub members: Vec<String>,
    #[serde(default)]
    pub layout: TeamLayout,
    #[serde(default)]
    pub size: Option<TeamSize>,
    #[serde(default)]
    pub link: Option<CompanyLink>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Testimonial {
    pub quote: String,
    pub name: String,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub organization: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Testimonials {
    pub id: String,
    pub heading: String,
    #[serde(default)]
    pub intro: String,
    #[serde(default)]
    pub eyebrow: String,
    #[serde(default)]
    pub quotes: Vec<Testimonial>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Relationship {
    #[default]
    Partner,
    Customer,
    Investor,
    Affiliation,
}

impl Relationship {
    pub fn label(self) -> &'static str {
        match self {
            Self::Partner => "Partner",
            Self::Customer => "Customer",
            Self::Investor => "Investor",
            Self::Affiliation => "Affiliation",
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Partner {
    pub name: String,
    #[serde(default)]
    pub relationship: Relationship,
    #[serde(default)]
    pub logo_src: Option<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub href: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Partnerships {
    pub id: String,
    pub heading: String,
    #[serde(default)]
    pub intro: String,
    #[serde(default)]
    pub eyebrow: String,
    #[serde(default)]
    pub partners: Vec<Partner>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metric {
    pub value: String,
    pub label: String,
    pub context: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Metrics {
    pub id: String,
    pub heading: String,
    #[serde(default)]
    pub eyebrow: String,
    pub items: Vec<Metric>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseStudy {
    pub title: String,
    pub client: String,
    pub problem: String,
    pub approach: String,
    pub outcome: String,
    #[serde(default)]
    pub image_src: Option<String>,
    #[serde(default)]
    pub href: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseStudies {
    pub id: String,
    pub heading: String,
    #[serde(default)]
    pub intro: String,
    #[serde(default)]
    pub eyebrow: String,
    pub studies: Vec<CaseStudy>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Contact {
    pub id: String,
    pub heading: String,
    #[serde(default)]
    pub intro: String,
    #[serde(default)]
    pub eyebrow: String,
    pub label: String,
    pub href: String,
}

/// Company sections can be mixed freely with existing project bricks.
#[derive(Clone)]
pub enum CompanySection {
    Mission(Mission),
    Team(Team),
    Testimonials(Testimonials),
    Partnerships(Partnerships),
    Metrics(Metrics),
    CaseStudies(CaseStudies),
    Contact(Contact),
}

impl CompanySection {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Mission(_) => "mission",
            Self::Team(_) => "team",
            Self::Testimonials(_) => "testimonials",
            Self::Partnerships(_) => "partnerships",
            Self::Metrics(_) => "metrics",
            Self::CaseStudies(_) => "case_studies",
            Self::Contact(_) => "contact",
        }
    }
    pub fn id(&self) -> &str {
        match self {
            Self::Mission(s) => &s.id,
            Self::Team(s) => &s.id,
            Self::Testimonials(s) => &s.id,
            Self::Partnerships(s) => &s.id,
            Self::Metrics(s) => &s.id,
            Self::CaseStudies(s) => &s.id,
            Self::Contact(s) => &s.id,
        }
    }
    pub fn heading(&self) -> &str {
        match self {
            Self::Mission(s) => &s.heading,
            Self::Team(s) => &s.heading,
            Self::Testimonials(s) => &s.heading,
            Self::Partnerships(s) => &s.heading,
            Self::Metrics(s) => &s.heading,
            Self::CaseStudies(s) => &s.heading,
            Self::Contact(s) => &s.heading,
        }
    }
}
