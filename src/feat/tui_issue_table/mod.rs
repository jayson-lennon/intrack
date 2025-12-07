mod draw;
mod input;

pub use draw::IssueTableDraw;
pub use input::IssueListPageInput;
use ratatui::widgets::TableState;

use crate::feat::tui_widget::InputBoxState;

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
            Self::Custom(key) => write!(f, "{}", key),
        }
    }
}

#[derive(Debug)]
pub struct IssueTableState {
    table: TableState,
    filter: InputBoxState,
    columns: Vec<Column>,
}

impl Default for IssueTableState {
    fn default() -> Self {
        Self {
            table: TableState::default(),
            filter: InputBoxState::default(),
            columns: vec![
                Column::Id,
                Column::Title,
                Column::Created,
                Column::Status,
                Column::Priority,
                Column::CreatedBy,
            ],
        }
    }
}

impl IssueTableState {
    fn cursor_previous(&mut self) {
        if let Some(current) = self.table.selected() {
            let index = current.saturating_sub(1);
            self.cursor_to_item(index);
        } else {
            self.cursor_to_item(0);
        }
    }

    fn cursor_next(&mut self) {
        if let Some(current) = self.table.selected() {
            let index = current.saturating_add(1);
            self.cursor_to_item(index);
        } else {
            self.cursor_to_item(0);
        }
    }
    fn cursor_to_item(&mut self, index: usize) {
        self.table.select(Some(index));
    }

    pub fn filter(&self) -> &InputBoxState {
        &self.filter
    }

    pub fn filter_mut(&mut self) -> &mut InputBoxState {
        &mut self.filter
    }
}
