use ratatui::prelude::*;

use crate::{common::Page, feat::tui_issue_list::IssueListDraw, App};

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.tuistate.page {
            Page::IssueList => IssueListDraw::render(self, area, buf),
        }
    }
}
