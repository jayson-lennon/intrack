use crate::{
    App,
    feat::tui::{Event, EventExt, EventPropagation, Focus, KeyCode},
};

pub trait IssueListPageInput {
    fn handle(&mut self, event: &Event) -> EventPropagation;
}

impl IssueListPageInput for App {
    fn handle(&mut self, event: &Event) -> EventPropagation {
        match self.tuistate.focus() {
            Focus::IssueList => {
                if let Some(key) = event.keypress() {
                    match key {
                        KeyCode::Up => {
                            self.tuistate.issue_list.select_previous_list_item();
                            EventPropagation::Stop
                        }
                        KeyCode::Down => {
                            self.tuistate.issue_list.select_next_list_item();
                            EventPropagation::Stop
                        }
                        KeyCode::Char('/') => {
                            if event.is_char('/') {
                                self.tuistate.set_focus(Focus::IssueListFilter);
                                self.tuistate.issue_list.filter_mut().set_focused(true);
                                EventPropagation::Stop
                            } else {
                                EventPropagation::Continue
                            }
                        }
                        _ => EventPropagation::Continue,
                    }
                } else {
                    EventPropagation::Continue
                }
            }
            Focus::IssueListFilter => {
                if let Event::Key(key) = event
                    && key.code == KeyCode::Esc
                {
                    self.tuistate.set_focus(Focus::IssueList);
                    self.tuistate.issue_list.filter_mut().set_focused(false);
                    EventPropagation::Stop
                } else {
                    self.tuistate.issue_list.filter_mut().handle_input(event)
                }
            }
            _ => {
                self.tuistate.set_focus(Focus::IssueList);
                EventPropagation::Stop
            }
        }
    }
}
