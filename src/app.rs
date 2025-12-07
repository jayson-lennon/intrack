mod config;
mod dependencies;

pub use config::AppConfig;

use bon::Builder;
use error_stack::{Report, ResultExt};
use ratatui::Frame;
use wherror::Error;

use crate::feat::{
    cli::CliArgs,
    issues::Issues,
    tui::{Event, Tui, TuiState},
    tui_issue_list::IssueListDraw,
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
            should_quit: false,
        }
    }

    pub async fn run(&mut self) -> Result<(), Report<AppError>> {
        let mut tui = Tui::new()
            .change_context(AppError)?
            .with_tick_rate(self.config.tick_rate)
            .with_frame_rate(self.config.frame_rate);

        // Starts event handler, enters raw mode, enters alternate screen
        tui.enter().change_context(AppError)?;

        loop {
            // `tui.next().await` blocks till next event
            if let Some(ev) = tui.next().await {
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

        // Stops event handler, exits raw mode, exits alternate screen
        tui.exit().change_context(AppError)?;

        Ok(())
    }

    fn on_tick(&mut self) {}

    fn draw(&mut self, frame: &mut Frame) {
        use crate::feat::tui::Page;

        let area = frame.area();
        let buf = frame.buffer_mut();
        match self.tuistate.page() {
            Page::IssueList => IssueListDraw::render(self, area, buf),
        }
    }

    pub fn handle_event(&mut self, event: &Event) -> Result<(), Report<EventHandlerError>> {
        use crate::feat::tui::{Focus, KeyCode, Page};
        use crate::feat::tui_issue_list::IssueListPageInput;

        match event {
            Event::Key(key) if key.code == KeyCode::Char('q') => self.should_quit = true,
            _ => (),
        }

        match self.tuistate.page() {
            Page::IssueList => IssueListPageInput::handle(self, event)?,
        };

        Ok(())
    }
}
