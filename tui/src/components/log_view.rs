use ratatui::{prelude::*, widgets::*};

pub struct LogView<'a> {
    pub title: Line<'a>,
    pub text: Text<'a>,
}

impl<'a> LogView<'a> {
    pub fn new(title: &'a str, log_content: &'a str) -> Self {
        Self {
            title: Line::from(title).alignment(Alignment::Center),
            text: Text::from(log_content),
        }
    }
}

impl<'a> Widget for LogView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(self.title)
            .borders(Borders::ALL);
        let paragraph = Paragraph::new(self.text)
            .block(block)
            .wrap(Wrap { trim: true });
        paragraph.render(area, buf);
    }
}