use super::centered_rect;
use crate::app::{App, LoginFocus};
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
    let popup_area = centered_rect(50, 60, area);
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
            Constraint::Length(2), // Remember me checkbox
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
    let username_style = if app.login_focus == LoginFocus::Username {
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
    if app.login_focus == LoginFocus::Username {
        frame.set_cursor_position((
            username_inner.x + app.login_username.len() as u16,
            username_inner.y,
        ));
    }

    // Password field
    let password_style = if app.login_focus == LoginFocus::Password {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let password_title = if app.login_show_password {
        " Password (visible) "
    } else {
        " Password "
    };
    let password_block = Block::default()
        .borders(Borders::ALL)
        .title(password_title)
        .border_style(password_style);

    let password_inner = password_block.inner(chunks[2]);
    frame.render_widget(password_block, chunks[2]);

    let password_display = if app.login_show_password {
        app.login_password.clone()
    } else {
        "*".repeat(app.login_password.len())
    };
    let password_text = Paragraph::new(password_display);
    frame.render_widget(password_text, password_inner);

    // Show cursor in password field
    if app.login_focus == LoginFocus::Password {
        frame.set_cursor_position((
            password_inner.x + app.login_password.len() as u16,
            password_inner.y,
        ));
    }

    // Remember me checkbox
    let checkbox_focused = app.login_focus == LoginFocus::RememberMe;
    let checkbox_style = if checkbox_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let checkbox_char = if app.login_remember_me { "[x]" } else { "[ ]" };
    let checkbox_line = Line::from(vec![
        Span::styled(if checkbox_focused { "> " } else { "  " }, checkbox_style),
        Span::styled(checkbox_char, checkbox_style),
        Span::styled(" Remember me", checkbox_style),
        Span::styled(
            " (save login session)",
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    frame.render_widget(Paragraph::new(checkbox_line), chunks[3]);

    // Error message
    if let Some(ref error) = app.login_error {
        let error_msg = Paragraph::new(Line::from(vec![Span::styled(
            error.as_str(),
            Style::default().fg(Color::Red),
        )]));
        frame.render_widget(error_msg, chunks[4]);
    }

    // Submit hint
    let hint = Paragraph::new(Line::from(vec![
        Span::styled("Tab/↑↓", Style::default().fg(Color::Yellow)),
        Span::raw(" navigate  "),
        Span::styled("Space", Style::default().fg(Color::Yellow)),
        Span::raw(" toggle  "),
        Span::styled("^S", Style::default().fg(Color::Yellow)),
        Span::raw(" show/hide  "),
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::raw(" login  "),
        Span::styled("Esc", Style::default().fg(Color::Yellow)),
        Span::raw(" quit"),
    ]));
    frame.render_widget(hint, chunks[5]);
}
