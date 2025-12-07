use std::borrow::Cow;

use ratatui::{layout::Rect, prelude::*, widgets::Widget};
use ropey::Rope;
use wherror::Error;

use crate::feat::tui::{Event, EventExt, EventPropagation, KeyCode};

#[derive(Debug, Error)]
#[error(debug)]
pub struct InputBoxError;

#[derive(Clone, Debug, Default)]
pub struct InputBoxState {
    text: Rope,
    cursor: usize,
    is_focused: bool,
}

impl InputBoxState {
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

    pub fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    pub fn text<'a>(&'a self) -> Cow<'a, str> {
        self.text.line(0).into()
    }
}

#[derive(Clone, Debug, Default)]
pub struct InputBox<'a> {
    pub prefix: Vec<Span<'a>>,
}

impl<'a> InputBox<'a> {
    pub fn new() -> Self {
        Self { prefix: vec![] }
    }

    pub fn with_prefix(mut self, prefix: Vec<Span<'a>>) -> Self {
        self.prefix = prefix;
        self
    }
}

impl<'a> StatefulWidget for InputBox<'a> {
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

/// Optionally applies highlight to the query box when it's in focus.
fn apply_focus_highlight(is_focused: bool) -> Style {
    if is_focused {
        Style::default().fg(Color::Black).bg(Color::Yellow)
    } else {
        Style::default()
    }
}

fn format_text_with_cursor<'a>(
    text: &'a Rope,
    cursor_pos: usize,
    cursor_style: Style,
    is_focused: bool,
) -> Vec<Span<'a>> {
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
        let text_at_cursor = text.slice(cursor_pos..cursor_pos + 1);
        let text_after_cursor = text.slice(cursor_pos + 1..);

        // Create styled spans for each component.
        let span_before = Span::from(Cow::from(text_before_cursor));
        let span_at_cursor = Span::styled(Cow::from(text_at_cursor), cursor_style);
        let span_after = Span::from(Cow::from(text_after_cursor));

        vec![span_before, span_at_cursor, span_after]
    }
}
