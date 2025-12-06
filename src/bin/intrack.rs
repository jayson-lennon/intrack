use clap::Parser;
use dotenvy::dotenv;
use error_stack::{Report, ResultExt};
use intrack::{
    feat::{cli::CliArgs, issues::Issues},
    state::AppState,
};
use wherror::Error;

/// A simple application error type used throughout the intrack application.
///
/// This error type is used as the context for all errors that occur during
/// application execution, providing a unified error reporting mechanism.
#[derive(Debug, Error)]
#[error(debug)]
pub struct AppError;

/// Executes the intrack application entry point.
///
/// Initializes the application by parsing command-line arguments, setting up
/// logging and tracing, loading environment variables, creating the event log,
/// and loading issues data. Returns `Ok(())` on successful initialization,
/// or `Err` with `AppError` if any initialization step fails.
///
/// # Errors
///
/// Returns `Err` when:
/// - Loading the .env file fails (excluding file not found)
/// - Creating the event log fails
/// - Loading issues from the event log file fails
///
pub fn main() -> Result<(), Report<AppError>> {
    let args = CliArgs::parse();

    intrack::init::trace::init(args.verbosity, args.tracing_log.clone())
        .change_context(AppError)
        .attach("failed to initialize tracer")?;

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
