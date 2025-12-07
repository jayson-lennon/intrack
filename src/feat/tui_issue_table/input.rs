use error_stack::{Report, ResultExt};
use wherror::Error;

use crate::{
    App,
    feat::{
        issues::IssueEvent,
        tui::{Event, EventExt, EventPropagation, Focus, KeyCode, KeyModifiers},
        tui_issue_table::SortDirection,
    },
};

/// Error type for issue table page input handling operations.
///
/// This error is emitted when input handling operations fail, such as
/// when logging issue events to the event log.
#[derive(Debug, Error)]
#[error(debug)]
pub struct IssueTablePageInputError;

/// Handles keyboard input events for the issue table interface.
///
/// Implementers of this trait define how keyboard events should be processed
/// and whether they should continue propagating through the event system.
pub trait IssueTablePageInput {
    /// Processes a keyboard input event for the issue table.
    ///
    /// This method is called when keyboard events occur in the issue table
    /// interface. Implementations can handle specific key combinations and
    /// return whether the event should continue propagating to other handlers.
    ///
    /// # Errors
    ///
    /// Returns an error if the input handling operation fails, such as when
    /// logging issue events encounters an error.
    fn handle(
        &mut self,
        event: &Event,
    ) -> Result<EventPropagation, Report<IssueTablePageInputError>>;
}

impl IssueTablePageInput for App {
    /// Handles keyboard input events for the application's issue table.
    ///
    /// This implementation manages keyboard shortcuts for the issue table interface,
    /// including sorting, filtering, navigation, and issue status management.
    ///
    /// The handler operates differently based on the current UI focus:
    /// - When focused on the issue table, it handles sorting, navigation, and editing commands
    /// - When focused on the filter input, it delegates events to the filter input handler
    /// - For other focus states, it returns focus to the issue table
    ///
    /// # Key Bindings
    ///
    /// - **Shift+C**: Open external editor to modify column configuration
    /// - **Shift+J**: Sort table in descending order
    /// - **Shift+K**: Sort table in ascending order
    /// - **Shift+L**: Sort by next column
    /// - **Shift+H**: Sort by previous column
    /// - **Alt+S**: Toggle status for selected issues
    /// - **Down/J**: Move cursor to next item
    /// - **Up/K**: Move cursor to previous item
    /// - **/**: Focus the search filter input box
    ///
    /// # Errors
    ///
    /// Returns an error when failing to log issue events to the event log
    /// (specifically when toggling issue status).
    #[allow(clippy::too_many_lines)]
    fn handle(
        &mut self,
        event: &Event,
    ) -> Result<EventPropagation, Report<IssueTablePageInputError>> {
        match self.tuistate.focus() {
            Focus::IssueTable => {
                if let (Some(key), mods) = (event.keypress(), event.modifiers()) {
                    match (key, mods) {
                        // Edit columns
                        (KeyCode::Char('C' | 'c'), Some(mods))
                            if mods.contains(KeyModifiers::SHIFT) =>
                        {
                            self.external_editor.edit("testing", "", |app, response| {
                                app.config.tick_rate = 9.9;
                                tracing::error!(response = response, "got response");
                            });
                            return Ok(EventPropagation::Stop);
                        }
                        // Sort descending
                        (KeyCode::Char('J' | 'j'), Some(mods))
                            if mods.contains(KeyModifiers::SHIFT) =>
                        {
                            self.tuistate
                                .issue_table
                                .set_sort_direction(SortDirection::Descending);
                            return Ok(EventPropagation::Stop);
                        }
                        // Sort ascending
                        (KeyCode::Char('K' | 'k'), Some(mods))
                            if mods.contains(KeyModifiers::SHIFT) =>
                        {
                            self.tuistate
                                .issue_table
                                .set_sort_direction(SortDirection::Ascending);
                            return Ok(EventPropagation::Stop);
                        }
                        // Sort next column
                        (KeyCode::Char('L' | 'l'), Some(mods))
                            if mods.contains(KeyModifiers::SHIFT) =>
                        {
                            self.tuistate.issue_table.sort_next_column();
                            return Ok(EventPropagation::Stop);
                        }
                        // Sort previous column
                        (KeyCode::Char('H' | 'h'), Some(mods))
                            if mods.contains(KeyModifiers::SHIFT) =>
                        {
                            self.tuistate.issue_table.sort_previous_column();
                            return Ok(EventPropagation::Stop);
                        }
                        // Toggle status line
                        (KeyCode::Char('s'), Some(mods)) if mods.contains(KeyModifiers::ALT) => {
                            let indices = self.tuistate.issue_table.selected();
                            if indices.is_empty() {
                                return Ok(EventPropagation::Stop);
                            }

                            for index in indices {
                                let event = {
                                    let issue = self
                                        .issues
                                        .get_issue(&self.tuistate.issue_table.display_map[&index])
                                        .unwrap();
                                    let issue_id = issue.id;
                                    let status = issue.status.invert();
                                    IssueEvent::StatusChanged { issue_id, status }
                                };
                                self.issues
                                    .append_to_log(&self.args.event_log, &event)
                                    .change_context(IssueTablePageInputError)?;
                            }
                            return Ok(EventPropagation::Stop);
                        }
                        // Cursor to next item
                        (KeyCode::Down | KeyCode::Char('j'), _) => {
                            self.tuistate.issue_table.cursor_next();
                            return Ok(EventPropagation::Stop);
                        }
                        // Cursor to previous item
                        (KeyCode::Up | KeyCode::Char('k'), _) => {
                            self.tuistate.issue_table.cursor_previous();
                            return Ok(EventPropagation::Stop);
                        }
                        // Focus search filter box
                        (KeyCode::Char('/'), _) => {
                            if event.is_char('/') {
                                self.tuistate.set_focus(Focus::IssueTableFilter);
                                self.tuistate
                                    .issue_table
                                    .filter_input_state_mut()
                                    .set_focused(true);
                                return Ok(EventPropagation::Stop);
                            }
                        }
                        _ => (),
                    }
                }
            }
            Focus::IssueTableFilter => {
                if let (Some(key), mods) = (event.keypress(), event.modifiers()) {
                    #[allow(clippy::single_match)]
                    match (key, mods) {
                        // Move focus back to input list.
                        (KeyCode::Esc | KeyCode::Enter, _) => {
                            self.tuistate.set_focus(Focus::IssueTable);
                            self.tuistate
                                .issue_table
                                .filter_input_state_mut()
                                .set_focused(false);
                            return Ok(EventPropagation::Stop);
                        }
                        _ => (),
                    }
                }
                // Delegate event to the input box.
                return Ok(self
                    .tuistate
                    .issue_table
                    .filter_input_state_mut()
                    .handle_input(event));
            }
            _ => {
                self.tuistate.set_focus(Focus::IssueTable);
                return Ok(EventPropagation::Stop);
            }
        }
        Ok(EventPropagation::Continue)
    }
}
