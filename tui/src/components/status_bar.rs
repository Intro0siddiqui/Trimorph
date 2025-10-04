use ratatui::{prelude::*, widgets::*};

pub struct StatusBar<'a> {
    text: Line<'a>,
    style: Style,
}

impl<'a> StatusBar<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text: Line::from(text).alignment(Alignment::Center),
            style: Style::default().fg(Color::White).bg(Color::Blue),
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl<'a> Widget for StatusBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().style(self.style);
        let paragraph = Paragraph::new(self.text).block(block);
        paragraph.render(area, buf);
    }
}