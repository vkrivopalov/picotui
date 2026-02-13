use crate::api::PicodataClient;
use crate::models::*;
use anyhow::Result;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Login,
}

#[derive(Debug, Clone)]
pub enum TreeItem {
    Tier(usize),
    Replicaset(usize, usize),
    Instance(usize, usize, usize),
}

pub struct App {
    pub client: PicodataClient,
    pub running: bool,

    // Input mode
    pub input_mode: InputMode,

    // Auth
    pub auth_enabled: bool,
    pub login_username: String,
    pub login_password: String,
    pub login_focus_password: bool,
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
    pub fn new(base_url: &str, debug: bool) -> Result<Self> {
        let client = PicodataClient::new(base_url, debug)?;
        Ok(Self {
            client,
            running: true,
            input_mode: InputMode::Normal,
            auth_enabled: false,
            login_username: String::new(),
            login_password: String::new(),
            login_focus_password: false,
            login_error: None,
            cluster_info: None,
            tiers: Vec::new(),
            last_error: None,
            expanded_tiers: HashSet::new(),
            expanded_replicasets: HashSet::new(),
            tree_items: Vec::new(),
            selected_index: 0,
            show_detail: false,
        })
    }

    pub async fn init(&mut self) -> Result<()> {
        match self.client.get_config().await {
            Ok(config) => {
                self.auth_enabled = config.is_auth_enabled;
                if self.auth_enabled {
                    self.input_mode = InputMode::Login;
                } else {
                    self.refresh_data().await?;
                }
            }
            Err(e) => {
                self.last_error = Some(format!("Failed to connect: {}", e));
            }
        }
        Ok(())
    }

    pub async fn refresh_data(&mut self) -> Result<()> {
        self.last_error = None;

        match self.client.get_cluster_info().await {
            Ok(info) => self.cluster_info = Some(info),
            Err(e) => {
                self.last_error = Some(format!("Cluster: {}", e));
                return Ok(());
            }
        }

        match self.client.get_tiers().await {
            Ok(tiers) => {
                self.tiers = tiers;
                self.rebuild_tree();
            }
            Err(e) => {
                self.last_error = Some(format!("Tiers: {}", e));
            }
        }

        Ok(())
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

    pub async fn do_login(&mut self) {
        self.login_error = None;
        match self
            .client
            .login(&self.login_username, &self.login_password)
            .await
        {
            Ok(_) => {
                self.input_mode = InputMode::Normal;
                self.login_password.clear();
                let _ = self.refresh_data().await;
            }
            Err(e) => {
                self.login_error = Some(e.to_string());
            }
        }
    }
}
