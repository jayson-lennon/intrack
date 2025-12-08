use std::borrow::Cow;

use crate::{
    App,
    feat::{
        tui_issue_table::{Column, SortDirection, apply_issue_filter, apply_issue_sort},
        tui_widget::{HelpPopup, InputBox},
    },
};

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Row, Table},
};

pub trait IssueTableDraw {
    fn render(self, area: Rect, buf: &mut Buffer);
}
impl IssueTableDraw for &mut App {
    #[allow(clippy::too_many_lines)]
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

        let mut filtered_issues = apply_issue_filter(
            &self.tuistate.issue_table.filter_input_state().text(),
            &self.issues,
        );

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

        let columns = &self.tuistate.issue_table.columns;

        // Sort filtered issues
        let sort_col = &self.tuistate.issue_table.sort_by;
        let sort_dir = self.tuistate.issue_table.sort_direction;
        apply_issue_sort(&mut filtered_issues, sort_col, sort_dir);

        for (table_index, issue) in filtered_issues.iter().enumerate() {
            self.tuistate
                .issue_table
                .display_map
                .insert(table_index, issue.id);
        }

        // Header
        let header_style = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);
        let selected_header_style = Style::default()
            .fg(Color::White)
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD);
        let header = Row::new(
            columns
                .iter()
                .map(|col| {
                    let mut label = col.to_string();
                    let cell_style = if col == sort_col {
                        let arrow = match sort_dir {
                            SortDirection::Ascending => " ▲",
                            SortDirection::Descending => " ▼",
                        };
                        label.push_str(arrow);
                        selected_header_style
                    } else {
                        header_style
                    };
                    Cell::from(label).style(cell_style)
                })
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
                                Column::Created => {
                                    issue.created.strftime("%FT%TZ").to_string().into()
                                }
                                Column::Status => format!("{:?}", issue.status).into(),
                                Column::Priority => format!("{:?}", issue.priority).into(),
                                Column::CreatedBy => issue.created_by.as_str().into(),
                                Column::Custom(key) => issue
                                    .custom
                                    .get(key.as_str())
                                    .map_or(String::default().into(), Cow::from),
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
                Column::Id => Constraint::Length(4),
                Column::Title => Constraint::Fill(1),
                Column::Created => Constraint::Length(20),
                Column::Status => Constraint::Length(8),
                Column::Priority => Constraint::Length(11),
                Column::CreatedBy => Constraint::Length(30),
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
            self.tuistate.issue_table.filter_input_state_mut(),
        );

        if self.tuistate.issue_table.show_help {
            let items = vec![
                ("<shift>h", "Sort column left"),
                ("<shift>l", "Sort column right"),
                ("<shift>j", "Sort descending"),
                ("<shift>k", "Sort ascending"),
                ("h", "Cursor left"),
                ("j", "Cursor down"),
                ("k", "Cursor up"),
                ("l", "Cursor right"),
                ("c", "Change columns"),
                ("s", "Toggle issue status"),
                ("/", "Filter"),
                ("n", "New issue"),
                ("?", "Show help"),
            ];
            let help_widget = HelpPopup::new(items).title("Hotkeys");
            help_widget.render(*buf.area(), buf);
        }
    }
}
