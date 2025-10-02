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
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .split(area);

        let cpu_gauge = Gauge::default()
            .block(Block::default().title("CPU").borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Green))
            .percent(self.cpu_usage);
        cpu_gauge.render(layout[0], buf);

        let mem_gauge = Gauge::default()
            .block(Block::default().title("Memory").borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Cyan))
            .percent(self.mem_usage);
        mem_gauge.render(layout[1], buf);
    }
}