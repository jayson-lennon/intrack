use std::borrow::Cow;

use ratatui::{layout::Rect, prelude::*, widgets::Widget};
use ropey::Rope;
use wherror::Error;

use crate::feat::tui::{Event, EventExt, EventPropagation, KeyCode};

/// Error type for input box operations.
///
/// This error is returned when input box operations fail. It uses debug formatting
/// to provide detailed error information for debugging purposes.
#[derive(Debug, Error)]
#[error(debug)]
pub struct InputBoxError;

/// State container for the input box widget.
///
/// This struct holds the current text content, cursor position, and focus state
/// of an input box. It manages the internal state required for text input and
/// cursor manipulation.
#[derive(Clone, Debug, Default)]
pub struct InputBoxState {
    text: Rope,
    cursor: usize,
    is_focused: bool,
}

impl InputBoxState {
    /// Handles keyboard input events for the input box.
    ///
    /// Processes character input, backspace, and cursor movement keys.
    /// Returns `EventPropagation::Stop` when the event is handled by this input box,
    /// or `EventPropagation::Continue` when the event should be passed to other handlers.
    ///
    /// - Character input inserts the character at the cursor position and advances the cursor
    /// - Backspace deletes the character before the cursor (if any)
    /// - Left/Right arrows move the cursor within the text bounds
    pub fn handle_input(&mut self, event: &Event) -> EventPropagation {
        if let Some(key) = event.keypress() {
            match key {
                KeyCode::Char(ch) => {
                    self.text.insert_char(self.cursor, ch);
                    self.cursor += 1;
                    return EventPropagation::Stop;
                }
                KeyCode::Backspace => {
                    let text_len = self.text.len_chars();

                    if text_len > 0 {
                        self.text.remove(self.cursor.saturating_sub(1)..self.cursor);
                        self.cursor = self.cursor.saturating_sub(1);
                    }
                    return EventPropagation::Stop;
                }
                KeyCode::Left => {
                    self.cursor = self.cursor.saturating_sub(1);
                    return EventPropagation::Stop;
                }
                KeyCode::Right => {
                    if self.cursor < self.text.len_chars() {
                        self.cursor += 1;
                    }
                    return EventPropagation::Stop;
                }
                _ => (),
            }
        }
        EventPropagation::Continue
    }

    /// Sets the focus state of the input box.
    ///
    /// When focused, the input box will respond to keyboard input and display
    /// visual focus indicators. When not focused, it ignores keyboard input.
    pub fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    /// Returns the current text content of the input box.
    ///
    /// Returns the text as a `Cow` to allow zero-copy conversions while also
    /// supporting owned string operations when needed.
    pub fn text(&self) -> Cow<'_, str> {
        self.text.line(0).into()
    }
}

/// A stateful text input widget for TUI applications.
///
/// This widget provides a text input field that can display a prefix (such as a prompt
/// or indicator) followed by editable text. It renders with visual feedback for focus
/// state and cursor position.
///
/// # Examples
///
/// ```
/// use ratatui::widgets::StatefulWidget;
/// use ratatui::prelude::*;
/// # use intrack::feat::tui_widget::InputBox;
/// # use intrack::feat::tui_widget::InputBoxState;
/// #
/// # fn example() {
/// let mut state = InputBoxState::default();
/// let widget = InputBox::new()
///     .with_prefix(vec![Span::raw("> ")]);
/// # }
/// ```
#[derive(Clone, Debug, Default)]
pub struct InputBox<'a> {
    pub prefix: Vec<Span<'a>>,
}

impl<'a> InputBox<'a> {
    /// Creates a new empty input box with no prefix.
    pub fn new() -> Self {
        Self { prefix: vec![] }
    }

    /// Sets the prefix for this input box.
    ///
    /// The prefix is displayed before the editable text area and can contain
    /// styled text (such as prompts or indicators).
    #[must_use]
    pub fn with_prefix(mut self, prefix: Vec<Span<'a>>) -> Self {
        self.prefix = prefix;
        self
    }
}

/// Renders the input box widget with the current state.
///
/// This implementation handles rendering the prefix, text content, and cursor
/// with appropriate styling based on the focus state. The cursor is displayed
/// as a reversed character when focused, and the entire input box is highlighted
/// when focused.
impl StatefulWidget for InputBox<'_> {
    type State = InputBoxState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let is_focused = state.is_focused;

        // Define the cursor's appearance.
        let cursor_style = Style::default().add_modifier(Modifier::REVERSED);

        // Generate the styled line with the visible cursor.
        let mut input_line =
            format_text_with_cursor(&state.text, state.cursor, cursor_style, is_focused);

        // Create the query indicator
        let mut full_line = self.prefix;

        // Merge the indicator with the user's text
        let full_line = {
            full_line.append(&mut input_line);

            Line::from(full_line).style(apply_focus_highlight(is_focused))
        };

        Widget::render(full_line, area, buf);
    }
}

/// Applies visual styling based on the focus state.
///
/// When focused, the input box uses a yellow background with black text
/// for clear visual feedback. When not focused, no special styling is applied.
fn apply_focus_highlight(is_focused: bool) -> Style {
    if is_focused {
        Style::default().fg(Color::Black).bg(Color::Yellow)
    } else {
        Style::default()
    }
}

/// Formats text with a visible cursor position.
///
/// This function takes text and a cursor position, then returns a vector of styled spans
/// representing the text with the cursor visually embedded. When the cursor is at the end
/// of the text or when the text is empty, it appends a styled space to represent the cursor.
/// When the cursor is in the middle, it splits the text and applies the cursor style to
/// the character at the cursor position.
///
/// The cursor is only visually styled when `is_focused` is true. When not focused,
/// the cursor style is neutral.
fn format_text_with_cursor(
    text: &Rope,
    cursor_pos: usize,
    cursor_style: Style,
    is_focused: bool,
) -> Vec<Span<'_>> {
    let mut cursor_style = cursor_style;
    // Clamp the cursor position to be within the valid range of the text's character length.
    let cursor_pos = cursor_pos.min(text.len_chars());
    if !is_focused {
        cursor_style = Style::default();
    }

    // Case 1: The cursor is at the end of the text or the text is empty.
    // We render the existing text and append a styled space to represent the cursor.
    if cursor_pos == text.len_chars() {
        let text_slice = text.slice(..);
        let text_span = Span::from(Cow::from(text_slice));
        let cursor_span = Span::styled(" ", cursor_style); // Styled space
        vec![text_span, cursor_span]
    }
    // Case 2: The cursor is somewhere in the middle of the text.
    // We split the text into three parts: before, at, and after the cursor.
    else {
        // Slice the text into its three components.
        let text_before_cursor = text.slice(..cursor_pos);
        let text_at_cursor = text.slice(cursor_pos..=cursor_pos);
        let text_after_cursor = text.slice(cursor_pos + 1..);

        // Create styled spans for each component.
        let span_before = Span::from(Cow::from(text_before_cursor));
        let span_at_cursor = Span::styled(Cow::from(text_at_cursor), cursor_style);
        let span_after = Span::from(Cow::from(text_after_cursor));

        vec![span_before, span_at_cursor, span_after]
    }
}
