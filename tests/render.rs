//! Render tests using ratatui's TestBackend
//!
//! These tests verify that the UI renders correctly for various app states.

mod common;

use common::{buffer_contains, mock_cluster_info, mock_tiers};
use picotui::app::{App, InputMode, SortField, SortOrder, ViewMode};
use picotui::models::{ClusterInfo, TierInfo};
use picotui::ui;
use ratatui::{backend::TestBackend, Terminal};
use std::sync::mpsc::channel;

/// Create a test app with mock data loaded
fn test_app_with_data() -> App {
    let (req_tx, _req_rx) = channel();
    let (_res_tx, res_rx) = channel();
    let mut app = App::new("http://test:8080".to_string(), req_tx, res_rx);

    // Load mock data
    let cluster_info: ClusterInfo = serde_json::from_value(mock_cluster_info()).unwrap();
    let tiers: Vec<TierInfo> = serde_json::from_value(mock_tiers()).unwrap();

    app.cluster_info = Some(cluster_info);
    app.tiers = tiers;
    app.rebuild_tree();
    app.input_mode = InputMode::Normal;

    app
}

/// Create a terminal with TestBackend
fn test_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    let backend = TestBackend::new(width, height);
    Terminal::new(backend).unwrap()
}

#[test]
fn test_tiers_view_renders_cluster_info() {
    let mut terminal = test_terminal(100, 30);
    let mut app = test_app_with_data();

    terminal.draw(|f| ui::draw(f, &mut app)).unwrap();

    let buffer = terminal.backend().buffer();

    // Check cluster info is displayed
    assert!(
        buffer_contains(buffer, "test-cluster"),
        "Should show cluster name"
    );
    assert!(
        buffer_contains(buffer, "1.0.0"),
        "Should show cluster version"
    );
}

#[test]
fn test_tiers_view_renders_tiers() {
    let mut terminal = test_terminal(100, 30);
    let mut app = test_app_with_data();

    terminal.draw(|f| ui::draw(f, &mut app)).unwrap();

    let buffer = terminal.backend().buffer();

    // Check tier names are displayed
    assert!(
        buffer_contains(buffer, "default"),
        "Should show 'default' tier"
    );
    assert!(
        buffer_contains(buffer, "storage"),
        "Should show 'storage' tier"
    );
}

#[test]
fn test_tiers_view_shows_collapsed_arrows() {
    let mut terminal = test_terminal(100, 30);
    let mut app = test_app_with_data();

    terminal.draw(|f| ui::draw(f, &mut app)).unwrap();

    let buffer = terminal.backend().buffer();

    // Collapsed tiers should show right arrow
    assert!(buffer_contains(buffer, "▶"), "Should show collapsed arrow");
}

#[test]
fn test_tiers_view_expanded_shows_replicasets() {
    let mut terminal = test_terminal(100, 30);
    let mut app = test_app_with_data();

    // Expand first tier
    app.expanded_tiers.insert(0);
    app.rebuild_tree();

    terminal.draw(|f| ui::draw(f, &mut app)).unwrap();

    let buffer = terminal.backend().buffer();

    // Should show expanded arrow and replicaset names
    assert!(buffer_contains(buffer, "▼"), "Should show expanded arrow");
    assert!(buffer_contains(buffer, "r1"), "Should show replicaset r1");
    assert!(buffer_contains(buffer, "r2"), "Should show replicaset r2");
}

#[test]
fn test_tiers_view_expanded_shows_instances() {
    let mut terminal = test_terminal(100, 30);
    let mut app = test_app_with_data();

    // Expand tier and replicaset
    app.expanded_tiers.insert(0);
    app.expanded_replicasets.insert((0, 0));
    app.rebuild_tree();

    terminal.draw(|f| ui::draw(f, &mut app)).unwrap();

    let buffer = terminal.backend().buffer();

    // Should show instance names
    assert!(buffer_contains(buffer, "i1"), "Should show instance i1");
    assert!(buffer_contains(buffer, "i2"), "Should show instance i2");
    // Leader should have star
    assert!(buffer_contains(buffer, "★"), "Should show leader star");
}

#[test]
fn test_replicasets_view_renders() {
    let mut terminal = test_terminal(100, 30);
    let mut app = test_app_with_data();

    app.view_mode = ViewMode::Replicasets;

    terminal.draw(|f| ui::draw(f, &mut app)).unwrap();

    let buffer = terminal.backend().buffer();

    // Check view title
    assert!(
        buffer_contains(buffer, "Replicasets"),
        "Should show Replicasets title"
    );

    // Check all replicasets are listed
    assert!(buffer_contains(buffer, "r1"), "Should show replicaset r1");
    assert!(buffer_contains(buffer, "r2"), "Should show replicaset r2");
    assert!(buffer_contains(buffer, "s1"), "Should show replicaset s1");
}

#[test]
fn test_instances_view_renders() {
    let mut terminal = test_terminal(100, 30);
    let mut app = test_app_with_data();

    app.view_mode = ViewMode::Instances;

    terminal.draw(|f| ui::draw(f, &mut app)).unwrap();

    let buffer = terminal.backend().buffer();

    // Check view title
    assert!(
        buffer_contains(buffer, "Instances"),
        "Should show Instances title"
    );

    // Check all instances are listed
    assert!(buffer_contains(buffer, "i1"), "Should show instance i1");
    assert!(buffer_contains(buffer, "i2"), "Should show instance i2");
    assert!(buffer_contains(buffer, "i3"), "Should show instance i3");
    assert!(buffer_contains(buffer, "i4"), "Should show instance i4");
    assert!(
        buffer_contains(buffer, "s1-i1"),
        "Should show instance s1-i1"
    );
    assert!(
        buffer_contains(buffer, "s1-i2"),
        "Should show instance s1-i2"
    );
}

#[test]
fn test_instances_view_shows_sort_indicator() {
    let mut terminal = test_terminal(100, 30);
    let mut app = test_app_with_data();

    app.view_mode = ViewMode::Instances;
    app.sort_field = SortField::Name;
    app.sort_order = SortOrder::Asc;

    terminal.draw(|f| ui::draw(f, &mut app)).unwrap();

    let buffer = terminal.backend().buffer();

    // Check sort indicator
    assert!(buffer_contains(buffer, "Sort:"), "Should show sort label");
    assert!(buffer_contains(buffer, "Name"), "Should show sort field");
    assert!(buffer_contains(buffer, "↑"), "Should show ascending arrow");
}

#[test]
fn test_instances_view_sort_descending() {
    let mut terminal = test_terminal(100, 30);
    let mut app = test_app_with_data();

    app.view_mode = ViewMode::Instances;
    app.sort_order = SortOrder::Desc;

    terminal.draw(|f| ui::draw(f, &mut app)).unwrap();

    let buffer = terminal.backend().buffer();

    assert!(buffer_contains(buffer, "↓"), "Should show descending arrow");
}

#[test]
fn test_instances_view_filter_shows_indicator() {
    let mut terminal = test_terminal(100, 30);
    let mut app = test_app_with_data();

    app.view_mode = ViewMode::Instances;
    app.filter_text = "dc1".to_string();

    terminal.draw(|f| ui::draw(f, &mut app)).unwrap();

    let buffer = terminal.backend().buffer();

    // Check filter indicator in title
    assert!(
        buffer_contains(buffer, "Filter:"),
        "Should show filter label"
    );
    assert!(buffer_contains(buffer, "dc1"), "Should show filter text");
}

#[test]
fn test_instances_view_filter_active_shows_cursor() {
    let mut terminal = test_terminal(100, 30);
    let mut app = test_app_with_data();

    app.view_mode = ViewMode::Instances;
    app.filter_active = true;
    app.filter_text = "test".to_string();

    terminal.draw(|f| ui::draw(f, &mut app)).unwrap();

    let buffer = terminal.backend().buffer();

    // Check filter input in status bar
    assert!(
        buffer_contains(buffer, "Filter:"),
        "Should show filter in status bar"
    );
    assert!(buffer_contains(buffer, "test"), "Should show filter text");
    // Cursor indicator
    assert!(buffer_contains(buffer, "█"), "Should show cursor");
}

#[test]
fn test_login_screen_renders() {
    let mut terminal = test_terminal(80, 24);
    let (req_tx, _req_rx) = channel();
    let (_res_tx, res_rx) = channel();
    let mut app = App::new("http://test:8080".to_string(), req_tx, res_rx);

    app.input_mode = InputMode::Login;
    app.auth_enabled = true;

    terminal.draw(|f| ui::draw(f, &mut app)).unwrap();

    let buffer = terminal.backend().buffer();

    // Check login form elements
    assert!(buffer_contains(buffer, "Login"), "Should show Login title");
    assert!(
        buffer_contains(buffer, "Username"),
        "Should show Username field"
    );
    assert!(
        buffer_contains(buffer, "Password"),
        "Should show Password field"
    );
    assert!(
        buffer_contains(buffer, "Remember"),
        "Should show Remember me checkbox"
    );
}

#[test]
fn test_view_mode_indicator_in_header() {
    let mut terminal = test_terminal(100, 30);
    let mut app = test_app_with_data();

    // Test each view mode shows correct indicator
    for (mode, label) in [
        (ViewMode::Tiers, "Tiers"),
        (ViewMode::Replicasets, "Replicasets"),
        (ViewMode::Instances, "Instances"),
    ] {
        app.view_mode = mode;
        terminal.draw(|f| ui::draw(f, &mut app)).unwrap();

        let buffer = terminal.backend().buffer();
        assert!(
            buffer_contains(buffer, label),
            "Should show {} mode indicator",
            label
        );
    }
}

#[test]
fn test_status_bar_shows_keybindings() {
    let mut terminal = test_terminal(120, 30);
    let mut app = test_app_with_data();

    terminal.draw(|f| ui::draw(f, &mut app)).unwrap();

    let buffer = terminal.backend().buffer();

    // Check common keybindings are shown
    assert!(
        buffer_contains(buffer, "Navigate"),
        "Should show Navigate hint"
    );
    assert!(
        buffer_contains(buffer, "Refresh"),
        "Should show Refresh hint"
    );
    assert!(buffer_contains(buffer, "Quit"), "Should show Quit hint");
}

#[test]
fn test_instances_view_status_bar_shows_filter_key() {
    let mut terminal = test_terminal(120, 30);
    let mut app = test_app_with_data();

    app.view_mode = ViewMode::Instances;

    terminal.draw(|f| ui::draw(f, &mut app)).unwrap();

    let buffer = terminal.backend().buffer();

    // Check filter keybinding is shown in Instances view
    assert!(buffer_contains(buffer, "Filter"), "Should show Filter hint");
    assert!(buffer_contains(buffer, "Sort"), "Should show Sort hint");
}

#[test]
fn test_offline_instance_shown_differently() {
    let mut terminal = test_terminal(100, 30);
    let mut app = test_app_with_data();

    app.view_mode = ViewMode::Instances;

    terminal.draw(|f| ui::draw(f, &mut app)).unwrap();

    let buffer = terminal.backend().buffer();

    // Instance i3 is offline
    assert!(
        buffer_contains(buffer, "i3"),
        "Should show offline instance"
    );
    assert!(
        buffer_contains(buffer, "Offline"),
        "Should show Offline state"
    );
}

#[test]
fn test_memory_usage_displayed() {
    let mut terminal = test_terminal(100, 30);
    let mut app = test_app_with_data();

    terminal.draw(|f| ui::draw(f, &mut app)).unwrap();

    let buffer = terminal.backend().buffer();

    // Memory bar should be visible in cluster header
    assert!(buffer_contains(buffer, "GiB"), "Should show memory in GiB");
}
