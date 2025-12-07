pub type Event = crate::feat::tui::wrapper::Event;
pub type KeyEvent = crossterm::event::KeyEvent;
pub type KeyCode = crossterm::event::KeyCode;
pub type KeyEventKind = crossterm::event::KeyEventKind;

/// Helper methods on [`Event`](crate::feat::tui::Event).
pub trait EventExt {
    /// Returns `true` if the event is a key press for the specified character.
    fn is_char(&self, ch: char) -> bool;
    /// Returns `Some` if the event is a keypress containing a keycode.
    fn keypress(&self) -> Option<KeyCode>;
}

impl EventExt for Event {
    fn is_char(&self, ch: char) -> bool {
        if let Event::Key(key) = self {
            key.kind == KeyEventKind::Press && key.code == KeyCode::Char(ch)
        } else {
            false
        }
    }
    fn keypress(&self) -> Option<KeyCode> {
        if let Event::Key(key) = self
            && key.kind == KeyEventKind::Press
        {
            Some(key.code)
        } else {
            None
        }
    }
}
