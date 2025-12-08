mod draw;
mod input;
mod state;

pub use draw::IssueTableDraw;
use error_stack::Report;
pub use input::IssueTablePageInput;
pub use state::IssueTableState;
use strum::EnumIter;
use wherror::Error;

use crate::feat::{
    issue::{Issue, Status},
    issues::Issues,
};

#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum SortDirection {
    #[default]
    Ascending,
    Descending,
}

#[derive(Debug, Error)]
#[error(debug)]
pub struct ColumnParseError;

#[derive(Debug, Clone, Hash, PartialEq, Eq, EnumIter)]
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

impl std::str::FromStr for Column {
    type Err = Report<ColumnParseError>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_lowercase();
        match lower.as_str() {
            "id" => Ok(Column::Id),
            "title" => Ok(Column::Title),
            "created" => Ok(Column::Created),
            "status" => Ok(Column::Status),
            "priority" => Ok(Column::Priority),
            "createdby" | "created by" => Ok(Column::CreatedBy),
            _ => Ok(Column::Custom(s.trim().to_string())),
        }
    }
}

fn apply_issue_filter<'a>(filter_text: &str, issues: &'a Issues) -> Vec<&'a Issue> {
    let filter_text = filter_text.to_lowercase();
    let remove_closed = filter_text.contains("!closed");
    let remove_open = filter_text.contains("!open");
    let filter_text = filter_text.replace("!closed", "");
    let filter_text = filter_text.replace("!open", "");
    issues
        .iter_issues()
        .filter(|issue| !(remove_open && issue.status == Status::Open))
        .filter(|issue| !(remove_closed && issue.status == Status::Closed))
        .filter(|issue| filter_text.is_empty() || issue.title.to_lowercase().contains(&filter_text))
        .collect()
}

fn apply_issue_sort(filtered_issues: &mut Vec<&Issue>, sort_col: &Column, sort_dir: SortDirection) {
    filtered_issues.sort_by(|issue1, issue2| {
        let ord = match sort_col {
            Column::Id => issue1.id.cmp(&issue2.id),
            Column::Title => issue1.title.cmp(&issue2.title),
            Column::Created => issue1.created.cmp(&issue2.created),
            Column::Status => issue1.status.cmp(&issue2.status),
            Column::Priority => issue1.priority.cmp(&issue2.priority),
            Column::CreatedBy => issue1.created_by.cmp(&issue2.created_by),
            Column::Custom(_) => todo!(),
        };
        match sort_dir {
            SortDirection::Ascending => ord,
            SortDirection::Descending => ord.reverse(),
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("id", Column::Id)]
    #[case("ID", Column::Id)]
    #[case("Id", Column::Id)]
    #[case("title", Column::Title)]
    #[case("TITLE", Column::Title)]
    #[case("Title", Column::Title)]
    #[case("created", Column::Created)]
    #[case("CREATED", Column::Created)]
    #[case("Created", Column::Created)]
    #[case("status", Column::Status)]
    #[case("STATUS", Column::Status)]
    #[case("priority", Column::Priority)]
    #[case("PRIORITY", Column::Priority)]
    #[case("Priority", Column::Priority)]
    #[case("createdby", Column::CreatedBy)]
    #[case("CREATEDBY", Column::CreatedBy)]
    #[case("CreatedBy", Column::CreatedBy)]
    #[case("Created By", Column::CreatedBy)]
    fn test_parse_known(#[case] input: &str, #[case] expected: Column) {
        use std::str::FromStr;

        assert_eq!(Column::from_str(input).unwrap(), expected);
    }

    #[rstest]
    #[case("foo", "foo")]
    #[case("Foo", "Foo")]
    #[case("", "")]
    #[case(" ", "")]
    #[case("  foo  ", "foo")]
    #[case("ID ", "ID")]
    #[case("id123", "id123")]
    #[case("Custom Field", "Custom Field")]
    fn test_parse_custom(#[case] input: &str, #[case] expected: &str) {
        use std::str::FromStr;

        let col = Column::from_str(input).unwrap();
        match col {
            Column::Custom(s) => assert_eq!(s.as_str(), expected),
            _ => panic!("Expected `Column::Custom({expected:?})`, got {col:?}"),
        }
    }
}
