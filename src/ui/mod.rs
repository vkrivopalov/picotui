mod cluster_header;
mod login;
mod nodes;

use crate::app::{App, InputMode};

/// Format bytes in human-readable binary units (KiB, MiB, GiB, etc.)
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB"];
    if bytes == 0 {
        return "0 B".to_string();
    }
    let mut size = bytes as f64;
    for unit in UNITS {
        if size < 1024.0 {
            return format!("{:.1} {}", size, unit);
        }
        size /= 1024.0;
    }
    format!("{:.1} PiB", size)
}
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header bar
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Status bar
        ])
        .split(frame.area());

    // Draw based on input mode
    match app.input_mode {
        InputMode::Login => {
            login::draw_login(frame, app, frame.area());
        }
        InputMode::Normal => {
            draw_header(frame, app, chunks[0]);
            nodes::draw_nodes(frame, app, chunks[1]);
            draw_status_bar(frame, app, chunks[2]);
        }
    }
}

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let mode_label = format!(" [{}] ", app.view_mode.label());
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" picotui - Picodata Cluster Monitor ")
        .title_bottom(Line::from(vec![
            Span::styled(mode_label, Style::default().fg(Color::Cyan)),
        ]).right_aligned());
    frame.render_widget(block, area);
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    use crate::app::ViewMode;

    let mut spans = vec![
        Span::styled(" ↑↓/jk", Style::default().fg(Color::Yellow)),
        Span::raw(" Navigate  "),
    ];

    // Show expand/collapse only in Tiers mode
    if app.view_mode == ViewMode::Tiers {
        spans.push(Span::styled("←→/hl", Style::default().fg(Color::Yellow)));
        spans.push(Span::raw(" Collapse/Expand  "));
    }

    spans.push(Span::styled("Enter", Style::default().fg(Color::Yellow)));
    spans.push(Span::raw(" Details  "));
    spans.push(Span::styled("g", Style::default().fg(Color::Yellow)));
    spans.push(Span::raw(" View  "));

    // Show sort options in Instances view
    if app.view_mode == ViewMode::Instances {
        spans.push(Span::styled("s", Style::default().fg(Color::Yellow)));
        spans.push(Span::raw(" Sort  "));
        spans.push(Span::styled("S", Style::default().fg(Color::Yellow)));
        spans.push(Span::raw(" Order  "));
    }

    spans.push(Span::styled("r", Style::default().fg(Color::Yellow)));
    spans.push(Span::raw(" Refresh  "));

    // Show logout option if auth is enabled
    if app.auth_enabled {
        spans.push(Span::styled("X", Style::default().fg(Color::Yellow)));
        spans.push(Span::raw(" Logout  "));
    }

    spans.push(Span::styled("q", Style::default().fg(Color::Yellow)));
    spans.push(Span::raw(" Quit"));

    if app.loading {
        spans.push(Span::raw("  │  "));
        spans.push(Span::styled(
            "Loading...",
            Style::default().fg(Color::Cyan),
        ));
    } else if let Some(ref error) = app.last_error {
        spans.push(Span::raw("  │  "));
        spans.push(Span::styled(
            format!("Error: {}", error),
            Style::default().fg(Color::Red),
        ));
    }

    let paragraph = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));

    frame.render_widget(paragraph, area);
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
