use crate::models::*;
use anyhow::{anyhow, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use std::fs::OpenOptions;
use std::io::Write;

pub struct PicodataClient {
    client: reqwest::Client,
    base_url: String,
    auth_token: Option<String>,
    debug: bool,
}

impl PicodataClient {
    pub fn new(base_url: &str, debug: bool) -> Result<Self> {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()?;

        if debug {
            // Clear log file on start
            let _ = std::fs::write("picotui.log", "");
        }

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            auth_token: None,
            debug,
        })
    }

    fn log(&self, message: &str) {
        if self.debug {
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open("picotui.log")
            {
                let timestamp = chrono::Local::now().format("%H:%M:%S%.3f");
                let _ = writeln!(file, "[{}] {}", timestamp, message);
            }
        }
    }

    fn auth_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        if let Some(ref token) = self.auth_token {
            if let Ok(value) = HeaderValue::from_str(&format!("Bearer {}", token)) {
                headers.insert(AUTHORIZATION, value);
            }
        }
        headers
    }

    pub async fn get_config(&self) -> Result<UiConfig> {
        let url = format!("{}/api/v1/config", self.base_url);
        self.log(&format!("GET {}", url));

        let resp = self.client.get(&url).send().await?;
        let status = resp.status();

        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            self.log(&format!("  ERROR {}: {}", status, text));
            return Err(anyhow!("Failed to get config: {} - {}", status, text));
        }

        let text = resp.text().await?;
        self.log(&format!("  OK {}: {}", status, text));

        serde_json::from_str(&text).map_err(|e| {
            self.log(&format!("  PARSE ERROR: {}", e));
            anyhow!("Failed to parse config: {}", e)
        })
    }

    pub async fn login(&mut self, username: &str, password: &str) -> Result<TokenResponse> {
        let url = format!("{}/api/v1/session", self.base_url);
        self.log(&format!("POST {} (user={})", url, username));

        let req = LoginRequest {
            username: username.to_string(),
            password: password.to_string(),
        };

        let resp = self
            .client
            .post(&url)
            .header(CONTENT_TYPE, "application/json")
            .json(&req)
            .send()
            .await?;

        let status = resp.status();

        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            self.log(&format!("  ERROR {}: {}", status, text));
            if let Ok(err) = serde_json::from_str::<ErrorResponse>(&text) {
                return Err(anyhow!("{}", err.error_message));
            }
            return Err(anyhow!("Login failed: {} - {}", status, text));
        }

        let text = resp.text().await?;
        self.log(&format!("  OK {}: (tokens received)", status));

        let tokens: TokenResponse = serde_json::from_str(&text).map_err(|e| {
            self.log(&format!("  PARSE ERROR: {}", e));
            anyhow!("Failed to parse tokens: {}", e)
        })?;

        self.auth_token = Some(tokens.auth.clone());
        Ok(tokens)
    }

    pub async fn get_cluster_info(&self) -> Result<ClusterInfo> {
        let url = format!("{}/api/v1/cluster", self.base_url);
        self.log(&format!("GET {}", url));

        let resp = self
            .client
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await?;

        let status = resp.status();

        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            self.log(&format!("  ERROR {}: {}", status, text));
            return Err(anyhow!("Failed to get cluster info: {} - {}", status, text));
        }

        let text = resp.text().await?;
        self.log(&format!("  OK {}: {}", status, text));

        serde_json::from_str(&text).map_err(|e| {
            self.log(&format!("  PARSE ERROR: {}", e));
            anyhow!("Failed to parse cluster info: {} (response: {})", e, text)
        })
    }

    pub async fn get_tiers(&self) -> Result<Vec<TierInfo>> {
        let url = format!("{}/api/v1/tiers", self.base_url);
        self.log(&format!("GET {}", url));

        let resp = self
            .client
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await?;

        let status = resp.status();

        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            self.log(&format!("  ERROR {}: {}", status, text));
            return Err(anyhow!("Failed to get tiers: {} - {}", status, text));
        }

        let text = resp.text().await?;
        self.log(&format!("  OK {}: {}", status, text));

        serde_json::from_str(&text).map_err(|e| {
            self.log(&format!("  PARSE ERROR: {}", e));
            anyhow!("Failed to parse tiers: {} (response: {})", e, text)
        })
    }
}
