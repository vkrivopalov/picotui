use anyhow::{anyhow, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use picotui::api;
use picotui::app::{App, InputMode, LoginFocus, ViewMode};
use picotui::ui;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::sync::mpsc::channel;
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

    let refresh: u64 = args.opt_value_from_str(["-r", "--refresh"])?.unwrap_or(5);

    let debug = args.contains(["-d", "--debug"]);

    let remaining = args.finish();
    if !remaining.is_empty() {
        return Err(anyhow!("Unknown arguments: {:?}", remaining));
    }

    Ok(Args {
        url,
        refresh,
        debug,
    })
}

fn main() -> Result<()> {
    let args = parse_args()?;

    // Clear debug log file if debug mode
    if args.debug {
        let _ = std::fs::write("picotui.log", "");
    }

    // Create channels for API communication
    let (request_tx, request_rx) = channel();
    let (response_tx, response_rx) = channel();

    // Spawn API worker thread
    api::spawn_api_worker(args.url.clone(), request_rx, response_tx, args.debug);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app with channels
    let mut app = App::new(args.url.clone(), request_tx, response_rx);

    // Start initialization (non-blocking)
    app.start_init();

    // Run main loop
    let result = run_app(&mut terminal, &mut app, args.refresh);

    // Shutdown API worker
    app.shutdown();

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

fn run_app(
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
        // Process any pending API responses (non-blocking)
        app.process_responses();

        // Draw UI
        terminal.draw(|f| ui::draw(f, &mut *app))?;

        // Poll for keyboard input with short timeout for responsiveness
        let timeout = Duration::from_millis(50);

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match app.input_mode {
                    InputMode::Login => handle_login_input(app, key.code, key.modifiers),
                    InputMode::Normal => {
                        if app.show_detail {
                            handle_detail_input(app, key.code);
                        } else {
                            handle_normal_input(app, key.code, key.modifiers);
                        }
                    }
                }
            }
        }

        // Auto-refresh
        if last_tick.elapsed() >= tick_rate && app.input_mode == InputMode::Normal && !app.loading {
            app.request_refresh();
            last_tick = Instant::now();
        }
    }

    Ok(())
}

fn handle_login_input(app: &mut App, key: KeyCode, modifiers: KeyModifiers) {
    match key {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.running = false;
        }
        KeyCode::Char('s') if modifiers.contains(KeyModifiers::CONTROL) => {
            // Toggle show/hide password
            app.login_show_password = !app.login_show_password;
        }
        KeyCode::Tab | KeyCode::Down => {
            // Cycle through: Username -> Password -> RememberMe -> Username
            app.login_focus = match app.login_focus {
                LoginFocus::Username => LoginFocus::Password,
                LoginFocus::Password => LoginFocus::RememberMe,
                LoginFocus::RememberMe => LoginFocus::Username,
            };
        }
        KeyCode::BackTab | KeyCode::Up => {
            // Cycle backwards
            app.login_focus = match app.login_focus {
                LoginFocus::Username => LoginFocus::RememberMe,
                LoginFocus::Password => LoginFocus::Username,
                LoginFocus::RememberMe => LoginFocus::Password,
            };
        }
        KeyCode::Enter => {
            match app.login_focus {
                LoginFocus::RememberMe => {
                    // Toggle checkbox
                    app.login_remember_me = !app.login_remember_me;
                }
                _ => {
                    // Submit login
                    if !app.login_username.is_empty() && !app.loading {
                        app.request_login();
                    }
                }
            }
        }
        KeyCode::Char(' ') if app.login_focus == LoginFocus::RememberMe => {
            // Space toggles checkbox
            app.login_remember_me = !app.login_remember_me;
        }
        KeyCode::Backspace => match app.login_focus {
            LoginFocus::Username => {
                app.login_username.pop();
            }
            LoginFocus::Password => {
                app.login_password.pop();
            }
            LoginFocus::RememberMe => {}
        },
        KeyCode::Char(c) => match app.login_focus {
            LoginFocus::Username => {
                app.login_username.push(c);
            }
            LoginFocus::Password => {
                app.login_password.push(c);
            }
            LoginFocus::RememberMe => {}
        },
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

// Default visible height for page navigation (will be overridden by actual terminal size)
const DEFAULT_PAGE_HEIGHT: usize = 20;

fn handle_normal_input(app: &mut App, key: KeyCode, modifiers: KeyModifiers) {
    // Handle filter input mode
    if app.filter_active {
        match key {
            KeyCode::Esc => {
                // Clear filter and exit filter mode
                app.filter_text.clear();
                app.filter_active = false;
                app.reset_selection();
            }
            KeyCode::Enter => {
                // Exit filter mode but keep filter
                app.filter_active = false;
            }
            KeyCode::Backspace => {
                app.filter_text.pop();
                app.reset_selection();
            }
            KeyCode::Char(c) => {
                app.filter_text.push(c);
                app.reset_selection();
            }
            _ => {}
        }
        return;
    }

    match key {
        KeyCode::Char('q') => {
            app.running = false;
        }
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.running = false;
        }
        // Basic navigation
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
        // Vim-style navigation
        KeyCode::Home => {
            // Go to first item
            app.select_first();
        }
        KeyCode::End => {
            // Go to last item
            app.select_last();
        }
        KeyCode::Char('d') if modifiers.contains(KeyModifiers::CONTROL) => {
            // Half page down (Ctrl+D)
            app.select_half_page_down(DEFAULT_PAGE_HEIGHT);
        }
        KeyCode::Char('u') if modifiers.contains(KeyModifiers::CONTROL) => {
            // Half page up (Ctrl+U)
            app.select_half_page_up(DEFAULT_PAGE_HEIGHT);
        }
        KeyCode::Char('f') if modifiers.contains(KeyModifiers::CONTROL) => {
            // Full page down (Ctrl+F)
            app.select_page_down(DEFAULT_PAGE_HEIGHT);
        }
        KeyCode::Char('b') if modifiers.contains(KeyModifiers::CONTROL) => {
            // Full page up (Ctrl+B)
            app.select_page_up(DEFAULT_PAGE_HEIGHT);
        }
        KeyCode::PageDown => {
            app.select_page_down(DEFAULT_PAGE_HEIGHT);
        }
        KeyCode::PageUp => {
            app.select_page_up(DEFAULT_PAGE_HEIGHT);
        }
        // Actions
        KeyCode::Enter => {
            app.toggle_detail();
        }
        KeyCode::Char('r') => {
            if !app.loading {
                app.request_refresh();
            }
        }
        KeyCode::Char('X') => {
            // Logout (capital X to avoid accidental logout)
            if app.auth_enabled {
                app.logout();
            }
        }
        // View modes
        KeyCode::Char('g') => {
            // Cycle view mode and clear filter
            app.view_mode = app.view_mode.cycle_next();
            app.filter_text.clear();
            app.filter_active = false;
            app.reset_selection();
        }
        KeyCode::Char('1') => {
            app.view_mode = ViewMode::Tiers;
            app.filter_text.clear();
            app.filter_active = false;
            app.reset_selection();
        }
        KeyCode::Char('2') => {
            app.view_mode = ViewMode::Replicasets;
            app.filter_text.clear();
            app.filter_active = false;
            app.reset_selection();
        }
        KeyCode::Char('3') => {
            app.view_mode = ViewMode::Instances;
            app.filter_text.clear();
            app.filter_active = false;
            app.reset_selection();
        }
        // Sorting
        KeyCode::Char('s') => {
            // Cycle sort field (only in instances view)
            if app.view_mode == ViewMode::Instances {
                app.sort_field = app.sort_field.cycle_next();
                app.reset_selection();
            }
        }
        KeyCode::Char('S') => {
            // Toggle sort order (only in instances view)
            if app.view_mode == ViewMode::Instances {
                app.sort_order = app.sort_order.toggle();
                app.reset_selection();
            }
        }
        // Filtering
        KeyCode::Char('/') => {
            // Start filter mode (only in instances view)
            if app.view_mode == ViewMode::Instances {
                app.filter_active = true;
            }
        }
        _ => {}
    }
}
