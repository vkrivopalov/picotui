mod api;
mod app;
mod models;
mod ui;

use anyhow::{anyhow, Result};
use app::{App, InputMode};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::{Duration, Instant};

struct Args {
    url: String,
    refresh: u64,
    debug: bool,
}

fn parse_args() -> Result<Args> {
    let mut args = pico_args::Arguments::from_env();

    if args.contains(["-h", "--help"]) {
        println!(
            "picotui - Terminal UI for Picodata cluster management

USAGE:
    picotui [OPTIONS]

OPTIONS:
    -u, --url <URL>       Picodata HTTP API URL [default: http://localhost:8080]
    -r, --refresh <SECS>  Auto-refresh interval in seconds, 0 to disable [default: 5]
    -d, --debug           Enable debug mode (log API responses to picotui.log)
    -h, --help            Print help
    -V, --version         Print version"
        );
        std::process::exit(0);
    }

    if args.contains(["-V", "--version"]) {
        println!("picotui {}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }

    let url: String = args
        .opt_value_from_str(["-u", "--url"])?
        .unwrap_or_else(|| "http://localhost:8080".to_string());

    let refresh: u64 = args
        .opt_value_from_str(["-r", "--refresh"])?
        .unwrap_or(5);

    let debug = args.contains(["-d", "--debug"]);

    let remaining = args.finish();
    if !remaining.is_empty() {
        return Err(anyhow!("Unknown arguments: {:?}", remaining));
    }

    Ok(Args { url, refresh, debug })
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = parse_args()?;

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
