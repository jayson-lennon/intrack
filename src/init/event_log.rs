use error_stack::{Report, ResultExt};
use std::path::Path;
use wherror::Error;

/// Error type representing failures that can occur when creating an event log file.
///
/// This error is returned when operations related to event log initialization fail,
/// such as unable to get the current directory, create necessary directories, or
/// open/create the log file.
#[derive(Debug, Error)]
#[error(debug)]
pub struct CreateEventLogError;

/// Creates and initializes an event log file at the specified path.
///
/// This function ensures that the event log file exists and can be written to by
/// creating all necessary parent directories and opening the file with write access.
/// If the file already exists, its contents are preserved.
///
/// # Arguments
///
/// * `path` - The path where the event log file should be created. Can be either
///   an absolute path or a relative path (relative to the current working directory).
///
/// # Errors
///
/// Returns an error if:
/// - The current working directory cannot be determined
/// - The log file path has no parent directory (e.g., is a bare filename without directory)
/// - Parent directories cannot be created
/// - The log file cannot be opened or created
///
pub fn create<P>(path: P) -> Result<(), Report<CreateEventLogError>>
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
