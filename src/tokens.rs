use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenEntry {
    pub auth: String,
    pub refresh: String,
    pub saved_at: u64,
}

/// Get the path to the tokens file
fn token_file_path() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("picotui/tokens.json"))
}

/// Save tokens for a given URL
pub fn save_tokens(url: &str, auth: &str, refresh: &str) -> anyhow::Result<()> {
    let path = token_file_path().ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

    // Create parent directory with restricted permissions
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
        #[cfg(unix)]
        fs::set_permissions(parent, fs::Permissions::from_mode(0o700))?;
    }

    // Load existing tokens or create new map
    let mut tokens: HashMap<String, TokenEntry> = if path.exists() {
        let file = File::open(&path)?;
        serde_json::from_reader(file).unwrap_or_default()
    } else {
        HashMap::new()
    };

    // Normalize URL (remove trailing slash)
    let normalized_url = url.trim_end_matches('/').to_string();

    // Insert/update token entry
    tokens.insert(
        normalized_url,
        TokenEntry {
            auth: auth.to_string(),
            refresh: refresh.to_string(),
            saved_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        },
    );

    // Write file with restricted permissions (owner read/write only)
    #[cfg(unix)]
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(&path)?;

    #[cfg(not(unix))]
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)?;

    serde_json::to_writer_pretty(file, &tokens)?;

    Ok(())
}

/// Load tokens for a given URL
pub fn load_tokens(url: &str) -> Option<TokenEntry> {
    let path = token_file_path()?;
    let file = File::open(path).ok()?;
    let tokens: HashMap<String, TokenEntry> = serde_json::from_reader(file).ok()?;

    // Normalize URL (remove trailing slash)
    let normalized_url = url.trim_end_matches('/');

    tokens.get(normalized_url).cloned()
}

/// Delete tokens for a given URL
#[allow(dead_code)]
pub fn delete_tokens(url: &str) -> anyhow::Result<()> {
    let path = token_file_path().ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

    if !path.exists() {
        return Ok(());
    }

    let file = File::open(&path)?;
    let mut tokens: HashMap<String, TokenEntry> = serde_json::from_reader(file).unwrap_or_default();

    // Normalize URL (remove trailing slash)
    let normalized_url = url.trim_end_matches('/');
    tokens.remove(normalized_url);

    // Write back
    #[cfg(unix)]
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(&path)?;

    #[cfg(not(unix))]
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)?;

    serde_json::to_writer_pretty(file, &tokens)?;

    Ok(())
}
