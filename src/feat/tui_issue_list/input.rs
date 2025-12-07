use error_stack::Report;

use crate::{
    App,
    app::EventHandlerError,
    feat::tui::{Event, EventExt, EventPropagation, Focus, KeyCode},
};

pub trait IssueListPageInput {
    fn handle(self, event: &Event) -> Result<EventPropagation, Report<EventHandlerError>>;
}

impl IssueListPageInput for &mut App {
    fn handle(self, event: &Event) -> Result<EventPropagation, Report<EventHandlerError>> {
        match self.tuistate.focus() {
            Focus::IssueList => {
                if event.is_char('/') {
                    self.tuistate.set_focus(Focus::IssueListFilter);
                    self.tuistate.issue_list.filter.set_focused(true);
                    Ok(EventPropagation::Stop)
                } else {
                    Ok(EventPropagation::Continue)
                }
            }
            Focus::IssueListFilter => {
                if let Event::Key(key) = event
                    && key.code == KeyCode::Esc
                {
                    self.tuistate.set_focus(Focus::IssueList);
                    self.tuistate.issue_list.filter.set_focused(false);
                    Ok(EventPropagation::Stop)
                } else {
                    Ok(self.tuistate.issue_list.filter.handle_input(event))
                }
            }
        }
    }
}
