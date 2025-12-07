/// Types that can be attached to error reports.
pub mod report {
    /// Show a suggestion.
    pub struct Suggestion(pub &'static str);

    /// Indicate that some item is missing.
    pub struct Missing(pub String);
}

#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq)]
pub enum Page {
    #[default]
    IssueList,
}

impl Into<usize> for Page {
    fn into(self) -> usize {
        match self {
            Self::IssueList => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq)]
pub enum Focus {
    #[default]
    IssueList,
    IssueListFilter,
}
