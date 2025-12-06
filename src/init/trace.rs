use clap_verbosity_flag::{Verbosity, WarnLevel};
use error_stack::{Report, ResultExt};
use std::env;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};
use wherror::Error;

/// Error type returned when tracing subscriber initialization fails.
///
/// This error is used when the tracing system cannot be properly initialized,
/// such as when failing to open or configure log files.
#[derive(Debug, Error)]
#[error(debug)]
pub struct TracingInitError;

/// Initializes the tracing subscriber with the specified verbosity level.
///
/// This function sets up the global tracing subscriber for the application. If a log file
/// is provided, tracing output will be written to that file in addition to stdout. If
/// the `RUST_LOG` environment variable is set, it takes precedence over the verbosity
/// parameter for filtering log output.
///
/// # Errors
///
/// Returns a `TracingInitError` if the log file cannot be opened or configured.
///
/// # Panics
///
/// Panics if called more than once or if another tracer has already been initialized.
pub fn init<P>(
    verbosity: Verbosity<WarnLevel>,
    file: Option<P>,
) -> Result<(), Report<TracingInitError>>
where
    P: AsRef<Path>,
{
    let filter = match env::var("RUST_LOG") {
        // Use RUST_LOG if found
        Ok(filter_str) => filter_str,
        // Otherwise use this fallback
        Err(_) => format!("intrack={verbosity}"),
    };

    match file {
        Some(path) => {
            let path = path.as_ref();
            let logfile = File::options()
                .create(true)
                .append(true)
                .open(path)
                .change_context(TracingInitError)
                .attach_with(|| format!("failed to open file '{}' for tracing", path.display()))?;

            let file_layer: Option<Box<dyn Layer<_> + Send + Sync + 'static>> = Some(
                tracing_subscriber::fmt::layer()
                    .with_file(true)
                    .with_line_number(true)
                    .with_target(true)
                    .with_writer(Arc::new(logfile))
                    .with_filter(EnvFilter::new(filter))
                    .boxed(),
            );
            tracing_subscriber::registry().with(file_layer).init();
        }
        None => {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer().with_filter(EnvFilter::new(filter)))
                .init();
        }
    }
    Ok(())
}
