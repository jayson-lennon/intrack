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

#[derive(Debug, Error)]
#[error(debug)]
pub struct IssueThreadPageInputError;

pub trait IssueThreadPageInput {
    fn handle(
        &mut self,
        event: &Event,
    ) -> Result<EventPropagation, Report<IssueThreadPageInputError>>;
}

impl IssueThreadPageInput for App {
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
