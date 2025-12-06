use std::path::PathBuf;

use clap::Parser;
use clap_verbosity_flag::{Verbosity, WarnLevel};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Path to the JSONL event log file.
    #[arg(short = 'f', long = "file", default_value = "issues.jsonl")]
    pub event_log: PathBuf,

    #[command(flatten)]
    pub verbosity: Verbosity<WarnLevel>,
}
