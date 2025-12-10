use error_stack::{Report, ResultExt};
use wherror::Error;

/// Types that can be attached to error reports.
pub mod report {
    /// Show a suggestion.
    pub struct Suggestion(pub &'static str);

    /// Indicate that some item is missing.
    pub struct Missing(pub String);
}

/// Git user information containing information from [`git_user_info`].
///
/// This struct holds the git configuration values retrieved from the global git configuration.
#[derive(Debug, Clone)]
pub struct GitUserInfo {
    pub name: String,
    pub email: String,
}

/// Error type for git configuration queries.
///
/// This error is returned when git user information cannot be retrieved,
/// such as when the git configuration cannot be read or required keys are missing.
#[derive(Debug, Error)]
#[error(debug)]
pub struct GitQueryError;

/// Retrieves git user information from the global git configuration.
///
/// Reads the user.name and user.email values from the global git configuration
/// and returns them as a GitUserInfo struct.
///
/// # Errors
///
/// Returns an error if:
/// - The git configuration cannot be read
/// - The 'user.name' key is not present in the configuration
/// - The 'user.email' key is not present in the configuration
pub fn git_user_info() -> Result<GitUserInfo, Report<GitQueryError>> {
    let git = gix_config::File::from_globals()
        .change_context(GitQueryError)
        .attach("failed to read git configuration")?;
    let name = git
        .string("user.name")
        .ok_or(GitQueryError)
        .attach("'name' key not present in git config")?;
    let email = git
        .string("user.email")
        .ok_or(GitQueryError)
        .attach("'email' key not present in git config")?;
    Ok(GitUserInfo {
        name: name.to_string(),
        email: email.to_string(),
    })
}
