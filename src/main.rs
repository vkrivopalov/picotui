mod api;
mod app;
mod models;
mod ui;

use anyhow::Result;
use app::{App, InputMode};
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[command(name = "picotui")]
#[command(about = "Terminal UI for Picodata cluster management")]
#[command(version)]
struct Args {
    /// Picodata HTTP API URL
    #[arg(short, long, default_value = "http://localhost:8080")]
    url: String,

    /// Auto-refresh interval in seconds (0 to disable)
    #[arg(short, long, default_value = "5")]
    refresh: u64,

    /// Enable debug mode (log API responses to picotui.log)
    #[arg(short, long, default_value = "false")]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new(&args.url, args.debug)?;

    // Initialize app (check auth, load initial data)
    app.init().await?;

    // Run main loop
    let result = run_app(&mut terminal, &mut app, args.refresh).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    refresh_secs: u64,
) -> Result<()> {
    let tick_rate = if refresh_secs > 0 {
        Duration::from_secs(refresh_secs)
    } else {
        Duration::from_secs(3600) // Effectively disabled
    };
    let mut last_tick = Instant::now();

    while app.running {
        terminal.draw(|f| ui::draw(f, app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_millis(100));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match app.input_mode {
                    InputMode::Login => handle_login_input(app, key.code).await,
                    InputMode::Normal => {
                        if app.show_detail {
                            handle_detail_input(app, key.code);
                        } else {
                            handle_normal_input(app, key.code, key.modifiers).await;
                        }
                    }
                }
            }
        }

        // Auto-refresh
        if last_tick.elapsed() >= tick_rate && app.input_mode == InputMode::Normal {
            let _ = app.refresh_data().await;
            last_tick = Instant::now();
        }
    }

    Ok(())
}

async fn handle_login_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.running = false;
        }
        KeyCode::Tab => {
            app.login_focus_password = !app.login_focus_password;
        }
        KeyCode::Enter => {
            if !app.login_username.is_empty() {
                app.do_login().await;
            }
        }
        KeyCode::Backspace => {
            if app.login_focus_password {
                app.login_password.pop();
            } else {
                app.login_username.pop();
            }
        }
        KeyCode::Char(c) => {
            if app.login_focus_password {
                app.login_password.push(c);
            } else {
                app.login_username.push(c);
            }
        }
        _ => {}
    }
}

fn handle_detail_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
            app.show_detail = false;
        }
        _ => {}
    }
}

async fn handle_normal_input(app: &mut App, key: KeyCode, modifiers: KeyModifiers) {
    match key {
        KeyCode::Char('q') => {
            app.running = false;
        }
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.running = false;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.select_previous();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.select_next();
        }
        KeyCode::Right | KeyCode::Char('l') => {
            app.expand_selected();
        }
        KeyCode::Left | KeyCode::Char('h') => {
            app.collapse_selected();
        }
        KeyCode::Enter => {
            app.toggle_detail();
        }
        KeyCode::Char('r') => {
            let _ = app.refresh_data().await;
        }
        _ => {}
    }
}
