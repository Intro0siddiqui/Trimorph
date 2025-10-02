use ratatui::{prelude::*, widgets::*};

pub struct TitleBar<'a> {
    title: Line<'a>,
    logo: Text<'a>,
}

impl<'a> TitleBar<'a> {
    pub fn new(title: &'a str) -> Self {
        let logo = Text::from(vec![
            Line::from("  _______ __ ___   ___  __  ___  __"),
            Line::from(" |_  / __/ _` \\ \\ / / |/ / / _ \\/ /"),
            Line::from("  / / (_| (_| |\\ V /| ' < |  __/ / "),
            Line::from(" /___\\___\\__,_| \\_/ |_|\\_\\ \\___/_/  "),
        ]);
        Self {
            title: Line::from(title).alignment(Alignment::Center),
            logo,
        }
    }
}

impl<'a> Widget for TitleBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(self.title)
            .borders(Borders::ALL);
        let paragraph = Paragraph::new(self.logo)
            .block(block)
            .alignment(Alignment::Center);
        paragraph.render(area, buf);
    }
}