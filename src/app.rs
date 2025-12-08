mod config;

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

/// Top-level error type for the application.
///
/// This error represents any unrecoverable failure that occurs during application execution,
/// including TUI initialization failures, rendering errors, and event handling failures.
#[derive(Debug, Error)]
#[error(debug)]
pub struct AppError;

/// Error type for event handler failures.
///
/// This error indicates that an event could not be processed successfully, typically
/// due to invalid event data or state inconsistencies.
#[derive(Debug, Error)]
#[error(debug)]
pub struct EventHandlerError;

/// Main application struct managing the terminal UI and event loop.
///
/// The application coordinates between issues, configuration, TUI state, and external
/// editor integration to provide a rich interactive terminal interface for managing issues.
#[derive(Debug)]
pub struct App {
    pub issues: Issues,
    pub args: CliArgs,
    pub config: AppConfig,

    pub tuistate: TuiState,

    pub external_editor: ExternalEditor,

    should_quit: bool,
}

/// Arguments for creating a new application instance.
///
/// This builder struct configures the initial state of the application, including command-line
/// arguments, configuration, issues data, and TUI state. All fields have sensible defaults except
/// `args`, which is required.
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
    /// Creates a new application instance with the provided setup arguments.
    ///
    /// Initializes the application with the given issues, command-line arguments, configuration,
    /// and TUI state. Sets up an external editor instance and initializes the quit flag to false.
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

    /// Initializes a new TUI backend with the provided configuration.
    ///
    /// Creates and configures a terminal interface including event handler, tick rate,
    /// and frame rate. Enters raw mode to capture keyboard input directly and starts
    /// the event capture system. The returned TUI instance is ready to receive events.
    ///
    /// # Errors
    ///
    /// Returns an error if entering raw mode fails or starting the event capture fails.
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

    /// Tears down the TUI backend and restores terminal state.
    ///
    /// Stops the event handler, exits raw mode, and exits the alternate screen, returning the
    /// terminal to its original state. The backend is dropped, which automatically performs
    /// cleanup operations.
    ///
    /// # Notes
    ///
    /// Events will no longer be sent to the application until a new backend is created. The
    /// parameter is intentionally unused as dropping performs all necessary cleanup.
    fn kill_backend(_: Tui) {
        // dropping the backend automatically cleans up.
    }

    /// Runs the main application event loop.
    ///
    /// Initializes the TUI backend and enters a continuous loop that processes events, handles
    /// external editor interactions, and renders the UI. The loop continues until the quit flag is
    /// set, typically in response to user input.
    ///
    /// Events are processed in order: external editor checks, rendering for visual events, tick
    /// handling for timed events, and event handler processing.
    ///
    /// # Errors
    ///
    /// Returns an error if the TUI backend cannot be initialized, if rendering fails, or if event
    /// handling encounters an unrecoverable error.
    pub async fn run(&mut self) -> Result<(), Report<AppError>> {
        let mut tui = Self::new_backend(&self.config)?;

        loop {
            // `tui.next().await` blocks till next event
            if let Some(ev) = tui.next().await {
                tui = self.try_external_editor(tui)?;
                // Determine whether to render or tick
                match ev {
                    Event::Render | Event::Key(_) | Event::Mouse(_) | Event::Resize(_, _) => {
                        tui.draw(|f| {
                            self.draw(f);
                        })
                        .change_context(AppError)?;
                    }
                    Event::Tick => {}
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

    /// Checks for and processes pending external editor requests.
    ///
    /// If an external editor entry is pending, this method temporarily tears down the TUI backend,
    /// launches the system's default editor with the provided data, waits for user input, then
    /// reinitializes the TUI backend and invokes the callback with the edited content. Returns the
    /// reinitialized TUI instance.
    ///
    /// # Errors
    ///
    /// Returns an error if the external editor cannot be launched, if content gathering fails, or
    /// if the TUI backend cannot be reinitialized.
    fn try_external_editor(&mut self, tui: Tui) -> Result<Tui, Report<AppError>> {
        let mut tui = tui;
        if let Some(ExternalEditorEntry {
            data,
            file_extension,
            callback,
        }) = self.external_editor.take()
        {
            Self::kill_backend(tui);

            let result = dialoguer::Editor::default()
                .require_save(true)
                .extension(&file_extension)
                .edit(&data)
                .change_context(AppError)
                .attach("failed to gather content from external editor")?;

            callback(self, result)
                .change_context(AppError)
                .attach("error handling input from external editor")?;

            tui = Self::new_backend(&self.config)?;

            tui.draw(|f| {
                self.draw(f);
            })
            .change_context(AppError)?;
        }
        Ok(tui)
    }

    /// Renders the application UI to the provided frame.
    ///
    /// Draws the current page's UI components based on the application's TUI state. The rendering
    /// is delegated to page-specific renderers.
    ///
    /// Uses the full frame area for rendering.
    fn draw(&mut self, frame: &mut Frame) {
        use crate::feat::tui::Page;

        let area = frame.area();
        let buf = frame.buffer_mut();
        match self.tuistate.page() {
            Page::IssueTable => IssueTableDraw::render(self, area, buf),
        }
    }

    /// Processes incoming events and updates application state.
    ///
    /// Handles keyboard, mouse, and other events by delegating to the current page's event
    /// handler. The page handler manages focus and page-specific interactions. After page
    /// processing, top-level keystrokes (like 'q' to quit) are handled.
    ///
    /// # Errors
    ///
    /// Returns an error if the event cannot be processed due to invalid state or event data.
    pub fn handle_event(&mut self, event: &Event) -> Result<(), Report<EventHandlerError>> {
        use crate::feat::tui::{EventPropagation, KeyCode, Page};
        use crate::feat::tui_issue_table::IssueTablePageInput;

        // Match only on the page. The page input handler will manage the focus.
        let propagation = match self.tuistate.page() {
            Page::IssueTable => {
                IssueTablePageInput::handle(self, event).change_context(EventHandlerError)?
            }
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
