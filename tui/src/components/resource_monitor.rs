use ratatui::{prelude::*, widgets::*};

pub struct ResourceMonitor<'a> {
    title: Line<'a>,
    cpu_usage: u16,
    mem_usage: u16,
}

impl<'a> ResourceMonitor<'a> {
    pub fn new(title: &'a str, cpu_usage: u16, mem_usage: u16) -> Self {
        Self {
            title: Line::from(title).alignment(Alignment::Center),
            cpu_usage,
            mem_usage,
        }
    }
}

impl<'a> Widget for ResourceMonitor<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let container = Block::default().title(self.title).borders(Borders::ALL);
        let inner_area = container.inner(area);
        container.render(area, buf);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .split(inner_area);

        let cpu_gauge = Gauge::default()
            .block(Block::default().title("CPU"))
            .gauge_style(Style::default().fg(Color::Green))
            .percent(self.cpu_usage);
        cpu_gauge.render(layout[0], buf);

        let mem_gauge = Gauge::default()
            .block(Block::default().title("Memory"))
            .gauge_style(Style::default().fg(Color::Cyan))
            .percent(self.mem_usage);
        mem_gauge.render(layout[1], buf);
    }
}