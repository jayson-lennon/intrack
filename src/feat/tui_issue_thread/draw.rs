use ratatui::{
    prelude::*,
    widgets::{Block, Borders},
};
use tui_widget_list::{ListBuilder, ListView};

use crate::{
    App,
    feat::{issue::Comment, tui_widget::HelpPopup},
};

// Define LineItem before impl IssueThreadDraw:
#[derive(Debug, Clone)]
struct LineItem {
    text: String,
    style: Style,
}

impl Widget for LineItem {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Line::from(self.text).style(self.style).render(area, buf);
    }
}

pub trait IssueThreadDraw {
    fn render(self, area: Rect, buf: &mut Buffer);
}

impl IssueThreadDraw for &mut App {
    #[allow(clippy::cast_possible_truncation)]
    fn render(self, area: Rect, buf: &mut Buffer) {
        let issue_id = self.tuistate.issue_thread.issue_id;
        let Some(issue) = self.issues.get_issue(&issue_id) else {
            let block = Block::default()
                .title("No issue selected")
                .borders(Borders::ALL);
            block.render(area, buf);
            return;
        };

        let block = Block::default()
            .title(format!("Issue #{}: {}", issue.id, issue.title))
            .borders(Borders::ALL);
        let inner_area = block.inner(area);
        block.render(area, buf);

        let list_state = &mut self.tuistate.issue_thread.list_state;

        let comments: Vec<&Comment> = self
            .issues
            .iter_comments()
            .filter(|(id, _)| **id == issue_id)
            .map(|(_, comments)| comments.iter().collect())
            .next()
            .unwrap_or_default();

        let line_data: Vec<String> = {
            let mut data = vec![
                format!("Status: {:?}", issue.status),
                format!("Priority: {}", issue.priority),
                format!("Created: {}", issue.created.strftime("%Y-%m-%d %H:%M:%S")),
                format!("Created by: {}", issue.created_by),
                String::new(),
            ];
            let indent_width = 3;
            let max_width = inner_area.width.saturating_sub(indent_width as u16).max(1) as usize;
            for comment in &comments {
                let header = format!(
                    "Comment by {} at {}",
                    comment.created_by,
                    comment.created.strftime("%Y-%m-%d %H:%M:%S")
                );
                data.push(header);
                for line in comment.content.lines() {
                    for wrapped in textwrap::wrap(line, max_width) {
                        let trimmed_line = wrapped.trim_start();
                        if !trimmed_line.is_empty() {
                            data.push(format!("{:indent_width$} {}", "", trimmed_line));
                        }
                    }
                    if line.trim().is_empty() {
                        data.push(format!("{:indent_width$}", ""));
                    }
                }
                data.push(String::new());
            }
            data
        };

        let total_count = line_data.len() as usize;

        // Clamp selection
        if let Some(selected) = list_state.selected {
            let len = total_count;
            if selected >= len {
                list_state.select(if len == 0 {
                    None
                } else {
                    Some(len.saturating_sub(1))
                });
            }
        }

        let builder = ListBuilder::new(move |context| {
            let text = line_data[context.index].clone();
            let mut item = LineItem {
                text,
                style: Style::default(),
            };
            if context.is_selected {
                item.style = Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD);
            }
            (item, 1u16)
        });

        let list = ListView::new(builder, total_count);

        list.render(inner_area, buf, list_state);

        if self.tuistate.issue_thread.show_help {
            let items = vec![
                ("q, <esc>", "Back to issues"),
                ("a", "Add comment"),
                ("j, <down>", "Cursor down"),
                ("k, <up>", "Cursor up"),
                ("<ctrl>d", "Cursor down by 10"),
                ("<ctrl>u", "Cursor up by 10"),
                ("?", "Toggle help"),
            ];
            let help_widget = HelpPopup::new(items).title("Hotkeys");
            help_widget.render(*buf.area(), buf);
        }
    }
}
