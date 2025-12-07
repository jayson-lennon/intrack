use crate::App;

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

pub trait IssueListDraw {
    fn render(self, area: Rect, buf: &mut Buffer);
}

impl IssueListDraw for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().title("Issue List").borders(Borders::ALL);
        let inner_area = block.inner(area);
        block.render(area, buf);

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

        Paragraph::new("Issue 1: Open...").render(inner_chunks[0], buf);
        Paragraph::new("Issue 2: In progress...").render(inner_chunks[1], buf);
        Paragraph::new("Issue 3: Closed...").render(inner_chunks[2], buf);
    }
}
