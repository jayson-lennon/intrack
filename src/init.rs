use clap_verbosity_flag::{Verbosity, WarnLevel};
use error_stack::{Report, ResultExt};
use std::env;
use std::path::Path;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};
use wherror::Error;

pub fn setup_tracing(verbosity: Verbosity<WarnLevel>) {
    let filter = match env::var("RUST_LOG") {
        // Use RUST_LOG if found
        Ok(filter_str) => filter_str,
        // Otherwise use this fallback
        Err(_) => format!("intrack={verbosity}"),
    };

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(EnvFilter::new(filter)))
        .init();
}

#[derive(Debug, Error)]
#[error(debug)]
pub struct CreateEventLogError;

pub fn create_event_log<P>(path: P) -> Result<(), Report<CreateEventLogError>>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let cwd = std::env::current_dir()
        .change_context(CreateEventLogError)
        .attach("failed to get current directory")?;
    let abs_log_path = if path.is_absolute() {
        path
    } else {
        &cwd.join(path)
    };
    let full_path_to_log = abs_log_path
        .parent()
        .ok_or(CreateEventLogError)
        .attach("log file path has no parent directory")?
        .to_path_buf();
    std::fs::create_dir_all(&full_path_to_log)
        .change_context(CreateEventLogError)
        .attach_with(|| {
            format!(
                "failed to create log directory {:?}",
                full_path_to_log.display()
            )
        })?;
    std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(path)
        .change_context(CreateEventLogError)
        .attach_with(|| format!("failed to initialize event log file {:?}", path.display()))?;
    Ok(())
}
