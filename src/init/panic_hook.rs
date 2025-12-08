use std::{
    io::{self, stdout},
    panic::{set_hook, take_hook},
};

use ratatui::crossterm::{
    execute,
    terminal::{LeaveAlternateScreen, disable_raw_mode},
};

/// Initializes a panic hook that restores the terminal UI before propagating a panic.
///
/// This function sets up a custom panic handler that ensures the terminal is properly
/// restored to its original state (leaving alternate screen and disabling raw mode)
/// before the panic is propagated to the previous hook. This prevents the terminal
/// from being left in an unusable state when a panic occurs.
pub fn init() {
    let original_hook = take_hook();
    set_hook(Box::new(move |panic_info| {
        // intentionally ignore errors here since we're already in a panic
        let _ = restore_tui();
        original_hook(panic_info);
    }));
}

/// Restores the terminal UI to its original state by disabling raw mode and leaving the alternate screen.
///
/// Returns an error if either disabling raw mode or leaving the alternate screen fails.
pub fn restore_tui() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
}
