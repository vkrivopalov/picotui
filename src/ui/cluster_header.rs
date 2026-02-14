use super::format_bytes;
use crate::models::ClusterInfo;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

pub fn draw_cluster_header(frame: &mut Frame, info: &ClusterInfo, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Cluster Info ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Cluster name & version
            Constraint::Length(1), // Instance counts
            Constraint::Length(1), // Memory gauge
        ])
        .split(inner);

    // Row 1: Cluster name and version
    let name_line = Line::from(vec![
        Span::styled("Cluster: ", Style::default().fg(Color::Gray)),
        Span::styled(&info.cluster_name, Style::default().fg(Color::White)),
        Span::raw("  │  "),
        Span::styled("Version: ", Style::default().fg(Color::Gray)),
        Span::styled(&info.cluster_version, Style::default().fg(Color::Cyan)),
        Span::raw("  │  "),
        Span::styled("Picodata: ", Style::default().fg(Color::Gray)),
        Span::styled(
            &info.current_instance_version,
            Style::default().fg(Color::Cyan),
        ),
        Span::raw("  │  "),
        Span::styled("Replicasets: ", Style::default().fg(Color::Gray)),
        Span::styled(
            info.replicasets_count.to_string(),
            Style::default().fg(Color::White),
        ),
    ]);
    frame.render_widget(Paragraph::new(name_line), chunks[0]);

    // Row 2: Instance counts
    let online = info.instances_current_state_online;
    let offline = info.instances_current_state_offline;
    let total = online + offline;

    let status_color = if offline == 0 {
        Color::Green
    } else if online == 0 {
        Color::Red
    } else {
        Color::Yellow
    };

    let instances_line = Line::from(vec![
        Span::styled("Instances: ", Style::default().fg(Color::Gray)),
        Span::styled(format!("{}", online), Style::default().fg(Color::Green)),
        Span::styled("/", Style::default().fg(Color::Gray)),
        Span::styled(format!("{}", total), Style::default().fg(status_color)),
        Span::styled(" online", Style::default().fg(Color::Gray)),
        if offline > 0 {
            Span::styled(
                format!(" ({} offline)", offline),
                Style::default().fg(Color::Red),
            )
        } else {
            Span::raw("")
        },
        Span::raw("  │  "),
        Span::styled("Plugins: ", Style::default().fg(Color::Gray)),
        Span::styled(
            if info.plugins.is_empty() {
                "none".to_string()
            } else {
                info.plugins.join(", ")
            },
            Style::default().fg(Color::White),
        ),
    ]);
    frame.render_widget(Paragraph::new(instances_line), chunks[1]);

    // Row 3: Memory gauge
    let used = info.memory.used;
    let usable = info.memory.usable;
    let ratio = if usable > 0 {
        used as f64 / usable as f64
    } else {
        0.0
    };

    let gauge_color = if ratio < 0.7 {
        Color::Green
    } else if ratio < 0.9 {
        Color::Yellow
    } else {
        Color::Red
    };

    let label = format!(
        "Memory: {} / {} ({:.1}%)",
        format_bytes(used),
        format_bytes(usable),
        info.capacity_usage
    );

    let gauge = Gauge::default()
        .ratio(ratio.min(1.0))
        .label(label)
        .gauge_style(Style::default().fg(gauge_color).bg(Color::DarkGray));

    frame.render_widget(gauge, chunks[2]);
}
