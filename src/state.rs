use crate::feat::{cli::CliArgs, issues::Issues};

#[derive(Debug)]
pub struct AppState {
    pub issues: Issues,
    pub args: CliArgs,
}
