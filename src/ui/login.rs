use super::centered_rect;
use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn draw_login(frame: &mut Frame, app: &App, area: Rect) {
    // Draw background
    let bg = Block::default().style(Style::default().bg(Color::Black));
    frame.render_widget(bg, area);

    // Draw centered login box
    let popup_area = centered_rect(50, 50, area);
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Picodata Login ")
        .style(Style::default().bg(Color::Black).fg(Color::White));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Title/instructions
            Constraint::Length(3), // Username field
            Constraint::Length(3), // Password field
            Constraint::Length(2), // Error message
            Constraint::Length(2), // Submit hint
            Constraint::Min(0),    // Padding
        ])
        .margin(1)
        .split(inner);

    // Instructions
    let instructions = Paragraph::new(Line::from(vec![
        Span::raw("Enter your credentials to connect to "),
        Span::styled("Picodata", Style::default().fg(Color::Cyan)),
    ]));
    frame.render_widget(instructions, chunks[0]);

    // Username field
    let username_style = if !app.login_focus_password {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let username_block = Block::default()
        .borders(Borders::ALL)
        .title(" Username ")
        .border_style(username_style);

    let username_inner = username_block.inner(chunks[1]);
    frame.render_widget(username_block, chunks[1]);

    let username_text = Paragraph::new(app.login_username.as_str());
    frame.render_widget(username_text, username_inner);

    // Show cursor in username field
    if !app.login_focus_password {
        frame.set_cursor_position((
            username_inner.x + app.login_username.len() as u16,
            username_inner.y,
        ));
    }

    // Password field
    let password_style = if app.login_focus_password {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let password_block = Block::default()
        .borders(Borders::ALL)
        .title(" Password ")
        .border_style(password_style);

    let password_inner = password_block.inner(chunks[2]);
    frame.render_widget(password_block, chunks[2]);

    let masked_password = "*".repeat(app.login_password.len());
    let password_text = Paragraph::new(masked_password);
    frame.render_widget(password_text, password_inner);

    // Show cursor in password field
    if app.login_focus_password {
        frame.set_cursor_position((
            password_inner.x + app.login_password.len() as u16,
            password_inner.y,
        ));
    }

    // Error message
    if let Some(ref error) = app.login_error {
        let error_msg = Paragraph::new(Line::from(vec![Span::styled(
            error.as_str(),
            Style::default().fg(Color::Red),
        )]));
        frame.render_widget(error_msg, chunks[3]);
    }

    // Submit hint
    let hint = Paragraph::new(Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::Yellow)),
        Span::raw(" switch field  "),
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::raw(" login  "),
        Span::styled("Esc/q", Style::default().fg(Color::Yellow)),
        Span::raw(" quit"),
    ]));
    frame.render_widget(hint, chunks[4]);
}
