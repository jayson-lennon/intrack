use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

#[derive(Clone)]
pub struct HelpPopup<'a> {
    items: Vec<(&'a str, &'a str)>,
    title: Option<&'a str>,
}

impl<'a> HelpPopup<'a> {
    pub fn new(items: Vec<(&'a str, &'a str)>) -> Self {
        Self { items, title: None }
    }

    #[must_use]
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }
}

impl Widget for HelpPopup<'_> {
    #[allow(clippy::cast_possible_truncation)]
    fn render(self, _: Rect, buf: &mut Buffer) {
        let area = buf.area();
        let n_items = self.items.len();
        let max_key_width: usize = self.items.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
        let max_desc_width: usize = self.items.iter().map(|(_, d)| d.len()).max().unwrap_or(0);
        let col_width_chars = max_key_width + 2 + max_desc_width;
        let col_inner_width = col_width_chars as u16;
        let sep_width: u16 = 4;
        let max_content_h: usize = area.height.saturating_sub(4) as usize;
        let mut num_cols = if n_items <= max_content_h || max_content_h == 0 {
            1usize
        } else {
            n_items.div_ceil(max_content_h)
        };
        let est_col_w = col_inner_width.max(20);
        let max_possible_cols = (area.width.saturating_sub(20) / est_col_w).max(1) as usize;
        num_cols = num_cols.min(max_possible_cols);
        num_cols = num_cols.max(1);

        let items_per_col = n_items / num_cols;
        let remainder = n_items % num_cols;
        let mut col_items: Vec<Vec<_>> = Vec::with_capacity(num_cols);
        let mut start = 0usize;
        for i in 0..num_cols {
            let col_size = items_per_col + usize::from(i < remainder);
            col_items.push(self.items[start..start + col_size].to_vec());
            start += col_size;
        }
        let max_col_h = col_items.iter().map(Vec::len).max().unwrap_or(0);
        let content_height = max_col_h as u16;
        let num_seps = (num_cols as u16).saturating_sub(1);
        let total_inner_width = (num_cols as u16) * col_inner_width + num_seps * sep_width;
        let popup_inner_width = total_inner_width.max(20u16);
        let popup_width = popup_inner_width + 2u16;
        let popup_x = area.width.saturating_sub(popup_width);
        let popup_height = content_height + 2u16;
        let bottom_pad = 1u16;
        let popup_y = area.y + area.height.saturating_sub(popup_height + bottom_pad);
        let popup_rect = Rect {
            x: popup_x - 1,
            y: popup_y,
            width: popup_width,
            height: popup_height,
        };

        buf.set_style(popup_rect, Style::default().bg(Color::Rgb(30, 30, 30)));

        let title = self.title.unwrap_or(" Keybinds ");
        let block = Block::default()
            .title(title)
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));

        let key_style = Style::default().fg(Color::Red).add_modifier(Modifier::BOLD);
        let desc_style = Style::default().fg(Color::Gray);
        let arrow_style = Style::default().fg(Color::DarkGray);

        let mut content: Vec<Line<'_>> = Vec::new();
        let key_fixed_width = max_key_width + 2usize;
        let desc_fixed_width = max_desc_width;
        let sep_chars = 4usize;
        for row in 0..max_col_h {
            let mut spans: Vec<Span<'_>> = Vec::new();
            for (col_i, col) in col_items.iter().enumerate() {
                if row < col.len() {
                    let (key, desc) = col[row];
                    let key_padding_len = key_fixed_width.saturating_sub(key.len());
                    let key_padding_str = " ".repeat(key_padding_len.saturating_sub(1));
                    let desc_padding_len = desc_fixed_width.saturating_sub(desc.len());
                    let desc_padding_str = " ".repeat(desc_padding_len);

                    spans.push(Span::raw(key_padding_str));
                    spans.push(Span::styled(key, key_style));
                    spans.push(Span::from(" "));
                    spans.push(Span::styled("➜ ", arrow_style));
                    spans.push(Span::styled(desc, desc_style));
                    spans.push(Span::raw(desc_padding_str));
                } else {
                    let pad_len = col_width_chars;
                    spans.push(Span::raw(" ".repeat(pad_len)));
                }
                if col_i < num_cols.saturating_sub(1) {
                    spans.push(Span::raw(" ".repeat(sep_chars)));
                }
            }
            content.push(Line::from(spans));
        }

        let paragraph = Paragraph::new(content).block(block);
        paragraph.render(popup_rect, buf);
    }
}
// let area = buf.area();
//         // Compute column widths
//         let max_key_width: usize = self.items.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
//         let max_desc_width: usize = self.items.iter().map(|(_, d)| d.len()).max().unwrap_or(0);
//         let pair_width = (max_key_width + 2 + max_desc_width) as u16;
//         let min_inner_width = 20u16;
//         let popup_inner_width = pair_width.max(min_inner_width);
//         let popup_width = popup_inner_width + 2u16; // left and right borders
//         let popup_x = area.width.saturating_sub(popup_width);
//
//         let popup_rect = Rect {
//             x: popup_x,
//             y: area.y + area.height - self.items.len() as u16 - 4,
//             width: popup_width,
//             height: self.items.len() as u16 + 2,
//         };
//
//         buf.set_style(popup_rect, Style::default().bg(Color::Rgb(30, 30, 30)));
//
//         let title = self.title.unwrap_or(" Keybinds ");
//         let block = Block::default()
//             .title(title)
//             .title_alignment(Alignment::Center)
//             .borders(Borders::ALL)
//             .border_style(Style::default().fg(Color::White));
//
//         let key_style = Style::default()
//             .fg(Color::LightCyan)
//             .add_modifier(Modifier::BOLD);
//         let desc_style = Style::default().fg(Color::Gray);
//
//         let mut content: Vec<Line<'_>> = Vec::new();
//         let key_fixed_width = max_key_width + 2;
//         let desc_fixed_width = max_desc_width;
//         for (key, desc) in self.items {
//             let key_padding_len = key_fixed_width.saturating_sub(key.len());
//             let key_padding_str = " ".repeat(key_padding_len);
//             let desc_padding_len = desc_fixed_width.saturating_sub(desc.len());
//             let desc_padding_str = " ".repeat(desc_padding_len);
//
//             let line = Line::from(vec![
//                 Span::styled(key, key_style),
//                 Span::raw(key_padding_str),
//                 Span::styled(desc, desc_style),
//                 Span::raw(desc_padding_str),
//             ]);
//             content.push(line);
//         }
//
//         let paragraph = Paragraph::new(content).block(block);
//         paragraph.render(popup_rect, buf);
