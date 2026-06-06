#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowSteps {
    pub id: Option<String>,
    pub heading: String,
    pub intro: String,
    pub steps: Vec<WorkflowStep>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowStep {
    pub title: String,
    pub description: String,
}
