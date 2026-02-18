use super::cluster_header::draw_cluster_header;
use super::{centered_rect, format_bytes};
use crate::app::{App, TreeItem, ViewMode};
use crate::models::{InstanceInfo, ReplicasetInfo, StateVariant};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

/// Helper to create spans with filter match highlighting
fn highlight_match(text: &str, filter: &str, base_style: Style) -> Vec<Span<'static>> {
    if filter.is_empty() {
        return vec![Span::styled(text.to_string(), base_style)];
    }

    let filter_lower = filter.to_lowercase();
    let text_lower = text.to_lowercase();

    let mut spans = Vec::new();
    let mut last_end = 0;

    for (start, _) in text_lower.match_indices(&filter_lower) {
        // Add text before match
        if start > last_end {
            spans.push(Span::styled(text[last_end..start].to_string(), base_style));
        }
        // Add highlighted match
        let end = start + filter.len();
        spans.push(Span::styled(
            text[start..end].to_string(),
            base_style.bg(Color::Yellow).fg(Color::Black),
        ));
        last_end = end;
    }

    // Add remaining text after last match
    if last_end < text.len() {
        spans.push(Span::styled(text[last_end..].to_string(), base_style));
    }

    if spans.is_empty() {
        vec![Span::styled(text.to_string(), base_style)]
    } else {
        spans
    }
}

pub fn draw_nodes(frame: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Cluster header
            Constraint::Min(0),    // Content
        ])
        .split(area);

    // Draw cluster header
    if let Some(ref info) = app.cluster_info {
        draw_cluster_header(frame, info, chunks[0]);
    } else {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Cluster Info ");
        let loading = Paragraph::new("Loading...").block(block);
        frame.render_widget(loading, chunks[0]);
    }

    // Draw content based on view mode
    match app.view_mode {
        ViewMode::Tiers => draw_tiers_view(frame, app, chunks[1]),
        ViewMode::Replicasets => draw_replicasets_view(frame, app, chunks[1]),
        ViewMode::Instances => draw_instances_view(frame, app, chunks[1]),
    }

    // Draw detail popup if active
    if app.show_detail {
        if let Some(instance) = app.get_selected_instance() {
            draw_instance_detail(frame, instance, frame.area());
        }
    }
}

fn draw_tiers_view(frame: &mut Frame, app: &mut App, area: Rect) {
    draw_tree(frame, app, area);
}

fn draw_tree(frame: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Tiers / Replicasets / Instances ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.tiers.is_empty() {
        let msg = Paragraph::new("No tiers found. Press 'r' to refresh.");
        frame.render_widget(msg, inner);
        return;
    }

    let items: Vec<ListItem> = app
        .tree_items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let is_selected = idx == app.selected_index;
            let line = match item {
                TreeItem::Tier(tier_idx) => format_tier_line(app, *tier_idx),
                TreeItem::Replicaset(tier_idx, rs_idx) => {
                    format_replicaset_line(app, *tier_idx, *rs_idx)
                }
                TreeItem::Instance(tier_idx, rs_idx, inst_idx) => {
                    format_instance_line(app, *tier_idx, *rs_idx, *inst_idx)
                }
            };

            let style = if is_selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_stateful_widget(list, inner, &mut app.list_state);
}

fn draw_replicasets_view(frame: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Replicasets ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Collect all replicasets from all tiers
    let replicasets: Vec<(&str, &ReplicasetInfo)> = app
        .tiers
        .iter()
        .flat_map(|tier| {
            tier.replicasets
                .iter()
                .map(move |rs| (tier.name.as_str(), rs))
        })
        .collect();

    if replicasets.is_empty() {
        let msg = Paragraph::new("No replicasets found. Press 'r' to refresh.");
        frame.render_widget(msg, inner);
        return;
    }

    let items: Vec<ListItem> = replicasets
        .iter()
        .enumerate()
        .map(|(idx, (tier_name, rs))| {
            let is_selected = idx == app.selected_index;

            let state_style = match rs.state {
                StateVariant::Online => Style::default().fg(Color::Green),
                StateVariant::Offline => Style::default().fg(Color::Red),
                StateVariant::Expelled => Style::default().fg(Color::DarkGray),
            };

            let mem_str = format!(
                "{}/{}",
                format_bytes(rs.memory.used),
                format_bytes(rs.memory.usable)
            );

            let line = Line::from(vec![
                Span::styled(rs.name.clone(), Style::default().fg(Color::White)),
                Span::raw(" ["),
                Span::styled(rs.state.to_string(), state_style),
                Span::raw("]  "),
                Span::styled("Tier:", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!(" {}  ", tier_name),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled("Inst:", Style::default().fg(Color::Gray)),
                Span::raw(format!(" {}  ", rs.instance_count)),
                Span::styled("Mem:", Style::default().fg(Color::Gray)),
                Span::raw(format!(" {} ({:.1}%)", mem_str, rs.capacity_usage)),
            ]);

            let style = if is_selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_stateful_widget(list, inner, &mut app.list_state);
}

fn draw_instances_view(frame: &mut Frame, app: &mut App, area: Rect) {
    // Build title with sort indicator
    let sort_indicator = format!(
        " Sort: {} {} ",
        app.sort_field.label(),
        app.sort_order.arrow()
    );

    // Build filter indicator for title
    let filter_indicator = if !app.filter_text.is_empty() {
        format!(" Filter: \"{}\" ", app.filter_text)
    } else if app.filter_active {
        " Filter: _ ".to_string()
    } else {
        String::new()
    };

    let mut title_spans = vec![Span::raw(" Instances ")];
    if !filter_indicator.is_empty() {
        title_spans.push(Span::styled(
            filter_indicator,
            Style::default().fg(Color::Yellow),
        ));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Line::from(title_spans))
        .title_bottom(
            Line::from(vec![Span::styled(
                sort_indicator,
                Style::default().fg(Color::Cyan),
            )])
            .right_aligned(),
        );

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Get sorted and filtered instances
    let instances = app.get_sorted_instances();

    if instances.is_empty() {
        let msg = if !app.filter_text.is_empty() {
            format!(
                "No instances match filter \"{}\". Press Esc to clear.",
                app.filter_text
            )
        } else {
            "No instances found. Press 'r' to refresh.".to_string()
        };
        let paragraph = Paragraph::new(msg);
        frame.render_widget(paragraph, inner);
        return;
    }

    let filter = &app.filter_text;

    let items: Vec<ListItem> = instances
        .iter()
        .enumerate()
        .map(|(idx, (_tier_name, rs_name, inst))| {
            let is_selected = idx == app.selected_index;

            let state_style = match inst.current_state {
                StateVariant::Online => Style::default().fg(Color::Green),
                StateVariant::Offline => Style::default().fg(Color::Red),
                StateVariant::Expelled => Style::default().fg(Color::DarkGray),
            };

            let leader_marker = if inst.is_leader { "★ " } else { "  " };

            let failure_domain_str = if inst.failure_domain.is_empty() {
                String::new()
            } else {
                inst.failure_domain
                    .iter()
                    .map(|(k, v)| format!("{}:{}", k, v))
                    .collect::<Vec<_>>()
                    .join(", ")
            };

            // Build line with highlighted matches
            let mut spans = vec![Span::styled(
                leader_marker,
                Style::default().fg(Color::Yellow),
            )];

            // Instance name (with highlighting)
            spans.extend(highlight_match(
                &inst.name,
                filter,
                Style::default().fg(Color::White),
            ));

            spans.push(Span::raw(" ["));
            spans.push(Span::styled(inst.current_state.to_string(), state_style));
            spans.push(Span::raw("]  "));
            spans.push(Span::styled("RS:", Style::default().fg(Color::Gray)));
            spans.push(Span::raw(" "));

            // Replicaset name (with highlighting)
            spans.extend(highlight_match(rs_name, filter, Style::default()));
            spans.push(Span::raw("  "));

            // Binary address (with highlighting)
            spans.extend(highlight_match(
                &inst.binary_address,
                filter,
                Style::default().fg(Color::Gray),
            ));

            // Failure domain (with highlighting)
            if !failure_domain_str.is_empty() {
                spans.push(Span::raw("  "));
                spans.extend(highlight_match(
                    &failure_domain_str,
                    filter,
                    Style::default().fg(Color::DarkGray),
                ));
            }

            let line = Line::from(spans);

            let style = if is_selected {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_stateful_widget(list, inner, &mut app.list_state);
}

fn format_tier_line(app: &App, tier_idx: usize) -> Line<'static> {
    let tier = &app.tiers[tier_idx];
    let expanded = app.expanded_tiers.contains(&tier_idx);
    let arrow = if expanded { "▼" } else { "▶" };

    let mem_str = format!(
        "{}/{}",
        format_bytes(tier.memory.used),
        format_bytes(tier.memory.usable)
    );

    Line::from(vec![
        Span::styled(arrow.to_string(), Style::default().fg(Color::Yellow)),
        Span::raw(" "),
        Span::styled(tier.name.clone(), Style::default().fg(Color::Cyan)),
        Span::raw("  "),
        Span::styled("RS:", Style::default().fg(Color::Gray)),
        Span::raw(format!(" {}  ", tier.replicaset_count)),
        Span::styled("Inst:", Style::default().fg(Color::Gray)),
        Span::raw(format!(" {}  ", tier.instance_count)),
        Span::styled("RF:", Style::default().fg(Color::Gray)),
        Span::raw(format!(" {}  ", tier.rf)),
        Span::styled("Buckets:", Style::default().fg(Color::Gray)),
        Span::raw(format!(" {}  ", tier.bucket_count)),
        Span::styled("Vote:", Style::default().fg(Color::Gray)),
        Span::raw(if tier.can_vote {
            " ✓  ".to_string()
        } else {
            " ✗  ".to_string()
        }),
        Span::styled("Mem:", Style::default().fg(Color::Gray)),
        Span::raw(format!(" {} ({:.1}%)", mem_str, tier.capacity_usage)),
    ])
}

fn format_replicaset_line(app: &App, tier_idx: usize, rs_idx: usize) -> Line<'static> {
    let tier = &app.tiers[tier_idx];
    let rs = &tier.replicasets[rs_idx];
    let expanded = app.expanded_replicasets.contains(&(tier_idx, rs_idx));
    let arrow = if expanded { "▼" } else { "▶" };

    let state_style = match rs.state {
        StateVariant::Online => Style::default().fg(Color::Green),
        StateVariant::Offline => Style::default().fg(Color::Red),
        StateVariant::Expelled => Style::default().fg(Color::DarkGray),
    };

    let mem_str = format!(
        "{}/{}",
        format_bytes(rs.memory.used),
        format_bytes(rs.memory.usable)
    );

    Line::from(vec![
        Span::raw("  ├─".to_string()),
        Span::styled(arrow.to_string(), Style::default().fg(Color::Yellow)),
        Span::raw(" "),
        Span::styled(rs.name.clone(), Style::default().fg(Color::White)),
        Span::raw(" ["),
        Span::styled(rs.state.to_string(), state_style),
        Span::raw("]  "),
        Span::styled("Inst:", Style::default().fg(Color::Gray)),
        Span::raw(format!(" {}  ", rs.instance_count)),
        Span::styled("Mem:", Style::default().fg(Color::Gray)),
        Span::raw(format!(" {} ({:.1}%)", mem_str, rs.capacity_usage)),
    ])
}

fn format_instance_line(
    app: &App,
    tier_idx: usize,
    rs_idx: usize,
    inst_idx: usize,
) -> Line<'static> {
    let tier = &app.tiers[tier_idx];
    let rs = &tier.replicasets[rs_idx];
    let inst = &rs.instances[inst_idx];

    let is_last = inst_idx == rs.instances.len() - 1;
    let prefix = if is_last {
        "  │  └─".to_string()
    } else {
        "  │  ├─".to_string()
    };

    let state_style = match inst.current_state {
        StateVariant::Online => Style::default().fg(Color::Green),
        StateVariant::Offline => Style::default().fg(Color::Red),
        StateVariant::Expelled => Style::default().fg(Color::DarkGray),
    };

    let leader_marker = if inst.is_leader {
        " ★".to_string()
    } else {
        "  ".to_string()
    };

    let pg_span = if !inst.pg_address.is_empty() {
        Span::styled(
            format!("  pg:{}", inst.pg_address),
            Style::default().fg(Color::Gray),
        )
    } else {
        Span::raw("".to_string())
    };

    Line::from(vec![
        Span::raw(prefix),
        Span::styled(leader_marker, Style::default().fg(Color::Yellow)),
        Span::raw(" "),
        Span::styled(inst.name.clone(), Style::default().fg(Color::White)),
        Span::raw(" ["),
        Span::styled(inst.current_state.to_string(), state_style),
        Span::raw("]  "),
        Span::styled(
            inst.binary_address.clone(),
            Style::default().fg(Color::Gray),
        ),
        pg_span,
    ])
}

fn draw_instance_detail(frame: &mut Frame, instance: &InstanceInfo, area: Rect) {
    let popup_area = centered_rect(60, 60, area);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Instance: {} ", instance.name))
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let state_color = match instance.current_state {
        StateVariant::Online => Color::Green,
        StateVariant::Offline => Color::Red,
        StateVariant::Expelled => Color::DarkGray,
    };

    let target_color = match instance.target_state {
        StateVariant::Online => Color::Green,
        StateVariant::Offline => Color::Red,
        StateVariant::Expelled => Color::DarkGray,
    };

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Name:          ", Style::default().fg(Color::Gray)),
            Span::styled(instance.name.clone(), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Current State: ", Style::default().fg(Color::Gray)),
            Span::styled(
                instance.current_state.to_string(),
                Style::default().fg(state_color),
            ),
        ]),
        Line::from(vec![
            Span::styled("Target State:  ", Style::default().fg(Color::Gray)),
            Span::styled(
                instance.target_state.to_string(),
                Style::default().fg(target_color),
            ),
        ]),
        Line::from(vec![
            Span::styled("Is Leader:     ", Style::default().fg(Color::Gray)),
            Span::styled(
                if instance.is_leader {
                    "Yes ★".to_string()
                } else {
                    "No".to_string()
                },
                Style::default().fg(if instance.is_leader {
                    Color::Yellow
                } else {
                    Color::White
                }),
            ),
        ]),
        Line::from(vec![
            Span::styled("Version:       ", Style::default().fg(Color::Gray)),
            Span::styled(instance.version.clone(), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Addresses:".to_string(),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  Binary:      ", Style::default().fg(Color::Gray)),
            Span::styled(
                instance.binary_address.clone(),
                Style::default().fg(Color::White),
            ),
        ]),
    ];

    if !instance.pg_address.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("  PostgreSQL:  ", Style::default().fg(Color::Gray)),
            Span::styled(
                instance.pg_address.clone(),
                Style::default().fg(Color::White),
            ),
        ]));
    }

    if !instance.http_address.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("  HTTP:        ", Style::default().fg(Color::Gray)),
            Span::styled(
                instance.http_address.clone(),
                Style::default().fg(Color::White),
            ),
        ]));
    }

    if !instance.failure_domain.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            "Failure Domain:".to_string(),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]));
        for (key, value) in &instance.failure_domain {
            lines.push(Line::from(vec![
                Span::styled(format!("  {}:", key), Style::default().fg(Color::Gray)),
                Span::raw(" "),
                Span::styled(value.clone(), Style::default().fg(Color::White)),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "Press Esc or Enter to close".to_string(),
        Style::default().fg(Color::DarkGray),
    )]));

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, inner);
}
