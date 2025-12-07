use crossterm::event::{KeyCode, KeyEvent};
use error_stack::{Report, ResultExt};
use wherror::Error;

use crate::{
    common::Focus,
    feat::{tui::Event, tui_issue_list::IssueListInput},
    App, AppError,
};

#[derive(Debug, Error)]
#[error(debug)]
pub struct EventHandlerError;

impl App {
    pub(super) fn handle_event(&mut self, event: &Event) -> Result<(), Report<EventHandlerError>> {
        match event {
            Event::Key(key) if key.code == KeyCode::Char('q') => self.should_quit = true,
            _ => (),
        }

        match self.tuistate.focus {
            Focus::IssueList => IssueListInput::handle(self, event)?,
            Focus::IssueListFilter => (),
        }

        Ok(())
        // for layer in self.input_stack.clone().iter().rev() {
        //     use feat::tabs::Tab;
        //
        //     #[rustfmt::skip]
        //     let response = match (layer,self.ui_state.tabs.current()) {
        //         // System-level input handling
        //         (InputLayer::Quit, _) => {
        //             InputLayerResponse::handle_if(|| event.is_char('q'))
        //                 .then(|| { self.should_quit = true; })
        //         }
        //         (InputLayer::Tabs(layer), _) => feat::tabs::ui::handle_event(self, layer, event)?,
        //
        //         // Tab-specific handling
        //         (InputLayer::EquipSearch(layer), Tab::EquipSearch) => feat::equip_query::ui::handle_event(self, layer, event)?,
        //         (InputLayer::MobLvlSearch(layer), Tab::MobLevelSearch) => feat::mob_lvl_search::ui::handle_event(self, layer, event)?,
        //
        //         _ => InputLayerResponse::Unhandled
        //     };
        //
        //     match response {
        //         InputLayerResponse::Handled => {
        //             break;
        //         }
        //         InputLayerResponse::HandledAndThen(action) => {
        //             match action {
        //                 InputLayerAction::Push(layer) => {
        //                     self.input_stack.push(layer);
        //                 }
        //                 InputLayerAction::Pop => {
        //                     self.input_stack.pop();
        //                 }
        //             }
        //
        //             break;
        //         }
        //         InputLayerResponse::HandleMultiple(actions) => {
        //             for action in actions {
        //                 match action {
        //                     InputLayerAction::Push(layer) => {
        //                         self.input_stack.push(layer);
        //                     }
        //                     InputLayerAction::Pop => {
        //                         self.input_stack.pop();
        //                     }
        //                 }
        //             }
        //         }
        //         InputLayerResponse::Unhandled | InputLayerResponse::Ignored => continue,
        //     }
        // }
        // Ok(())
    }
}
