use crate::feat::{
    tui::{Focus, Page},
    tui_issue_table::IssueTableState,
};

/// State of the TUI.
#[derive(Debug, Default)]
pub struct TuiState {
    page: Page,
    focus: Focus,

    pub issue_table: IssueTableState,
}

impl TuiState {
    /// Returns the current focus.
    pub fn focus(&self) -> Focus {
        self.focus
    }

    /// Set the focus.
    pub fn set_focus(&mut self, focus: Focus) {
        self.focus = focus;
    }

    /// Returns the current page.
    pub fn page(&self) -> Page {
        self.page
    }

    /// Set the page.
    pub fn set_page(&mut self, page: Page) {
        self.page = page;
    }
}
