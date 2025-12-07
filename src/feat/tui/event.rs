pub type Event = crate::feat::tui::wrapper::Event;
pub type KeyEvent = crossterm::event::KeyEvent;
pub type KeyModifiers = crossterm::event::KeyModifiers;
pub type KeyCode = crossterm::event::KeyCode;
pub type KeyEventKind = crossterm::event::KeyEventKind;

/// Helper methods on [`Event`].
pub trait EventExt {
    /// Returns `true` if the event is a key press for the specified character.
    fn is_char(&self, ch: char) -> bool;
    /// Returns `true` if the event is a key press for the specified key code.
    fn is_code(&self, code: KeyCode) -> bool;
    /// Returns `Some` if the event is a keypress containing a keycode.
    fn keypress(&self) -> Option<KeyCode>;
    /// Returns the modifiers held during a keypress.
    fn modifiers(&self) -> Option<KeyModifiers>;
}

impl EventExt for Event {
    fn is_char(&self, ch: char) -> bool {
        self.is_code(KeyCode::Char(ch))
    }

    fn is_code(&self, code: KeyCode) -> bool {
        if let Event::Key(key) = self {
            key.kind == KeyEventKind::Press && key.code == code
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

    fn modifiers(&self) -> Option<KeyModifiers> {
        if let Event::Key(key) = self
            && key.kind == KeyEventKind::Press
        {
            Some(key.modifiers)
        } else {
            None
        }
    }
}
