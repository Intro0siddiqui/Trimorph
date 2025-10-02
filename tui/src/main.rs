mod components;
mod config;

use std::{io, process::Command, time::Duration};
use crossterm::event::{self, Event, KeyCode};
use glob::glob;
use ratatui::{prelude::*, widgets::*};
use regex::Regex;
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
    sync::mpsc,
    task::JoinHandle,
};

use components::{
    log_view::LogView,
    resource_monitor::ResourceMonitor,
    status_bar::StatusBar,
    title_bar::TitleBar,
};
use config::{load_config, Config};

struct App {
    jails: Vec<String>,
    jail_list_state: ListState,
    log_lines: Vec<String>,
    log_receiver: mpsc::Receiver<String>,
    log_sender: mpsc::Sender<String>,
    resource_receiver: mpsc::Receiver<(u16, u16)>,
    resource_sender: mpsc::Sender<(u16, u16)>,
    active_log_task: Option<JoinHandle<()>>,
    active_resource_task: Option<JoinHandle<()>>,
    cpu_usage: u16,
    mem_usage: u16,
}

impl App {
    fn new(
        jails: Vec<String>,
        log_sender: mpsc::Sender<String>,
        log_receiver: mpsc::Receiver<String>,
        resource_sender: mpsc::Sender<(u16, u16)>,
        resource_receiver: mpsc::Receiver<(u16, u16)>,
    ) -> Self {
        let mut app = App {
            jails,
            jail_list_state: ListState::default(),
            log_lines: Vec::new(),
            log_receiver,
            log_sender,
            resource_receiver,
            resource_sender,
            active_log_task: None,
            active_resource_task: None,
            cpu_usage: 0,
            mem_usage: 0,
        };
        if !app.jails.is_empty() {
            app.jail_list_state.select(Some(0));
            app.spawn_tasks();
        }
        app
    }

    fn spawn_tasks(&mut self) {
        if let Some(task) = self.active_log_task.take() {
            task.abort();
        }
        if let Some(task) = self.active_resource_task.take() {
            task.abort();
        }
        self.log_lines.clear();
        self.cpu_usage = 0;
        self.mem_usage = 0;

        if let Some(selected) = self.jail_list_state.selected() {
            let jail_name = self.jails[selected].clone();
            let log_sender = self.log_sender.clone();
            self.active_log_task = Some(tokio::spawn(tail_log_file(jail_name.clone(), log_sender)));

            let resource_sender = self.resource_sender.clone();
            self.active_resource_task = Some(tokio::spawn(monitor_resources(jail_name, resource_sender)));
        }
    }

    pub fn next_jail(&mut self) {
        let i = match self.jail_list_state.selected() {
            Some(i) => {
                if i >= self.jails.len() - 1 { 0 } else { i + 1 }
            }
            None => 0,
        };
        self.jail_list_state.select(Some(i));
        self.spawn_tasks();
    }

    pub fn previous_jail(&mut self) {
        let i = match self.jail_list_state.selected() {
            Some(i) => {
                if i == 0 { self.jails.len() - 1 } else { i - 1 }
            }
            None => 0,
        };
        self.jail_list_state.select(Some(i));
        self.spawn_tasks();
    }
}

async fn tail_log_file(jail_name: String, sender: mpsc::Sender<String>) {
    let log_path = format!("/var/log/trimorph/{}.log", jail_name);
    if let Ok(file) = File::open(&log_path).await {
        let mut reader = BufReader::new(file);
        let mut line = String::new();
        while let Ok(len) = reader.read_line(&mut line).await {
            if len == 0 {
                tokio::time::sleep(Duration::from_millis(200)).await;
                continue;
            }
            if sender.send(line.clone()).await.is_err() {
                break;
            }
            line.clear();
        }
    } else {
        let _ = sender.send(format!("Log file not found: {}", log_path)).await;
    }
}

async fn monitor_resources(jail_name: String, sender: mpsc::Sender<(u16, u16)>) {
    let scope_name = format!("trimorph-{}.scope", jail_name);
    let mem_re = Regex::new(r"Memory:\s+([0-9.]+)M").unwrap();
    let cpu_re = Regex::new(r"CPU:\s+([0-9.]+)s").unwrap(); // This is cumulative, but we'll show it for now

    loop {
        let output = Command::new("systemctl").arg("status").arg(&scope_name).output();
        if let Ok(output) = output {
            let status_str = String::from_utf8_lossy(&output.stdout);
            let mem_usage = mem_re.captures(&status_str).and_then(|caps| {
                caps.get(1)?.as_str().parse::<f32>().ok()
            }).unwrap_or(0.0);

            // For demo, we'll treat total seconds as a percentage. A real impl would be more complex.
            let cpu_usage = cpu_re.captures(&status_str).and_then(|caps| {
                caps.get(1)?.as_str().parse::<f32>().ok()
            }).unwrap_or(0.0);

            // Assuming a max of 1GB for memory and 10s for CPU for percentage calc
            let mem_percent = ((mem_usage / 1024.0) * 100.0).min(100.0) as u16;
            let cpu_percent = (cpu_usage * 10.0).min(100.0) as u16;

            if sender.send((cpu_percent, mem_percent)).await.is_err() {
                break;
            }
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}


fn get_jail_list() -> io::Result<Vec<String>> {
    let mut jails = Vec::new();
    let path = "/etc/trimorph/jails.d/*.conf";
    for entry in glob(path).map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))? {
        if let Ok(path) = entry {
            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                jails.push(name.to_string());
            }
        }
    }
    Ok(jails)
}

fn get_jail_status(jail_name: &str) -> bool {
    Command::new("systemctl").arg("is-active").arg("--quiet").arg(&format!("trimorph-{}.scope", jail_name)).status().map_or(false, |s| s.success())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let config = load_config();
    let jails = get_jail_list().unwrap_or_else(|_| vec!["Error reading jails".to_string()]);
    let (log_tx, log_rx) = mpsc::channel(100);
    let (res_tx, res_rx) = mpsc::channel(10);
    let mut app = App::new(jails, log_tx, log_rx, res_tx, res_rx);

    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, &mut app, &config).await;

    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), crossterm::terminal::LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }
    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App, config: &Config) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app, config))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down => app.next_jail(),
                    KeyCode::Up => app.previous_jail(),
                    _ => {}
                }
            }
        }

        while let Ok(line) = app.log_receiver.try_recv() {
            app.log_lines.push(line);
        }

        if let Ok((cpu, mem)) = app.resource_receiver.try_recv() {
            app.cpu_usage = cpu;
            app.mem_usage = mem;
        }
    }
}

fn ui(f: &mut Frame, app: &mut App, config: &Config) {
    let chunks = Layout::default().direction(Direction::Vertical).margin(1)
        .constraints([Constraint::Length(5), Constraint::Min(0), Constraint::Length(1)].as_ref())
        .split(f.size());

    f.render_widget(TitleBar::new("Trimorph TUI"), chunks[0]);

    let main_chunks = Layout::default().direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(chunks[1]);

    let left_chunks = Layout::default().direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(main_chunks[0]);

    let jail_items: Vec<ListItem> = app.jails.iter().map(|name| {
        let status = if get_jail_status(name) { Span::styled(" (active)", Style::default().fg(Color::Green)) } else { Span::raw("") };
        ListItem::new(Line::from(vec![Span::raw(name.clone()), status]))
    }).collect();

    let jail_list_widget = List::new(jail_items)
        .block(Block::default().title("Jails").borders(Borders::ALL))
        .highlight_style(Style::default().fg(config.theme.highlight_fg.0).bg(config.theme.highlight_bg.0))
        .highlight_symbol("> ");
    f.render_stateful_widget(jail_list_widget, left_chunks[0], &mut app.jail_list_state);

    f.render_widget(ResourceMonitor::new("Resources", app.cpu_usage, app.mem_usage), left_chunks[1]);

    let log_text = app.log_lines.join("");
    f.render_widget(LogView::new("Logs", &log_text), main_chunks[1]);

    f.render_widget(StatusBar::new("Press 'q' to quit, 'Up'/'Down' to navigate.")
        .style(Style::default().fg(config.theme.main_fg.0).bg(config.theme.main_bg.0)), chunks[2]);
}