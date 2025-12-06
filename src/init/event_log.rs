use error_stack::{Report, ResultExt};
use std::path::Path;
use wherror::Error;

#[derive(Debug, Error)]
#[error(debug)]
pub struct CreateEventLogError;

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
