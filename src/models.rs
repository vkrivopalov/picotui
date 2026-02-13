use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClusterInfo {
    pub capacity_usage: f64,
    pub cluster_name: String,
    pub cluster_version: String,
    #[serde(rename = "currentInstaceVersion")]
    pub current_instance_version: String,
    pub replicasets_count: usize,
    pub instances_current_state_offline: usize,
    pub instances_current_state_online: usize,
    pub memory: MemoryInfo,
    pub plugins: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TierInfo {
    pub replicasets: Vec<ReplicasetInfo>,
    pub replicaset_count: usize,
    pub rf: u8,
    pub bucket_count: u64,
    pub instance_count: usize,
    #[serde(rename = "can_vote")]
    pub can_vote: bool,
    pub name: String,
    #[allow(dead_code)]
    pub services: Vec<String>,
    pub memory: MemoryInfo,
    pub capacity_usage: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplicasetInfo {
    #[allow(dead_code)]
    pub version: String,
    pub state: StateVariant,
    pub instance_count: usize,
    #[allow(dead_code)]
    pub uuid: String,
    pub instances: Vec<InstanceInfo>,
    pub capacity_usage: f64,
    pub memory: MemoryInfo,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceInfo {
    pub http_address: String,
    pub version: String,
    pub failure_domain: HashMap<String, String>,
    pub is_leader: bool,
    pub current_state: StateVariant,
    pub target_state: StateVariant,
    pub name: String,
    pub binary_address: String,
    pub pg_address: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum StateVariant {
    Online,
    Offline,
    Expelled,
}

impl std::fmt::Display for StateVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateVariant::Online => write!(f, "Online"),
            StateVariant::Offline => write!(f, "Offline"),
            StateVariant::Expelled => write!(f, "Expelled"),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct MemoryInfo {
    pub usable: u64,
    pub used: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TokenResponse {
    pub auth: String,
    #[allow(dead_code)]
    pub refresh: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiConfig {
    pub is_auth_enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    #[allow(dead_code)]
    pub error: String,
    pub error_message: String,
}
