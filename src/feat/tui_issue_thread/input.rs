use error_stack::{Report, ResultExt};
use jiff::Timestamp;
use wherror::Error;

use crate::{
    App,
    feat::{
        external_editor::ExternalEditorError,
        issue::Comment,
        tui::{Event, EventExt, EventPropagation, Focus, KeyCode, KeyModifiers, Page},
    },
};

/// Error type for issue thread page input handling operations.
///
/// This error is returned when input handling operations fail, such as when
/// interacting with the external editor for adding comments or appending
/// comments to the event log.
#[derive(Debug, Error)]
#[error(debug)]
pub struct IssueThreadPageInputError;

/// Handles keyboard input events for the issue thread page.
///
/// Implementers of this trait process keyboard events when the issue thread
/// page is active. The handler interprets key presses and performs appropriate
/// actions such as navigation, toggling help, or adding comments.
pub trait IssueThreadPageInput {
    /// Process a keyboard event for the issue thread page.
    ///
    /// When the issue thread page is focused, this method handles keyboard
    /// input and returns whether event propagation should continue. The
    /// implementation supports navigation (cursor movement, page changes),
    /// help toggle, and comment creation.
    ///
    /// # Errors
    ///
    /// Returns an error if the input handling operation fails, such as when
    /// using the external editor or appending to the event log.
    fn handle(
        &mut self,
        event: &Event,
    ) -> Result<EventPropagation, Report<IssueThreadPageInputError>>;
}

/// Implementation of issue thread page input handling for the application.
///
/// This handler processes keyboard events when the issue thread page is focused.
/// It supports navigation, help toggle, and comment creation via external editor.
impl IssueThreadPageInput for App {
    /// Process keyboard events for the issue thread page.
    ///
    /// When the issue thread has focus, this handler processes the following keys:
    /// - `q` or `Esc`: Return to the issue table
    /// - `Down` or `j`: Move cursor down
    /// - `Up` or `k`: Move cursor up
    /// - `Ctrl+d`: Move cursor down by 10 items (page down)
    /// - `Ctrl+u`: Move cursor up by 10 items (page up)
    /// - `?`: Toggle help display
    /// - `a`: Add a new comment using the external editor
    ///
    /// If another page has focus, this handler will set focus to the issue thread
    /// page and stop event propagation.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The external editor operation fails when adding a comment
    /// - Appending the comment to the event log fails
    fn handle(
        &mut self,
        event: &Event,
    ) -> Result<EventPropagation, Report<IssueThreadPageInputError>> {
        match self.tuistate.focus() {
            Focus::IssueThread => {
                if let (Some(key), mods) = (event.keypress(), event.modifiers()) {
                    match (key, mods) {
                        // Back to issue table
                        (KeyCode::Char('q') | KeyCode::Esc, _) => {
                            self.tuistate.set_page(Page::IssueTable);
                            self.tuistate.set_focus(Focus::IssueTable);
                            self.tuistate.issue_thread.show_help = false;
                            return Ok(EventPropagation::Stop);
                        }
                        // Cursor down
                        (KeyCode::Down | KeyCode::Char('j'), _) => {
                            self.tuistate.issue_thread.cursor_next();
                            return Ok(EventPropagation::Stop);
                        }
                        // Cursor up
                        (KeyCode::Up | KeyCode::Char('k'), _) => {
                            self.tuistate.issue_thread.cursor_previous();
                            return Ok(EventPropagation::Stop);
                        }
                        // Cursor page down
                        (KeyCode::Char('d'), Some(mods))
                            if mods.contains(KeyModifiers::CONTROL) =>
                        {
                            self.tuistate.issue_thread.cursor_add(10);
                            return Ok(EventPropagation::Stop);
                        }
                        // Cursor page up
                        (KeyCode::Char('u'), Some(mods))
                            if mods.contains(KeyModifiers::CONTROL) =>
                        {
                            self.tuistate.issue_thread.cursor_sub(10);
                            return Ok(EventPropagation::Stop);
                        }
                        // Toggle help
                        (KeyCode::Char('?'), _) => {
                            self.tuistate.issue_thread.toggle_help();
                            return Ok(EventPropagation::Stop);
                        }
                        // Add new comment
                        (KeyCode::Char('a'), _) => {
                            let issue_id = self.tuistate.issue_thread.issue_id;
                            let template = "Enter comment here.\n\n";
                            self.external_editor
                                .edit(template, "txt", move |app, response| {
                                    if let Some(content) = response {
                                        let content = content.trim().to_string();
                                        if content.is_empty() {
                                            return Ok(());
                                        }
                                        let comment = Comment {
                                            parent_issue: issue_id,
                                            content,
                                            created: Timestamp::now(),
                                            created_by: "TODO: current user email or from config"
                                                .to_string(),
                                        };
                                        app.issues
                                            .append_to_log(&app.args.event_log, comment)
                                            .change_context(ExternalEditorError)?;
                                    }
                                    Ok(())
                                });
                            return Ok(EventPropagation::Stop);
                        }
                        _ => (),
                    }
                }
            }
            _ => {
                self.tuistate.set_focus(Focus::IssueThread);
                return Ok(EventPropagation::Stop);
            }
        }
        Ok(EventPropagation::Continue)
    }
}
