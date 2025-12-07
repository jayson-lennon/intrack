use std::time::Duration;

use clap::Parser;
use dotenvy::dotenv;
use error_stack::{Report, ResultExt};
use intrack::{
    common::report::{Missing, Suggestion},
    feat::{cli::CliArgs, issues::Issues, tui::TuiState},
    App, AppConfig, AppError, AppNewArgs,
};

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
#[tokio::main]
pub async fn main() -> Result<(), Report<AppError>> {
    intrack::init::error_report::init();

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

    let args = AppNewArgs::builder()
        .issues(
            Issues::from_jsonl_file(&args.event_log)
                .change_context(AppError)
                .attach("failed to load issues")?,
        )
        .args(args)
        .build();

    let mut app = App::new(args);
    app.run().await?;

    Ok(())
}
