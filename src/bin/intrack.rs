use clap::Parser;
use dotenvy::dotenv;
use error_stack::{Report, ResultExt};
use intrack::{
    feat::{cli::CliArgs, issues::Issues},
    state::AppState,
};
use wherror::Error;

#[derive(Debug, Error)]
#[error(debug)]
pub struct AppError;

pub fn main() -> Result<(), Report<AppError>> {
    let args = CliArgs::parse();

    intrack::init::trace::init(args.verbosity);

    match dotenv() {
        Ok(_) => {}
        Err(dotenvy::Error::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => {
            return Err(e)
                .change_context(AppError)
                .attach("error reacing .env file")?;
        }
    }

    intrack::init::event_log::create(&args.event_log)
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

    Ok(())
}
