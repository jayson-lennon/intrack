mod draw;
mod input;

pub use draw::IssueListDraw;
pub use input::IssueListPageInput;
use ratatui::widgets::ListState;

use crate::feat::tui_widget::InputBoxState;

#[derive(Debug, Default)]
pub struct IssueListState {
    list: ListState,
    filter: InputBoxState,
}

impl IssueListState {
    fn select_previous_list_item(&mut self) {
        if let Some(current) = self.list.selected() {
            let index = current.saturating_sub(1);
            self.select_list_item(index);
        } else {
            self.select_list_item(0);
        }
    }

    fn select_next_list_item(&mut self) {
        if let Some(current) = self.list.selected() {
            let index = current.saturating_add(1);
            self.select_list_item(index);
        } else {
            self.select_list_item(0);
        }
    }
    fn select_list_item(&mut self, index: usize) {
        self.list.select(Some(index));
    }

    pub fn filter(&self) -> &InputBoxState {
        &self.filter
    }

    pub fn filter_mut(&mut self) -> &mut InputBoxState {
        &mut self.filter
    }
}
