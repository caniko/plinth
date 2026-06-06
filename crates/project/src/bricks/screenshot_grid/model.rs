#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScreenshotGrid {
    pub id: String,
    pub heading: String,
    pub intro: String,
    pub screenshots: Vec<Screenshot>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Screenshot {
    pub src: String,
    pub alt: String,
    pub caption: String,
}
