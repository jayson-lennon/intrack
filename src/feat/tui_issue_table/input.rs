use crate::{
    App,
    feat::{
        tui::{Event, EventExt, EventPropagation, Focus, KeyCode, KeyModifiers},
        tui_issue_table::SortDirection,
    },
};

pub trait IssueListPageInput {
    fn handle(&mut self, event: &Event) -> EventPropagation;
}

impl IssueListPageInput for App {
    fn handle(&mut self, event: &Event) -> EventPropagation {
        match self.tuistate.focus() {
            Focus::IssueList => {
                if let (Some(key), mods) = (event.keypress(), event.modifiers()) {
                    match (key, mods) {
                        // Sort descending
                        (KeyCode::Char('J' | 'j'), Some(mods))
                            if mods.contains(KeyModifiers::SHIFT) =>
                        {
                            self.tuistate
                                .issue_table
                                .set_sort_direction(SortDirection::Descending);
                            return EventPropagation::Stop;
                        }
                        // Sort ascending
                        (KeyCode::Char('K' | 'k'), Some(mods))
                            if mods.contains(KeyModifiers::SHIFT) =>
                        {
                            self.tuistate
                                .issue_table
                                .set_sort_direction(SortDirection::Ascending);
                            return EventPropagation::Stop;
                        }
                        // Sort next column
                        (KeyCode::Char('L' | 'l'), Some(mods))
                            if mods.contains(KeyModifiers::SHIFT) =>
                        {
                            self.tuistate.issue_table.sort_next_column();
                            return EventPropagation::Stop;
                        }
                        // Sort previous column
                        (KeyCode::Char('H' | 'h'), Some(mods))
                            if mods.contains(KeyModifiers::SHIFT) =>
                        {
                            self.tuistate.issue_table.sort_previous_column();
                            return EventPropagation::Stop;
                        }
                        // Cursor to next item
                        (KeyCode::Down | KeyCode::Char('j'), _) => {
                            self.tuistate.issue_table.cursor_next();
                            return EventPropagation::Stop;
                        }
                        // Cursor to previous item
                        (KeyCode::Up | KeyCode::Char('k'), _) => {
                            self.tuistate.issue_table.cursor_previous();
                            return EventPropagation::Stop;
                        }
                        // Focus search filter box
                        (KeyCode::Char('/'), _) => {
                            if event.is_char('/') {
                                self.tuistate.set_focus(Focus::IssueListFilter);
                                self.tuistate
                                    .issue_table
                                    .filter_input_state_mut()
                                    .set_focused(true);
                                return EventPropagation::Stop;
                            }
                        }
                        _ => (),
                    }
                }
            }
            Focus::IssueListFilter => {
                if let (Some(key), mods) = (event.keypress(), event.modifiers()) {
                    #[allow(clippy::single_match)]
                    match (key, mods) {
                        // Move focus back to input list.
                        (KeyCode::Esc | KeyCode::Enter, _) => {
                            self.tuistate.set_focus(Focus::IssueList);
                            self.tuistate
                                .issue_table
                                .filter_input_state_mut()
                                .set_focused(false);
                            return EventPropagation::Stop;
                        }
                        _ => (),
                    }
                }
                // Delegate event to the input box.
                return self
                    .tuistate
                    .issue_table
                    .filter_input_state_mut()
                    .handle_input(event);
            }
            _ => {
                self.tuistate.set_focus(Focus::IssueList);
                return EventPropagation::Stop;
            }
        }
        EventPropagation::Continue
    }
}
