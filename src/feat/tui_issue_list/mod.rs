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
