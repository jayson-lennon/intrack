mod draw;
mod input;
mod state;

pub use draw::IssueTableDraw;
pub use input::IssueListPageInput;
pub use state::IssueTableState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Column {
    Id,
    Title,
    Created,
    Status,
    Priority,
    CreatedBy,
    Custom(String),
}

impl std::fmt::Display for Column {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Id => write!(f, "ID"),
            Self::Title => write!(f, "Title"),
            Self::Created => write!(f, "Created"),
            Self::Status => write!(f, "Status"),
            Self::Priority => write!(f, "Priority"),
            Self::CreatedBy => write!(f, "Created By"),
            Self::Custom(key) => write!(f, "{key}"),
        }
    }
}
