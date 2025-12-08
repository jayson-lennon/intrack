use tui_widget_list::ListState;

use crate::feat::issue::IssueId;

#[derive(Debug, Default)]
pub struct IssueThreadState {
    pub(in crate::feat::tui_issue_thread) list_state: ListState,
    pub(in crate::feat::tui_issue_thread) issue_id: IssueId,
    pub(in crate::feat::tui_issue_thread) show_help: bool,
}

impl IssueThreadState {
    pub fn selected(&self) -> Option<usize> {
        self.list_state.selected
    }

    pub fn cursor_add(&mut self, amount: usize) {
        let current = self.list_state.selected.unwrap_or(0);
        let next = current.saturating_add(amount);
        self.list_state.select(Some(next));
    }

    pub fn cursor_sub(&mut self, amount: usize) {
        let current = self.list_state.selected.unwrap_or(0);
        let next = current.saturating_sub(amount);
        self.list_state.select(Some(next));
    }

    pub fn cursor_next(&mut self) {
        let current = self.list_state.selected.unwrap_or(0);
        let next = current.saturating_add(1);
        self.list_state.select(Some(next));
    }

    pub fn cursor_previous(&mut self) {
        let current = self.list_state.selected.unwrap_or(0);
        let prev = current.saturating_sub(1);
        self.list_state.select(Some(prev));
    }

    pub fn set_issue_id(&mut self, issue_id: IssueId) {
        self.issue_id = issue_id;
        self.list_state.select(Some(0));
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }
}
