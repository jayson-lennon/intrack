mod config;

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

pub use config::AppConfig;

use bon::Builder;
use derive_more::Debug;
use error_stack::{Report, ResultExt};
use ratatui::Frame;
use wherror::Error;

use crate::feat::{
    cli::CliArgs,
    external_editor::{ExternalEditor, ExternalEditorEntry},
    issues::Issues,
    tui::{Event, Tui, TuiState},
    tui_issue_table::IssueTableDraw,
};

/// Top-level error type.
#[derive(Debug, Error)]
#[error(debug)]
pub struct AppError;

#[derive(Debug, Error)]
#[error(debug)]
pub struct EventHandlerError;

#[derive(Debug)]
pub struct App {
    pub issues: Issues,
    pub args: CliArgs,
    pub config: AppConfig,

    pub tuistate: TuiState,

    pub external_editor: ExternalEditor,

    should_quit: bool,
}

#[derive(Builder)]
pub struct AppNewArgs {
    pub args: CliArgs,
    #[builder(default)]
    pub config: AppConfig,
    #[builder(default)]
    pub issues: Issues,
    #[builder(default)]
    pub tuistate: TuiState,
}

impl App {
    pub fn new(setup: AppNewArgs) -> Self {
        Self {
            issues: setup.issues,
            args: setup.args,
            config: setup.config,
            tuistate: setup.tuistate,
            external_editor: ExternalEditor::default(),
            should_quit: false,
        }
    }

    /// Starts event handler, enters raw mode, enters alternate screen
    fn new_backend(config: &AppConfig) -> Result<Tui, Report<AppError>> {
        let mut tui = Tui::new()
            .change_context(AppError)?
            .with_tick_rate(config.tick_rate)
            .with_frame_rate(config.frame_rate);
        tui.enter_raw_mode()
            .change_context(AppError)
            .attach("failed to enter raw mode")?;
        tui.start()
            .change_context(AppError)
            .attach("failed to start event capture")?;
        Ok(tui)
    }

    /// Stops event handler, exits raw mode, exits alternate screen
    fn kill_backend(tui: Tui) {
        // dropping the backend automatically cleans up.
    }

    pub async fn run(&mut self) -> Result<(), Report<AppError>> {
        let mut tui = Self::new_backend(&self.config)?;

        loop {
            // `tui.next().await` blocks till next event
            if let Some(ev) = tui.next().await {
                if let Some(ExternalEditorEntry {
                    data: prompt,
                    file_extension,
                    callback,
                }) = self.external_editor.take()
                {
                    Self::kill_backend(tui);
                    let result = dialoguer::Editor::default()
                        .require_save(true)
                        .extension(&file_extension)
                        .edit(&prompt)
                        .change_context(AppError)
                        .attach("failed to gather content from external editor")?;
                    tui = Self::new_backend(&self.config)?;
                    callback(self, result);
                    tui.draw(|f| {
                        self.draw(f);
                    })
                    .change_context(AppError)?;
                }
                // Determine whether to render or tick
                match ev {
                    Event::Render | Event::Key(_) | Event::Mouse(_) | Event::Resize(_, _) => {
                        tui.draw(|f| {
                            self.draw(f);
                        })
                        .change_context(AppError)?;
                    }
                    Event::Tick => {
                        self.on_tick();
                    }
                    _ => (),
                }

                self.handle_event(&ev).change_context(AppError)?;
            }

            if self.should_quit {
                break;
            }
        }

        Self::kill_backend(tui);

        Ok(())
    }

    fn on_tick(&mut self) {}

    fn draw(&mut self, frame: &mut Frame) {
        use crate::feat::tui::Page;

        let area = frame.area();
        let buf = frame.buffer_mut();
        match self.tuistate.page() {
            Page::IssueTable => IssueTableDraw::render(self, area, buf),
        }
    }

    pub fn handle_event(&mut self, event: &Event) -> Result<(), Report<EventHandlerError>> {
        use crate::feat::tui::{EventPropagation, KeyCode, Page};
        use crate::feat::tui_issue_table::IssueListPageInput;

        // Match only on the page. The page input handler will manage the focus.
        let propagation = match self.tuistate.page() {
            Page::IssueTable => IssueListPageInput::handle(self, event),
        };

        match propagation {
            EventPropagation::Continue => {
                // Handle top-level keystrokes here
                match event {
                    Event::Key(key) if key.code == KeyCode::Char('q') => self.should_quit = true,
                    _ => (),
                }
            }
            EventPropagation::Stop => (),
        }

        Ok(())
    }
}
