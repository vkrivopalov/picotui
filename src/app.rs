use crate::api::{ApiRequest, ApiResponse};
use crate::models::*;
use crate::tokens;
use std::collections::HashSet;
use std::sync::mpsc::{Receiver, Sender};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Login,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginFocus {
    Username,
    Password,
    RememberMe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    #[default]
    Tiers,
    Replicasets,
    Instances,
}

impl ViewMode {
    pub fn cycle_next(self) -> Self {
        match self {
            ViewMode::Tiers => ViewMode::Replicasets,
            ViewMode::Replicasets => ViewMode::Instances,
            ViewMode::Instances => ViewMode::Tiers,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ViewMode::Tiers => "Tiers",
            ViewMode::Replicasets => "Replicasets",
            ViewMode::Instances => "Instances",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortField {
    #[default]
    Name,
    FailureDomain,
}

impl SortField {
    pub fn cycle_next(self) -> Self {
        match self {
            SortField::Name => SortField::FailureDomain,
            SortField::FailureDomain => SortField::Name,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SortField::Name => "Name",
            SortField::FailureDomain => "Domain",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortOrder {
    #[default]
    Asc,
    Desc,
}

impl SortOrder {
    pub fn toggle(self) -> Self {
        match self {
            SortOrder::Asc => SortOrder::Desc,
            SortOrder::Desc => SortOrder::Asc,
        }
    }

    pub fn arrow(self) -> &'static str {
        match self {
            SortOrder::Asc => "↑",
            SortOrder::Desc => "↓",
        }
    }
}

#[derive(Debug, Clone)]
pub enum TreeItem {
    Tier(usize),
    Replicaset(usize, usize),
    Instance(usize, usize, usize),
}

pub struct App {
    pub running: bool,

    // Connection info
    pub base_url: String,

    // Channels for API communication
    pub request_tx: Sender<ApiRequest>,
    pub response_rx: Receiver<ApiResponse>,

    // Loading state
    pub loading: bool,
    pub pending_init: bool,

    // Input mode
    pub input_mode: InputMode,

    // Auth
    pub auth_enabled: bool,
    pub has_saved_token: bool,
    pub login_username: String,
    pub login_password: String,
    pub login_focus: LoginFocus,
    pub login_remember_me: bool,
    pub login_show_password: bool,
    pub login_error: Option<String>,

    // Data
    pub cluster_info: Option<ClusterInfo>,
    pub tiers: Vec<TierInfo>,
    pub last_error: Option<String>,

    // Tree state
    pub expanded_tiers: HashSet<usize>,
    pub expanded_replicasets: HashSet<(usize, usize)>,
    pub tree_items: Vec<TreeItem>,
    pub selected_index: usize,

    // Detail popup
    pub show_detail: bool,

    // View mode
    pub view_mode: ViewMode,

    // Sorting (instances view)
    pub sort_field: SortField,
    pub sort_order: SortOrder,

    // Filtering (instances view)
    pub filter_text: String,
    pub filter_active: bool,
}

impl App {
    pub fn new(
        base_url: String,
        request_tx: Sender<ApiRequest>,
        response_rx: Receiver<ApiResponse>,
    ) -> Self {
        // Check for saved token
        let saved_token = tokens::load_tokens(&base_url);
        let has_saved_token = saved_token.is_some();

        // If we have a saved token, send it to the API worker
        if let Some(token_entry) = saved_token {
            let _ = request_tx.send(ApiRequest::SetToken {
                auth: token_entry.auth,
                refresh: token_entry.refresh,
            });
        }

        Self {
            running: true,
            base_url,
            request_tx,
            response_rx,
            loading: false,
            pending_init: true,
            input_mode: InputMode::Normal,
            auth_enabled: false,
            has_saved_token,
            login_username: String::new(),
            login_password: String::new(),
            login_focus: LoginFocus::Username,
            login_remember_me: true,
            login_show_password: false,
            login_error: None,
            cluster_info: None,
            tiers: Vec::new(),
            last_error: None,
            expanded_tiers: HashSet::new(),
            expanded_replicasets: HashSet::new(),
            tree_items: Vec::new(),
            selected_index: 0,
            show_detail: false,
            view_mode: ViewMode::default(),
            sort_field: SortField::default(),
            sort_order: SortOrder::default(),
            filter_text: String::new(),
            filter_active: false,
        }
    }

    /// Start initialization by requesting config
    pub fn start_init(&mut self) {
        self.loading = true;
        self.pending_init = true;
        let _ = self.request_tx.send(ApiRequest::GetConfig);
    }

    /// Request a data refresh (non-blocking)
    pub fn request_refresh(&mut self) {
        self.loading = true;
        self.last_error = None;
        let _ = self.request_tx.send(ApiRequest::GetClusterInfo);
        let _ = self.request_tx.send(ApiRequest::GetTiers);
    }

    /// Request login (non-blocking)
    pub fn request_login(&mut self) {
        self.loading = true;
        self.login_error = None;
        let _ = self.request_tx.send(ApiRequest::Login {
            username: self.login_username.clone(),
            password: self.login_password.clone(),
            remember_me: self.login_remember_me,
        });
    }

    /// Logout, clear saved tokens, and exit
    pub fn logout(&mut self) {
        // Delete tokens directly (don't rely on worker thread)
        let _ = tokens::delete_tokens(&self.base_url);
        self.running = false;
    }

    /// Process any pending API responses (non-blocking)
    pub fn process_responses(&mut self) {
        use std::sync::mpsc::TryRecvError;

        loop {
            match self.response_rx.try_recv() {
                Ok(response) => self.handle_response(response),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.last_error = Some("API worker disconnected".to_string());
                    break;
                }
            }
        }
    }

    fn handle_response(&mut self, response: ApiResponse) {
        match response {
            ApiResponse::Config(result) => {
                self.loading = false;
                match result {
                    Ok(config) => {
                        self.auth_enabled = config.is_auth_enabled;
                        if self.auth_enabled {
                            if self.has_saved_token {
                                // Try using saved token - fetch data directly
                                // If it fails with 401, we'll show login
                                self.request_refresh();
                                self.pending_init = false;
                            } else {
                                self.input_mode = InputMode::Login;
                                self.pending_init = false;
                            }
                        } else {
                            // No auth needed, request data
                            self.request_refresh();
                            self.pending_init = false;
                        }
                    }
                    Err(e) => {
                        self.last_error = Some(format!("Failed to connect: {}", e));
                        self.pending_init = false;
                    }
                }
            }

            ApiResponse::Login(result) => {
                self.loading = false;
                match result {
                    Ok(_) => {
                        self.input_mode = InputMode::Normal;
                        self.login_password.clear();
                        self.request_refresh();
                    }
                    Err(e) => {
                        self.login_error = Some(e);
                    }
                }
            }

            ApiResponse::ClusterInfo(result) => {
                match result {
                    Ok(info) => {
                        self.cluster_info = Some(info);
                        self.last_error = None;
                    }
                    Err(e) => {
                        // Check if this is an auth error (401)
                        if e.contains("401") || e.to_lowercase().contains("unauthorized") {
                            if self.has_saved_token {
                                // Saved token is invalid, need to re-login
                                self.has_saved_token = false;
                                self.loading = false;
                                self.input_mode = InputMode::Login;
                                self.login_error =
                                    Some("Session expired, please login again".to_string());
                                // Clear invalid token from disk
                                let _ = tokens::delete_tokens(&self.base_url);
                            }
                        } else {
                            self.last_error = Some(format!("Cluster: {}", e));
                        }
                    }
                }
                // Check if all data loaded
                self.check_loading_complete();
            }

            ApiResponse::Tiers(result) => {
                match result {
                    Ok(tiers) => {
                        self.tiers = tiers;
                        self.rebuild_tree();
                    }
                    Err(e) => {
                        // Check if this is an auth error (401)
                        if (e.contains("401") || e.to_lowercase().contains("unauthorized"))
                            && self.has_saved_token
                        {
                            // Saved token is invalid, need to re-login
                            self.has_saved_token = false;
                            self.loading = false;
                            self.input_mode = InputMode::Login;
                            self.login_error =
                                Some("Session expired, please login again".to_string());
                            // Clear invalid token from disk
                            let _ = tokens::delete_tokens(&self.base_url);
                            return;
                        }
                        if self.last_error.is_none() {
                            self.last_error = Some(format!("Tiers: {}", e));
                        }
                    }
                }
                // Check if all data loaded
                self.check_loading_complete();
            }
        }
    }

    fn check_loading_complete(&mut self) {
        // Simple heuristic: loading complete when we have cluster info
        if self.cluster_info.is_some() {
            self.loading = false;
        }
    }

    pub fn rebuild_tree(&mut self) {
        self.tree_items.clear();

        for (tier_idx, tier) in self.tiers.iter().enumerate() {
            self.tree_items.push(TreeItem::Tier(tier_idx));

            if self.expanded_tiers.contains(&tier_idx) {
                for (rs_idx, replicaset) in tier.replicasets.iter().enumerate() {
                    self.tree_items.push(TreeItem::Replicaset(tier_idx, rs_idx));

                    if self.expanded_replicasets.contains(&(tier_idx, rs_idx)) {
                        for inst_idx in 0..replicaset.instances.len() {
                            self.tree_items
                                .push(TreeItem::Instance(tier_idx, rs_idx, inst_idx));
                        }
                    }
                }
            }
        }

        // Clamp selection
        if !self.tree_items.is_empty() && self.selected_index >= self.tree_items.len() {
            self.selected_index = self.tree_items.len() - 1;
        }
    }

    pub fn select_next(&mut self) {
        let count = self.get_item_count();
        if count > 0 {
            self.selected_index = (self.selected_index + 1) % count;
        }
    }

    pub fn select_previous(&mut self) {
        let count = self.get_item_count();
        if count > 0 {
            self.selected_index = if self.selected_index == 0 {
                count - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    pub fn expand_selected(&mut self) {
        match self.view_mode {
            ViewMode::Tiers => {
                if let Some(item) = self.tree_items.get(self.selected_index) {
                    match item {
                        TreeItem::Tier(tier_idx) => {
                            self.expanded_tiers.insert(*tier_idx);
                            self.rebuild_tree();
                        }
                        TreeItem::Replicaset(tier_idx, rs_idx) => {
                            self.expanded_replicasets.insert((*tier_idx, *rs_idx));
                            self.rebuild_tree();
                        }
                        TreeItem::Instance(_, _, _) => {
                            self.show_detail = true;
                        }
                    }
                }
            }
            ViewMode::Replicasets => {
                // Could expand to show instances, but for now do nothing
            }
            ViewMode::Instances => {
                self.show_detail = true;
            }
        }
    }

    pub fn collapse_selected(&mut self) {
        if self.view_mode != ViewMode::Tiers {
            return;
        }

        if let Some(item) = self.tree_items.get(self.selected_index) {
            match item {
                TreeItem::Tier(tier_idx) => {
                    self.expanded_tiers.remove(tier_idx);
                    // Also collapse all replicasets in this tier
                    self.expanded_replicasets.retain(|(t, _)| *t != *tier_idx);
                    self.rebuild_tree();
                }
                TreeItem::Replicaset(tier_idx, rs_idx) => {
                    self.expanded_replicasets.remove(&(*tier_idx, *rs_idx));
                    self.rebuild_tree();
                }
                TreeItem::Instance(tier_idx, rs_idx, _) => {
                    // Collapse parent replicaset
                    self.expanded_replicasets.remove(&(*tier_idx, *rs_idx));
                    self.rebuild_tree();
                }
            }
        }
    }

    pub fn toggle_detail(&mut self) {
        // Only show detail if we can get an instance
        match self.view_mode {
            ViewMode::Tiers => {
                // Only toggle if an instance is selected
                if let Some(TreeItem::Instance(_, _, _)) = self.tree_items.get(self.selected_index)
                {
                    self.show_detail = !self.show_detail;
                }
            }
            ViewMode::Replicasets => {
                // Can't show instance detail in replicasets view
            }
            ViewMode::Instances => {
                self.show_detail = !self.show_detail;
            }
        }
    }

    pub fn get_selected_instance(&self) -> Option<&InstanceInfo> {
        match self.view_mode {
            ViewMode::Tiers => {
                if let Some(TreeItem::Instance(tier_idx, rs_idx, inst_idx)) =
                    self.tree_items.get(self.selected_index)
                {
                    self.tiers
                        .get(*tier_idx)
                        .and_then(|t| t.replicasets.get(*rs_idx))
                        .and_then(|r| r.instances.get(*inst_idx))
                } else {
                    None
                }
            }
            ViewMode::Replicasets => None, // Can't select instance in replicasets view
            ViewMode::Instances => {
                // Get sorted instances and select by index
                let instances = self.get_sorted_instances();
                instances.get(self.selected_index).map(|(_, _, inst)| *inst)
            }
        }
    }

    /// Get sorted and filtered instances for Instances view
    pub fn get_sorted_instances(&self) -> Vec<(&str, &str, &InstanceInfo)> {
        let filter_lower = self.filter_text.to_lowercase();

        let mut instances: Vec<(&str, &str, &InstanceInfo)> = self
            .tiers
            .iter()
            .flat_map(|tier| {
                tier.replicasets.iter().flat_map(move |rs| {
                    rs.instances
                        .iter()
                        .map(move |inst| (tier.name.as_str(), rs.name.as_str(), inst))
                })
            })
            .filter(|(tier_name, rs_name, inst)| {
                if filter_lower.is_empty() {
                    return true;
                }
                // Match against instance name, tier, replicaset, address, or failure domain
                inst.name.to_lowercase().contains(&filter_lower)
                    || tier_name.to_lowercase().contains(&filter_lower)
                    || rs_name.to_lowercase().contains(&filter_lower)
                    || inst.binary_address.to_lowercase().contains(&filter_lower)
                    || inst
                        .failure_domain
                        .values()
                        .any(|v| v.to_lowercase().contains(&filter_lower))
            })
            .collect();

        // Sort based on current sort settings
        match self.sort_field {
            SortField::Name => {
                instances.sort_by(|a, b| {
                    let cmp = a.2.name.cmp(&b.2.name);
                    if self.sort_order == SortOrder::Desc {
                        cmp.reverse()
                    } else {
                        cmp
                    }
                });
            }
            SortField::FailureDomain => {
                instances.sort_by(|a, b| {
                    let domain_a = Self::format_failure_domain(&a.2.failure_domain);
                    let domain_b = Self::format_failure_domain(&b.2.failure_domain);
                    let cmp = domain_a.cmp(&domain_b);
                    // If domains are equal, sort by name
                    let cmp = if cmp == std::cmp::Ordering::Equal {
                        a.2.name.cmp(&b.2.name)
                    } else {
                        cmp
                    };
                    if self.sort_order == SortOrder::Desc {
                        cmp.reverse()
                    } else {
                        cmp
                    }
                });
            }
        }

        instances
    }

    fn format_failure_domain(domain: &std::collections::HashMap<String, String>) -> String {
        if domain.is_empty() {
            return String::new();
        }
        let mut pairs: Vec<_> = domain.iter().collect();
        pairs.sort_by(|a, b| a.0.cmp(b.0));
        pairs
            .iter()
            .map(|(k, v)| format!("{}:{}", k, v))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Get the total number of items in the current view
    pub fn get_item_count(&self) -> usize {
        match self.view_mode {
            ViewMode::Tiers => self.tree_items.len(),
            ViewMode::Replicasets => self.tiers.iter().map(|t| t.replicasets.len()).sum(),
            ViewMode::Instances => self
                .tiers
                .iter()
                .flat_map(|t| t.replicasets.iter())
                .map(|r| r.instances.len())
                .sum(),
        }
    }

    pub fn shutdown(&self) {
        let _ = self.request_tx.send(ApiRequest::Shutdown);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::channel;

    /// Create a test app with saved token state
    fn test_app_with_saved_token() -> App {
        let (req_tx, _req_rx) = channel();
        let (_res_tx, res_rx) = channel();
        let mut app = App::new("http://test:8080".to_string(), req_tx, res_rx);
        app.has_saved_token = true;
        app.loading = true;
        app.auth_enabled = true;
        app.input_mode = InputMode::Normal;
        app
    }

    #[test]
    fn test_401_error_on_cluster_info_allows_relogin() {
        let mut app = test_app_with_saved_token();

        // Simulate receiving a 401 error from ClusterInfo
        app.handle_response(ApiResponse::ClusterInfo(Err(
            "HTTP 401 Unauthorized".to_string()
        )));

        // Verify the app is ready for login
        assert!(
            !app.loading,
            "loading should be false to allow login submission"
        );
        assert!(!app.has_saved_token, "has_saved_token should be cleared");
        assert_eq!(
            app.input_mode,
            InputMode::Login,
            "should switch to login mode"
        );
        assert!(app.login_error.is_some(), "should have login error message");
        assert!(
            app.login_error
                .as_ref()
                .unwrap()
                .contains("Session expired"),
            "error should mention session expired"
        );
    }

    #[test]
    fn test_401_error_on_tiers_allows_relogin() {
        let mut app = test_app_with_saved_token();

        // Simulate receiving a 401 error from Tiers
        app.handle_response(ApiResponse::Tiers(Err("HTTP 401 Unauthorized".to_string())));

        // Verify the app is ready for login
        assert!(
            !app.loading,
            "loading should be false to allow login submission"
        );
        assert!(!app.has_saved_token, "has_saved_token should be cleared");
        assert_eq!(
            app.input_mode,
            InputMode::Login,
            "should switch to login mode"
        );
        assert!(app.login_error.is_some(), "should have login error message");
    }

    #[test]
    fn test_non_401_error_does_not_trigger_relogin() {
        let mut app = test_app_with_saved_token();

        // Simulate receiving a non-401 error
        app.handle_response(ApiResponse::ClusterInfo(Err(
            "HTTP 500 Internal Server Error".to_string(),
        )));

        // Should NOT switch to login mode
        assert!(app.has_saved_token, "has_saved_token should remain true");
        assert_eq!(
            app.input_mode,
            InputMode::Normal,
            "should stay in normal mode"
        );
        assert!(app.login_error.is_none(), "should not have login error");
        assert!(app.last_error.is_some(), "should have last_error set");
    }
}
