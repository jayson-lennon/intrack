use ratatui::widgets::TableState;

use crate::feat::{
    tui_issue_table::{Column, SortDirection},
    tui_widget::InputBoxState,
};

#[derive(Debug)]
pub struct IssueTableState {
    pub(in crate::feat::tui_issue_table) table: TableState,
    pub(in crate::feat::tui_issue_table) filter_input: InputBoxState,
    pub(in crate::feat::tui_issue_table) sort_by: Column,
    pub(in crate::feat::tui_issue_table) sort_direction: SortDirection,
    pub(in crate::feat::tui_issue_table) columns: Vec<Column>,
}

impl Default for IssueTableState {
    fn default() -> Self {
        Self {
            table: TableState::default(),
            filter_input: InputBoxState::default(),
            sort_by: Column::Created,
            sort_direction: SortDirection::Descending,
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
    pub fn sort_next_column(&mut self) {
        if let Some(i) = self.columns.iter().position(|c| *c == self.sort_by) {
            let next_i = (i + 1) % self.columns.len();
            self.sort_by = self.columns[next_i].clone();
        }
    }

    pub fn sort_previous_column(&mut self) {
        if let Some(i) = self.columns.iter().position(|c| *c == self.sort_by) {
            let prev_i = (i + self.columns.len() - 1) % self.columns.len();
            self.sort_by = self.columns[prev_i].clone();
        }
    }

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

    pub fn filter_input_state(&self) -> &InputBoxState {
        &self.filter_input
    }

    pub fn filter_input_state_mut(&mut self) -> &mut InputBoxState {
        &mut self.filter_input
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

    pub fn sort_direction(&self) -> &SortDirection {
        &self.sort_direction
    }

    pub fn sort_by(&self) -> &Column {
        &self.sort_by
    }

    pub fn set_sort_by(&mut self, sort_by: Column) {
        self.sort_by = sort_by;
    }

    pub fn set_sort_direction(&mut self, sort_direction: SortDirection) {
        self.sort_direction = sort_direction;
    }
}
