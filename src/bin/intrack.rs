use clap::Parser;
use dotenvy::dotenv;
use error_stack::{Report, ResultExt};
use intrack::{
    feat::{cli::CliArgs, issues::Issues},
    state::AppState,
};
use skim::prelude::*;
use wherror::Error;

#[derive(Debug, Error)]
#[error(debug)]
pub struct AppError;

pub fn main() -> Result<(), Report<AppError>> {
    let args = CliArgs::parse();

    intrack::init::setup_tracing(args.verbosity);

    match dotenv() {
        Ok(_) => {}
        Err(dotenvy::Error::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => {
            return Err(e)
                .change_context(AppError)
                .attach("error reacing .env file")?;
        }
    }

    let skim_options = SkimOptionsBuilder::default()
        .multi(false)
        .bind(vec!["alt-n:abort".into()])
        .header_lines(1)
        .layout("reverse".to_string())
        .preview_window("down:40%".to_string())
        .color(Some(
            "dark,current_bg:#444400,current_match_bg:#000000,current_match:#ffff00".to_string(),
        ))
        .header(Some(
            "this is the permanent keybind line help thing".to_string(),
        ))
        .header_lines(1)
        .preview_fn(Some(PreviewCallback::from(|_| vec![])))
        .build()
        .unwrap();

    intrack::init::create_event_log(&args.event_log)
        .change_context(AppError)
        .attach("failed to create event log")?;

    let state = AppState {
        issues: {
            Issues::from_jsonl_file(&args.event_log)
                .change_context(AppError)
                .attach("failed to load issues")?
        },
        args,
    };

    intrack::feat::tui::run(state, skim_options)
        .change_context(AppError)
        .attach("error while running skim")?;

    Ok(())
}
