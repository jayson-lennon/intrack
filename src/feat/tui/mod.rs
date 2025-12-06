// (1) issue list
//
//
//
//
//
//
//
//
//
//
//
//
// / Search >

// use error_stack::{Report, ResultExt};
// use skim::prelude::*;
// use wherror::Error;
//
// use crate::{
//     feat::{issue::Issue, issues::IssueEvent},
//     state::AppState,
// };
//
// #[derive(Debug, Error)]
// #[error(debug)]
// pub struct OutputHandleError;
//
// #[derive(Debug, Error)]
// #[error(debug)]
// pub struct RunSkimError;
//
// #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
// enum AppAction {
//     Continue,
//     Break,
// }
//
// pub fn run(mut state: AppState, skim_options: SkimOptions) -> Result<(), Report<RunSkimError>> {
//     loop {
//         let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) = unbounded();
//
//         drop(tx_item);
//
//         let skim_out = Skim::run_with(&skim_options, Some(rx_item))
//             .ok_or(RunSkimError)
//             .attach("a skim error occurred")?;
//
//         match handle_skim_output(&mut state, skim_out).change_context(RunSkimError)? {
//             AppAction::Continue => continue,
//             AppAction::Break => break,
//         }
//     }
//     Ok(())
// }
//
// fn handle_skim_output(
//     app: &mut AppState,
//     skim_out: SkimOutput,
// ) -> Result<AppAction, Report<OutputHandleError>> {
//     match skim_out.final_key {
//         Key::Alt('n') => {
//             let next_id = app.issues.next_issue_id();
//             if let Some((issue, comment)) = Issue::new(next_id).change_context(OutputHandleError)? {
//                 app.issues
//                     .append_to_log(&app.args.event_log, IssueEvent::IssueCreated(issue))
//                     .change_context(OutputHandleError)
//                     .attach("failed to append new issue to log")?;
//                 app.issues
//                     .append_to_log(&app.args.event_log, IssueEvent::CommentAdded(comment))
//                     .change_context(OutputHandleError)
//                     .attach("failed to append new issue comment to log")?;
//             }
//             Ok(AppAction::Continue)
//         }
//         other => {
//             dbg!(other);
//             Ok(AppAction::Break)
//         }
//     }
// }
