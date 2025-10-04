mod components;
mod config;

use std::{io, time::Duration};
use crossterm::event::{self, Event, KeyCode};
use glob::glob;
use ratatui::{prelude::*, widgets::*};
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
    process::Command,
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
    show_popup: bool,
    show_help: bool,
    popup_title: String,
    popup_content: String,
    parallel_mode: bool,
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
            show_popup: false,
            show_help: false,
            popup_title: String::new(),
            popup_content: String::new(),
            parallel_mode: false,
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

    async fn run_check(&mut self) {
        if let Some(selected) = self.jail_list_state.selected() {
            let jail_name = self.jails[selected].clone();
            self.popup_title = format!("Check: {}", jail_name);
            let mut cmd = Command::new("trimorph-solo");
            if self.parallel_mode {
                cmd.arg("--parallel");
            }
            cmd.arg("--check").arg(&jail_name);
            let output = cmd.output().await;
            match output {
                Ok(output) => {
                    self.popup_content = String::from_utf8_lossy(&output.stdout).to_string() + &String::from_utf8_lossy(&output.stderr).to_string();
                }
                Err(e) => {
                    self.popup_content = format!("Error executing command: {}", e);
                }
            }
            self.show_popup = true;
        }
    }

    async fn run_dry_run(&mut self) {
        if let Some(selected) = self.jail_list_state.selected() {
            let jail_name = self.jails[selected].clone();
            self.popup_title = format!("Dry Run: {}", jail_name);
            let mut cmd = Command::new("trimorph-solo");
            if self.parallel_mode {
                cmd.arg("--parallel");
            }
            cmd.arg("--dry-run").arg(&jail_name).arg("echo").arg("test");
            let output = cmd.output().await;
            match output {
                Ok(output) => {
                    self.popup_content = String::from_utf8_lossy(&output.stdout).to_string() + &String::from_utf8_lossy(&output.stderr).to_string();
                }
                Err(e) => {
                    self.popup_content = format!("Error executing command: {}", e);
                }
            }
            self.show_popup = true;
        }
    }

    async fn run_install_to_host(&mut self) {
        if let Some(selected) = self.jail_list_state.selected() {
            let jail_name = self.jails[selected].clone();
            self.popup_title = format!("Install to Host: {}", jail_name);
            // Note: trimorph-install-to-host doesn't support parallel mode directly
            let output = Command::new("trimorph-install-to-host").arg(&jail_name).arg("echo").arg("test").output().await;
            match output {
                Ok(output) => {
                    self.popup_content = String::from_utf8_lossy(&output.stdout).to_string() + &String::from_utf8_lossy(&output.stderr).to_string();
                }
                Err(e) => {
                    self.popup_content = format!("Error executing command: {}", e);
                }
            }
            self.show_popup = true;
        }
    }

    async fn run_update_check(&mut self) {
        self.popup_title = "Update Check".to_string();
        let output = Command::new("trimorph-update-check").output().await;
        match output {
            Ok(output) => {
                self.popup_content = String::from_utf8_lossy(&output.stdout).to_string() + &String::from_utf8_lossy(&output.stderr).to_string();
            }
            Err(e) => {
                self.popup_content = format!("Error executing command: {}", e);
            }
        }
        self.show_popup = true;
    }

    async fn run_status(&mut self) {
        self.popup_title = "Status".to_string();
        let output = Command::new("trimorph-status").output().await;
        match output {
            Ok(output) => {
                self.popup_content = String::from_utf8_lossy(&output.stdout).to_string() + &String::from_utf8_lossy(&output.stderr).to_string();
            }
            Err(e) => {
                self.popup_content = format!("Error executing command: {}", e);
            }
        }
        self.show_popup = true;
    }

    async fn run_config_get(&mut self) {
        self.popup_title = "Configuration".to_string();
        let output = Command::new("trimorph-config").arg("get").output().await;
        match output {
            Ok(output) => {
                self.popup_content = String::from_utf8_lossy(&output.stdout).to_string() + &String::from_utf8_lossy(&output.stderr).to_string();
            }
            Err(e) => {
                self.popup_content = format!("Error executing command: {}", e);
            }
        }
        self.show_popup = true;
    }

    async fn run_cleanup(&mut self) {
        self.popup_title = "Cleanup".to_string();
        let output = Command::new("trimorph-cleanup").output().await;
        match output {
            Ok(output) => {
                self.popup_content = String::from_utf8_lossy(&output.stdout).to_string() + &String::from_utf8_lossy(&output.stderr).to_string();
            }
            Err(e) => {
                self.popup_content = format!("Error executing command: {}", e);
            }
        }
        self.show_popup = true;
    }

    async fn run_export(&mut self) {
        if let Some(selected) = self.jail_list_state.selected() {
            let jail_name = self.jails[selected].clone();
            self.popup_title = format!("Export: {}", jail_name);
            let output = Command::new("trimorph-export").arg(&jail_name).arg("--help").output().await;  // Using --help to test the command
            match output {
                Ok(output) => {
                    self.popup_content = String::from_utf8_lossy(&output.stdout).to_string() + &String::from_utf8_lossy(&output.stderr).to_string();
                }
                Err(e) => {
                    self.popup_content = format!("Error executing command: {}", e);
                }
            }
            self.show_popup = true;
        }
    }

    fn toggle_parallel_mode(&mut self) {
        self.parallel_mode = !self.parallel_mode;
    }
    
    fn show_help_info(&mut self) {
        self.show_help = true;
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
    let mut last_cpu_usage_nsec = 0;
    let mut last_time = std::time::Instant::now();

    loop {
        let output = Command::new("systemctl").arg("show").arg(&scope_name).arg("-p").arg("MemoryCurrent").arg("-p").arg("CPUUsageNSec").output().await;
        if let Ok(output) = output {
            let status_str = String::from_utf8_lossy(&output.stdout);
            let mem_usage_str = status_str.lines().find(|line| line.starts_with("MemoryCurrent=")).and_then(|line| line.split('=').nth(1)).unwrap_or("0");
            let cpu_usage_nsec_str = status_str.lines().find(|line| line.starts_with("CPUUsageNSec=")).and_then(|line| line.split('=').nth(1)).unwrap_or("0");

            let mem_usage_bytes = mem_usage_str.parse::<u64>().unwrap_or(0);
            let current_cpu_usage_nsec = cpu_usage_nsec_str.parse::<u64>().unwrap_or(0);
            let now = std::time::Instant::now();
            let duration = now.duration_since(last_time);
            last_time = now;

            let cpu_diff_nsec = current_cpu_usage_nsec.saturating_sub(last_cpu_usage_nsec);
            last_cpu_usage_nsec = current_cpu_usage_nsec;

            let cpu_percent = if duration.as_nanos() > 0 {
                ((cpu_diff_nsec as f64 / duration.as_nanos() as f64) * 100.0).min(100.0) as u16
            } else {
                0
            };

            // Assuming a max of 1GB for memory for percentage calc. A better approach would be to get total system memory.
            let mem_percent = ((mem_usage_bytes as f64 / (1024.0 * 1024.0 * 1024.0)) * 100.0).min(100.0) as u16;


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
    std::process::Command::new("systemctl").arg("is-active").arg("--quiet").arg(&format!("trimorph-{}.scope", jail_name)).status().map_or(false, |s| s.success())
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
                if app.show_popup {
                    if key.code == KeyCode::Esc {
                        app.show_popup = false;
                    }
                } else if app.show_help {
                    if key.code == KeyCode::Esc || key.code == KeyCode::Char('h') {
                        app.show_help = false;
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Down => app.next_jail(),
                        KeyCode::Up => app.previous_jail(),
                        KeyCode::Char('c') => {
                            app.run_check().await;
                        }
                        KeyCode::Char('d') => {
                            app.run_dry_run().await;
                        }
                        KeyCode::Char('i') => {
                            app.run_install_to_host().await;
                        }
                        KeyCode::Char('u') => {
                            app.run_update_check().await;
                        }
                        KeyCode::Char('s') => {
                            app.run_status().await;
                        }
                        KeyCode::Char('g') => {
                            app.run_config_get().await;
                        }
                        KeyCode::Char('e') => {
                            app.run_cleanup().await;
                        }
                        KeyCode::Char('x') => {
                            app.run_export().await;
                        }
                        KeyCode::Char('p') => {
                            app.toggle_parallel_mode();
                        }
                        KeyCode::Char('h') => {
                            app.show_help_info();
                        }
                        _ => {}
                    }
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

    let parallel_status = if app.parallel_mode { " [PARALLEL ON]" } else { " [PARALLEL OFF]" };
    let status_text = format!("Press 'q' to quit, 'Up'/'Down' to navigate, 'c' for check, 'd' for dry-run, 'i' for install to host, 'u' for update check, 's' for status, 'g' for config, 'e' for cleanup, 'x' for export.{}", parallel_status);
    f.render_widget(StatusBar::new(&status_text)
        .style(Style::default().fg(config.theme.main_fg.0).bg(config.theme.main_bg.0)), chunks[2]);

    if app.show_popup {
        let block = Block::default().title(app.popup_title.as_str()).borders(Borders::ALL);
        let area = centered_rect(60, 50, f.size());
        let paragraph = Paragraph::new(app.popup_content.as_str()).block(block).wrap(Wrap { trim: true });
        f.render_widget(Clear, area); //this clears the background
        f.render_widget(paragraph, area);
    } else if app.show_help {
        let block = Block::default().title("Help Information").borders(Borders::ALL);
        let area = centered_rect(70, 80, f.size());
        let parallel_status = if app.parallel_mode { "ON" } else { "OFF" };
        let help_content = format!("Trimorph TUI Help:\n\n\
            Navigation:\n\
            - Up/Down: Navigate between jails\n\
            - q: Quit the application\n\n\
            Jail Operations:\n\
            - c: Run check command on selected jail\n\
            - d: Run dry-run command on selected jail\n\
            - p: Toggle parallel mode\n\n\
            Host Installation:\n\
            - i: Install packages to host from selected jail\n\n\
            Updates & Management:\n\
            - u: Check for updates\n\
            - g: Get configuration settings\n\n\
            Utilities:\n\
            - s: Show status of all jails\n\
            - e: Run cleanup command\n\
            - x: Export from selected jail\n\
            - h: Show this help\n\n\
            Parallel Mode: {}",
            parallel_status);
        let paragraph = Paragraph::new(help_content).block(block).wrap(Wrap { trim: true });
        f.render_widget(Clear, area); //this clears the background
        f.render_widget(paragraph, area);
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
