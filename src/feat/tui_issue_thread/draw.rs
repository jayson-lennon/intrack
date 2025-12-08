use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem},
};

use crate::{
    App,
    feat::{issue::Comment, tui_widget::HelpPopup},
};

pub trait IssueThreadDraw {
    fn render(self, area: Rect, buf: &mut Buffer);
}

impl IssueThreadDraw for &mut App {
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

        let mut items: Vec<ListItem> = vec![
            ListItem::new(format!("Status: {:?}", issue.status)),
            ListItem::new(format!("Priority: {}", issue.priority)),
            ListItem::new(format!(
                "Created: {}",
                issue.created.strftime("%Y-%m-%d %H:%M:%S")
            )),
            ListItem::new(format!("Created by: {}", issue.created_by)),
            ListItem::new(""),
        ];

        for comment in &comments {
            let header = format!(
                "Comment by {} at {}",
                comment.created_by,
                comment.created.strftime("%Y-%m-%d %H:%M:%S")
            );
            items.push(ListItem::new(header));
            for line in comment.content.lines() {
                let trimmed_line = line.trim_start();
                if !trimmed_line.is_empty() {
                    items.push(ListItem::new(format!("  {trimmed_line}")));
                }
            }
            items.push(ListItem::new(""));
        }

        // Clamp selection to items length
        if let Some(selected) = list_state.selected() {
            let len = items.len();
            if selected >= len {
                list_state.select(if len == 0 {
                    None
                } else {
                    Some(len.saturating_sub(1))
                });
            }
        }

        let list = List::new(items)
            .block(Block::default().title("Comments").borders(Borders::ALL))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );

        StatefulWidget::render(list, inner_area, buf, list_state);

        if self.tuistate.issue_thread.show_help {
            let items = vec![
                ("q, <esc>", "Back to issues"),
                ("a", "Add comment"),
                ("j, <down>", "Cursor down"),
                ("k, <up>", "Cursor up"),
                ("?", "Toggle help"),
            ];
            let help_widget = HelpPopup::new(items).title("Hotkeys");
            help_widget.render(*buf.area(), buf);
        }
    }
}
