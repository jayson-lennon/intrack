use std::borrow::Cow;

use crate::{
    App,
    feat::{tui_issue_table::Column, tui_widget::InputBox},
};

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Row, Table},
};

pub trait IssueTableDraw {
    fn render(self, area: Rect, buf: &mut Buffer);
}
impl IssueTableDraw for &mut App {
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

        // Compute filtered issues based on filter text
        let filter_text = self.tuistate.issue_table.filter.text().to_lowercase();
        let filtered_issues: Vec<&crate::feat::issue::Issue> = self
            .issues
            // TODO: Replace `self.issues.iter()` with actual iterator over `&Issue` from `Issues` (e.g., `self.issues.issues.iter()` if `Issues` has `pub issues: Vec<Issue>`).
            .iter_issues()
            .filter(|issue| {
                filter_text.is_empty() || issue.title.to_lowercase().contains(&filter_text)
            })
            .collect();

        let columns = {
            let state = &self.tuistate.issue_table;
            state.columns.clone()
        };

        // Clamp table selection to filtered length
        let table_state = &mut self.tuistate.issue_table.table;
        if let Some(selected) = table_state.selected() {
            let len = filtered_issues.len();
            if selected >= len {
                table_state.select(if len == 0 {
                    None
                } else {
                    Some(len.saturating_sub(1))
                });
            }
        }

        // Header
        let header_style = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);
        let header = Row::new(
            columns
                .iter()
                .map(|col| Cell::from(col.to_string()).style(header_style))
                .collect::<Vec<_>>(),
        );

        // Rows
        let rows: Vec<Row> = filtered_issues
            .iter()
            .map(|issue| {
                Row::new(
                    columns
                        .iter()
                        .map(|col| {
                            let content: Cow<'_, str> = match col {
                                Column::Id => issue.id.to_string().into(),
                                Column::Title => issue.title.as_str().into(),
                                Column::Created => format!("{}", issue.created).into(),
                                Column::Status => format!("{:?}", issue.status).into(),
                                Column::Priority => format!("{:?}", issue.priority).into(),
                                Column::CreatedBy => issue.created_by.as_str().into(),
                                Column::Custom(key) => issue
                                    .custom
                                    .get(key.as_str())
                                    .map_or(String::default().into(), |v| v.into()),
                            };
                            Cell::from(content)
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .collect();

        // Constraints based on columns
        let constraints: Vec<Constraint> = columns
            .iter()
            .map(|col| match col {
                Column::Id => Constraint::Length(6),
                Column::Title => Constraint::Fill(1),
                Column::Created => Constraint::Length(20),
                Column::Status => Constraint::Length(8),
                Column::Priority => Constraint::Length(10),
                Column::CreatedBy => Constraint::Length(15),
                Column::Custom(_) => Constraint::Min(12),
            })
            .collect();

        let table = Table::new(rows, &constraints[..])
            .header(header)
            .row_highlight_style(Style::new().reversed());

        StatefulWidget::render(table, content_area, buf, table_state);

        StatefulWidget::render(
            input_box,
            filter_area,
            buf,
            &mut self.tuistate.issue_table.filter,
        );
    }
}
