use ratatui::widgets::TableState;

use crate::feat::{tui_issue_table::Column, tui_widget::InputBoxState};

#[derive(Debug)]
pub struct IssueTableState {
    pub(in crate::feat::tui_issue_table) table: TableState,
    filter: InputBoxState,
    pub(in crate::feat::tui_issue_table) columns: Vec<Column>,
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
    pub fn cursor_previous(&mut self) {
        if let Some(current) = self.table.selected() {
            let index = current.saturating_sub(1);
            self.cursor_to_item(index);
        } else {
            self.cursor_to_item(0);
        }
    }

    pub fn cursor_next(&mut self) {
        if let Some(current) = self.table.selected() {
            let index = current.saturating_add(1);
            self.cursor_to_item(index);
        } else {
            self.cursor_to_item(0);
        }
    }

    pub fn cursor_to_item(&mut self, index: usize) {
        self.table.select(Some(index));
    }

    pub fn filter(&self) -> &InputBoxState {
        &self.filter
    }

    pub fn filter_mut(&mut self) -> &mut InputBoxState {
        &mut self.filter
    }

    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    pub fn set_columns(&mut self, columns: Vec<Column>) {
        self.columns = columns;
    }

    pub fn columns_mut(&mut self) -> &mut Vec<Column> {
        &mut self.columns
    }
}
