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
            login_error: None,
            cluster_info: None,
            tiers: Vec::new(),
            last_error: None,
            expanded_tiers: HashSet::new(),
            expanded_replicasets: HashSet::new(),
            tree_items: Vec::new(),
            selected_index: 0,
            show_detail: false,
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
                                self.input_mode = InputMode::Login;
                                self.login_error = Some("Session expired, please login again".to_string());
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
        if !self.tree_items.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.tree_items.len();
        }
    }

    pub fn select_previous(&mut self) {
        if !self.tree_items.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.tree_items.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    pub fn expand_selected(&mut self) {
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

    pub fn collapse_selected(&mut self) {
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
        self.show_detail = !self.show_detail;
    }

    pub fn get_selected_instance(&self) -> Option<&InstanceInfo> {
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

    pub fn shutdown(&self) {
        let _ = self.request_tx.send(ApiRequest::Shutdown);
    }
}
