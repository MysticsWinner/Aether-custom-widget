//! Aether Live Performance Dashboard
//!
//! A ratatui terminal dashboard that connects to the Core Engine via Named
//! Pipe IPC and renders live CPU%, GPU%, and RAM used/free bars.
//!
//! Usage:
//!   1. Start the daemon first:  `cargo run -p core_engine`
//!   2. Open a new terminal:     `cargo run -p dashboard_tui`
//!   3. Press `q` or `Ctrl+C` to exit.

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ipc_protocol::ControlCommand;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Gauge, Paragraph, Wrap},
    Frame, Terminal,
};
use serde::Deserialize;
use std::{io, time::{Duration, Instant}};
use tracing::warn;

const PIPE_NAME: &str = r"\\.\pipe\CustomWidgetEngineControlPipe";
const POLL_MS: u64 = 500;

// ── IPC response schema ───────────────────────────────────────────────────────

#[derive(Debug, Default, Deserialize, Clone)]
struct StatusResponse {
    #[serde(default)]
    status: String,
    #[serde(default)]
    cpu_pct: f32,
    #[serde(default)]
    gpu_pct: f32,
    #[serde(default)]
    memory_used_mb: f32,
    #[serde(default)]
    memory_total_mb: f32,
    #[serde(default)]
    memory_free_mb: f32,
    #[serde(default)]
    active_widgets: Vec<String>,
    #[serde(default)]
    engine_version: String,
}

// ── Application state ─────────────────────────────────────────────────────────

#[derive(Default)]
struct App {
    metrics: StatusResponse,
    ipc_state: IpcState,
    uptime_secs: u64,
}

#[derive(Default, PartialEq, Clone, Copy)]
enum IpcState {
    #[default]
    Connecting,
    Connected,
    Error,
}

impl App {
    fn poll_ipc(&mut self) {
        match query_ipc() {
            Ok(resp) => {
                self.metrics = resp;
                self.ipc_state = IpcState::Connected;
            }
            Err(e) => {
                warn!("IPC poll error: {e:?}");
                self.ipc_state = IpcState::Error;
            }
        }
    }
}

// ── IPC helper ────────────────────────────────────────────────────────────────

#[cfg(windows)]
fn query_ipc() -> Result<StatusResponse> {
    use std::io::{Read, Write};
    use std::fs::OpenOptions;

    let cmd = serde_json::to_string(&ControlCommand::GetStatus)?;

    let mut pipe = OpenOptions::new()
        .read(true)
        .write(true)
        .open(PIPE_NAME)
        .map_err(|e| anyhow::anyhow!("Pipe open failed: {e}"))?;

    pipe.write_all(cmd.as_bytes())?;

    let mut buf = vec![0u8; 8192];
    let n = pipe.read(&mut buf)?;
    let raw = &buf[..n];

    let resp: StatusResponse = serde_json::from_slice(raw)
        .map_err(|e| anyhow::anyhow!("JSON parse error: {e}"))?;
    Ok(resp)
}

#[cfg(not(windows))]
fn query_ipc() -> Result<StatusResponse> {
    anyhow::bail!("Named pipe IPC is Windows-only")
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {err:?}");
    }
    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    let mut app = App::default();
    let mut last_poll = Instant::now() - Duration::from_secs(1); // poll immediately

    loop {
        // Poll IPC on interval
        if last_poll.elapsed() >= Duration::from_millis(POLL_MS) {
            app.uptime_secs += 1;
            app.poll_ipc();
            last_poll = Instant::now();
        }

        terminal.draw(|f| draw(f, &app))?;

        // Event handling (non-blocking)
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match (key.code, key.modifiers) {
                    (KeyCode::Char('q'), _)
                    | (KeyCode::Char('c'), KeyModifiers::CONTROL) => return Ok(()),
                    _ => {}
                }
            }
        }
    }
}

// ── Render ────────────────────────────────────────────────────────────────────

fn draw(f: &mut Frame, app: &App) {
    let area = f.area();

    // Outer card
    let outer = Block::default()
        .title(Line::from(vec![
            Span::styled(" ⚡ ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(
                "Aether Performance Monitor",
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(
                    "  Phase 15  v{}  ",
                    if app.metrics.engine_version.is_empty() {
                        "0.1.0"
                    } else {
                        &app.metrics.engine_version
                    }
                ),
                Style::default().fg(Color::DarkGray),
            ),
        ]))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color(app.ipc_state)));

    let inner = outer.inner(area);
    f.render_widget(outer, area);

    // Split: metrics (top) + footer (bottom 3 rows)
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(inner);

    draw_metrics(f, sections[0], app);
    draw_footer(f, sections[1], app);
}

fn draw_metrics(f: &mut Frame, area: Rect, app: &App) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // CPU
            Constraint::Length(1), // spacer
            Constraint::Length(3), // GPU
            Constraint::Length(1), // spacer
            Constraint::Length(3), // RAM
            Constraint::Min(0),
        ])
        .split(area);

    // CPU gauge
    let cpu = app.metrics.cpu_pct.clamp(0.0, 100.0);
    let cpu_gauge = Gauge::default()
        .block(
            Block::default()
                .title(Span::styled(
                    format!(" CPU  {:5.1}% ", cpu),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .gauge_style(
            Style::default()
                .fg(Color::Cyan)
                .bg(Color::Rgb(20, 22, 30))
                .add_modifier(Modifier::BOLD),
        )
        .ratio(cpu as f64 / 100.0)
        .label(format!("{:.1}%", cpu));
    f.render_widget(cpu_gauge, rows[0]);

    // GPU gauge
    let gpu = app.metrics.gpu_pct.clamp(0.0, 100.0);
    let gpu_gauge = Gauge::default()
        .block(
            Block::default()
                .title(Span::styled(
                    format!(" GPU  {:5.1}% ", gpu),
                    Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .gauge_style(
            Style::default()
                .fg(Color::Magenta)
                .bg(Color::Rgb(20, 22, 30))
                .add_modifier(Modifier::BOLD),
        )
        .ratio(gpu as f64 / 100.0)
        .label(format!("{:.1}%", gpu));
    f.render_widget(gpu_gauge, rows[2]);

    // RAM gauge
    let total = app.metrics.memory_total_mb;
    let used  = app.metrics.memory_used_mb.min(total);
    let free  = (total - used).max(0.0);
    let ram_pct = if total > 0.0 { used / total } else { 0.0 };
    let ram_gauge = Gauge::default()
        .block(
            Block::default()
                .title(Span::styled(
                    format!(
                        " RAM  {:.2}/{:.2} GB  ({:.0}% used • {:.2} GB free) ",
                        used / 1024.0, total / 1024.0,
                        ram_pct * 100.0, free / 1024.0,
                    ),
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .gauge_style(
            Style::default()
                .fg(Color::Green)
                .bg(Color::Rgb(20, 22, 30))
                .add_modifier(Modifier::BOLD),
        )
        .ratio(ram_pct as f64)
        .label(format!("{:.0}%", ram_pct * 100.0));
    f.render_widget(ram_gauge, rows[4]);
}

fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(40),
            Constraint::Percentage(20),
        ])
        .split(area);

    // IPC status
    let (ipc_label, ipc_color) = match app.ipc_state {
        IpcState::Connected  => ("● IPC Connected", Color::Green),
        IpcState::Connecting => ("○ Connecting…",   Color::Yellow),
        IpcState::Error      => ("✗ IPC Error – start core_engine first", Color::Red),
    };
    let status_p = Paragraph::new(ipc_label)
        .style(Style::default().fg(ipc_color))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    f.render_widget(status_p, cols[0]);

    // Active widgets
    let widgets_text = if app.metrics.active_widgets.is_empty() {
        "No active widgets".to_string()
    } else {
        format!("Widgets: {}", app.metrics.active_widgets.join(", "))
    };
    let widgets_p = Paragraph::new(widgets_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(widgets_p, cols[1]);

    // Quit hint + uptime
    let hint = Paragraph::new(format!("q/Ctrl+C quit  ↑{}s", app.uptime_secs))
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Right);
    f.render_widget(hint, cols[2]);
}

fn border_color(state: IpcState) -> Color {
    match state {
        IpcState::Connected  => Color::Rgb(0, 180, 255),
        IpcState::Connecting => Color::Yellow,
        IpcState::Error      => Color::Red,
    }
}
