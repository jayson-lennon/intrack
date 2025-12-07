use crate::{App, feat::tui_widget::InputBox};

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

pub trait IssueListDraw {
    fn render(self, area: Rect, buf: &mut Buffer);
}

impl IssueListDraw for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().title("Issue List").borders(Borders::ALL);
        let inner_area = block.inner(area);
        block.render(area, buf);

        let input_box = InputBox::default().with_prefix(vec![
            Span::from("/").style(Style::default().fg(Color::Red)),
            Span::from(" Query >> "),
        ]);

        // Render any arbitrary widgets within inner_area
        // Example: multiple independent widgets
        let inner_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(inner_area);
        let item = ListItem::new("hi");
        let items: Vec<_> = (1..=1).map(|i| item.clone()).collect();
        let list = List::new(items)
            .block(Block::bordered().title("List"))
            .highlight_style(Style::new().reversed())
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true);

        let state = &mut self.tuistate.issue_list.list;
        *state.offset_mut() = 1; // display the second item and onwards
        state.select(Some(3)); // select the forth item (0-indexed)
        StatefulWidget::render(list, area, buf, state);

        StatefulWidget::render(
            input_box,
            inner_chunks[1],
            buf,
            &mut self.tuistate.issue_list.filter,
        );
    }
}
