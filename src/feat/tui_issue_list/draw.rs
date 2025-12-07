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
        let (content_area, filter_area) = {
            let block = Block::default().title("Issue List").borders(Borders::ALL);
            let content_area = block.inner(area);
            block.render(area, buf);

            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Fill(1), Constraint::Length(1)])
                .split(content_area);
            (layout[0], layout[1])
        };

        let input_box = InputBox::default().with_prefix(vec![
            Span::from("/").style(Style::default().fg(Color::Red)),
            Span::from(" Filter >> "),
        ]);

        let items: Vec<_> = (1..=200).map(|i| ListItem::new(format!("{i}"))).collect();
        let list = List::new(items)
            .highlight_style(Style::new().reversed())
            .repeat_highlight_symbol(true);

        StatefulWidget::render(list, content_area, buf, &mut self.tuistate.issue_list.list);

        StatefulWidget::render(
            input_box,
            filter_area,
            buf,
            &mut self.tuistate.issue_list.filter,
        );
    }
}
