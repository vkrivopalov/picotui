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
    /// State of the replicaset's leader instance (kept for backward compatibility).
    pub state: StateVariant,
    /// Actual replicaset state from _pico_replicaset (Picodata 26.2+).
    /// Defaults to Ready when missing (older Picodata versions).
    #[serde(default)]
    pub replicaset_state: ReplicasetState,
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
    /// Whether this instance is the vshard leader of its replicaset.
    pub is_leader: bool,
    /// Whether this instance is a Raft voter (Picodata 26.2+).
    /// Defaults to false when missing (older Picodata versions).
    #[serde(default)]
    pub is_voter: bool,
    /// Whether this instance is the Raft leader (Picodata 26.2+).
    /// Defaults to false when missing (older Picodata versions).
    #[serde(default)]
    pub is_raft_leader: bool,
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

/// Replicaset state from _pico_replicaset system table.
/// Note: This is different from the `state` field in ReplicasetInfo,
/// which represents the leader instance's state (kept for backward compatibility).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ReplicasetState {
    #[default]
    Ready,
    NotReady,
}

impl std::fmt::Display for ReplicasetState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReplicasetState::Ready => write!(f, "Ready"),
            ReplicasetState::NotReady => write!(f, "Not Ready"),
        }
    }
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
    #[allow(dead_code)]
    pub error_message: String,
}

// Health status types (from /api/v1/health/status)

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatusLevel {
    Healthy,
    Degraded,
    Unhealthy,
}

impl std::fmt::Display for HealthStatusLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatusLevel::Healthy => write!(f, "Healthy"),
            HealthStatusLevel::Degraded => write!(f, "Degraded"),
            HealthStatusLevel::Unhealthy => write!(f, "Unhealthy"),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthStatus {
    pub status: HealthStatusLevel,
    #[serde(default)]
    pub reasons: Vec<String>,
    pub uptime_seconds: u64,
    pub name: String,
    #[allow(dead_code)]
    pub uuid: String,
    pub version: String,
    pub raft_id: u64,
    pub tier: String,
    pub replicaset: String,
    pub current_state: String,
    pub target_state: String,
    pub target_state_reason: Option<String>,
    pub limbo_owner: u64,
    pub raft: RaftStatus,
    pub buckets: BucketStatus,
    pub cluster: ClusterHealthInfo,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RaftStatus {
    pub state: String,
    pub term: u64,
    pub leader_id: u64,
    pub leader_name: String,
    pub applied_index: u64,
    #[serde(rename = "commitedIndex")] // typo in picodata
    pub committed_index: u64,
    pub compacted_index: u64,
    pub persisted_index: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BucketStatus {
    pub active: usize,
    pub pinned: usize,
    pub sending: usize,
    pub receiving: usize,
    pub garbage: usize,
    pub total: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClusterHealthInfo {
    #[allow(dead_code)]
    pub uuid: String,
    pub version: String,
}
