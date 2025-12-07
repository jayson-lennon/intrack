use error_stack::{Report, ResultExt};
use wherror::Error;

use crate::{app::EventHandlerError, feat::tui::Event, App, AppError};

pub trait IssueListInput {
    fn handle(self, event: &Event) -> Result<(), Report<EventHandlerError>>;
}

impl IssueListInput for &mut App {
    fn handle(self, event: &Event) -> Result<(), Report<EventHandlerError>> {
        Err(EventHandlerError).attach("todo: issue list input handler")
    }
}
