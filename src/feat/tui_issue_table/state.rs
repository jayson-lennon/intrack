use std::collections::HashMap;

use ratatui::widgets::TableState;
use strum::IntoEnumIterator;

use crate::feat::{
    issue::IssueId,
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
    pub(in crate::feat::tui_issue_table) show_help: bool,

    pub(in crate::feat::tui_issue_table) display_map: HashMap<usize, IssueId>,
}

impl Default for IssueTableState {
    fn default() -> Self {
        Self {
            table: TableState::default(),
            filter_input: {
                let mut input = InputBoxState::default();
                input.set_text("!closed ");
                input
            },
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
            show_help: false,
            display_map: HashMap::default(),
        }
    }
}

impl IssueTableState {
    /// Returns the index of the currently selected items.
    ///
    /// # Notes
    ///
    /// Currently only 1 selection is supported. A `Vec` is returned here so that adding
    /// multi-select doesn't cause breaking changes.
    pub fn selected(&self) -> Vec<usize> {
        if let Some(selected) = self.table.selected() {
            vec![selected]
        } else {
            vec![]
        }
    }

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

    pub fn available_columns_for_editing(columns: &[Column]) -> String {
        let mut known_columns: Vec<Column> = Column::iter().collect();
        known_columns.pop(); // Last entry is always Custom, remove it.

        let present: std::collections::HashSet<Column> = columns.iter().cloned().collect();

        let mut lines = Vec::new();

        // First, include all input columns in their current order as active.
        for col in columns {
            lines.push(format!("{col}"));
        }

        // Then, append any missing known columns with comment prefix, in enum order.
        for col in &known_columns {
            if !present.contains(col) {
                lines.push(format!("# {col}"));
            }
        }

        lines.join("\n")
    }

    pub fn columns_from_edited(input: &str) -> Vec<Column> {
        let input = input.trim();
        input
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                if trimmed.starts_with('#') || trimmed.is_empty() {
                    None
                } else {
                    trimmed.parse().ok()
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(vec![], "# ID\n# Title\n# Created\n# Status\n# Priority\n# Created By")]
    #[case(vec![Column::Id], "ID\n# Title\n# Created\n# Status\n# Priority\n# Created By")]
    #[case(vec![Column::CreatedBy], "Created By\n# ID\n# Title\n# Created\n# Status\n# Priority")]
    #[case(vec![Column::Custom("Custom".to_string())], "Custom\n# ID\n# Title\n# Created\n# Status\n# Priority\n# Created By")]
    #[case(vec![Column::Id, Column::Status], "ID\nStatus\n# Title\n# Created\n# Priority\n# Created By")]
    #[case(vec![Column::Priority, Column::Title, Column::Status], "Priority\nTitle\nStatus\n# ID\n# Created\n# Created By")]
    #[case(vec![Column::Custom("Foo".to_string()), Column::CreatedBy, Column::Id], "Foo\nCreated By\nID\n# Title\n# Created\n# Status\n# Priority")]
    fn test_available_columns_preserves_order(
        #[case] input_columns: Vec<Column>,
        #[case] expected: &str,
    ) {
        assert_eq!(
            IssueTableState::available_columns_for_editing(&input_columns),
            expected
        );
    }

    #[rstest]
    #[case("", vec![])]
    #[case("  \n\t\n", vec![])]
    #[case("foo", vec![Column::Custom("foo".to_string())])]
    #[case("id", vec![Column::Id])]
    #[case("ID", vec![Column::Id])]
    #[case("createdby", vec![Column::CreatedBy])]
    #[case("Created By", vec![Column::CreatedBy])]
    #[case("  status  \n\n", vec![Column::Status])]
    #[case("#id\nid\n# title", vec![Column::Id])]
    #[case(
        "status\n\n# prio\npriority\nfoo-bar\n  baz  \n# ignored",
        vec![
            Column::Status,
            Column::Priority,
            Column::Custom("foo-bar".to_string()),
            Column::Custom("baz".to_string())
        ]
    )]
    fn test_columns_from_edited(#[case] input: &str, #[case] expected: Vec<Column>) {
        assert_eq!(IssueTableState::columns_from_edited(input), expected);
    }
}
