use std::borrow::Cow;

use skim::{AnsiString, DisplayContext, ItemPreview, PreviewContext, SkimItem};

use crate::feat::issue::Issue;

impl SkimItem for Issue {
    fn text(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.title)
    }

    fn display<'a>(&'a self, _: DisplayContext<'a>) -> AnsiString<'a> {
        AnsiString::parse(&format!("\x1b[32m{}\x1b[m", self.title))
    }

    fn preview(&self, _context: PreviewContext) -> ItemPreview {
        if self.title.starts_with("color") {
            ItemPreview::AnsiText(format!("\x1b[31mhello:\x1b[m\n{}", self.title))
        } else {
            ItemPreview::Text(format!("hello:\n{}", self.title))
        }
    }
}
