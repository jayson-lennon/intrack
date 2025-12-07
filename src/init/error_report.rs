use error_stack::{
    fmt::{Charset, ColorMode},
    Report,
};
use owo_colors::OwoColorize;

use crate::common::report::{Missing, Suggestion};

pub fn init() {
    Report::set_color_mode(ColorMode::Color);

    Report::install_debug_hook::<Suggestion>(|Suggestion(value), context| {
        match context.charset() {
            Charset::Utf8 => context.push_body(format!("👉 {}: {value}", "suggestion".cyan())),
            Charset::Ascii => context.push_body(format!("{}: {value}", "suggestion".cyan())),
        }
    });

    Report::install_debug_hook::<Missing>(|Missing(value), context| match context.charset() {
        Charset::Utf8 => context.push_body(format!("❓ {}: {value}", "missing".red())),
        Charset::Ascii => context.push_body(format!("{}: {value}", "missing".red())),
    });
}
