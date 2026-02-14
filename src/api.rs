use crate::models::*;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

/// Requests that can be sent to the API worker
#[derive(Debug)]
pub enum ApiRequest {
    GetConfig,
    Login { username: String, password: String },
    GetClusterInfo,
    GetTiers,
    Shutdown,
}

/// Responses from the API worker
#[derive(Debug)]
pub enum ApiResponse {
    Config(Result<UiConfig, String>),
    Login(Result<TokenResponse, String>),
    ClusterInfo(Result<ClusterInfo, String>),
    Tiers(Result<Vec<TierInfo>, String>),
}

/// Spawns a background thread that handles all HTTP requests
pub fn spawn_api_worker(
    base_url: String,
    request_rx: Receiver<ApiRequest>,
    response_tx: Sender<ApiResponse>,
    debug: bool,
) {
    thread::spawn(move || {
        let config = ureq::Agent::config_builder()
            .timeout_connect(Some(Duration::from_secs(5)))
            .timeout_recv_response(Some(Duration::from_secs(10)))
            .build();
        let client = config.new_agent();

        let mut auth_token: Option<String> = None;
        let base_url = base_url.trim_end_matches('/').to_string();

        for request in request_rx {
            match request {
                ApiRequest::Shutdown => break,

                ApiRequest::GetConfig => {
                    let url = format!("{}/api/v1/config", base_url);
                    log_debug(debug, &format!("GET {}", url));

                    let result = client.get(&url).call();
                    let response = match result {
                        Ok(resp) => match resp.into_body().read_json::<UiConfig>() {
                            Ok(config) => {
                                log_debug(debug, "  OK: config received");
                                Ok(config)
                            }
                            Err(e) => {
                                log_debug(debug, &format!("  PARSE ERROR: {}", e));
                                Err(format!("Failed to parse config: {}", e))
                            }
                        },
                        Err(e) => {
                            log_debug(debug, &format!("  ERROR: {}", e));
                            Err(format!("Failed to get config: {}", e))
                        }
                    };
                    let _ = response_tx.send(ApiResponse::Config(response));
                }

                ApiRequest::Login { username, password } => {
                    let url = format!("{}/api/v1/session", base_url);
                    log_debug(debug, &format!("POST {} (user={})", url, username));

                    let req_body = LoginRequest { username, password };
                    let result = client
                        .post(&url)
                        .header("Content-Type", "application/json")
                        .send_json(&req_body);

                    let response = match result {
                        Ok(resp) => match resp.into_body().read_json::<TokenResponse>() {
                            Ok(tokens) => {
                                log_debug(debug, "  OK: tokens received");
                                auth_token = Some(tokens.auth.clone());
                                Ok(tokens)
                            }
                            Err(e) => {
                                log_debug(debug, &format!("  PARSE ERROR: {}", e));
                                Err(format!("Failed to parse tokens: {}", e))
                            }
                        },
                        Err(ureq::Error::StatusCode(status)) => {
                            let msg = format!("Login failed: HTTP {}", status);
                            log_debug(debug, &format!("  ERROR: {}", msg));
                            Err(msg)
                        }
                        Err(e) => {
                            log_debug(debug, &format!("  ERROR: {}", e));
                            Err(format!("Login failed: {}", e))
                        }
                    };
                    let _ = response_tx.send(ApiResponse::Login(response));
                }

                ApiRequest::GetClusterInfo => {
                    let url = format!("{}/api/v1/cluster", base_url);
                    log_debug(debug, &format!("GET {}", url));

                    let mut req = client.get(&url);
                    if let Some(ref token) = auth_token {
                        req = req.header("Authorization", &format!("Bearer {}", token));
                    }

                    let result = req.call();
                    let response = match result {
                        Ok(resp) => match resp.into_body().read_json::<ClusterInfo>() {
                            Ok(info) => {
                                log_debug(debug, "  OK: cluster info received");
                                Ok(info)
                            }
                            Err(e) => {
                                log_debug(debug, &format!("  PARSE ERROR: {}", e));
                                Err(format!("Failed to parse cluster info: {}", e))
                            }
                        },
                        Err(e) => {
                            log_debug(debug, &format!("  ERROR: {}", e));
                            Err(format!("Failed to get cluster info: {}", e))
                        }
                    };
                    let _ = response_tx.send(ApiResponse::ClusterInfo(response));
                }

                ApiRequest::GetTiers => {
                    let url = format!("{}/api/v1/tiers", base_url);
                    log_debug(debug, &format!("GET {}", url));

                    let mut req = client.get(&url);
                    if let Some(ref token) = auth_token {
                        req = req.header("Authorization", &format!("Bearer {}", token));
                    }

                    let result = req.call();
                    let response = match result {
                        Ok(resp) => match resp.into_body().read_json::<Vec<TierInfo>>() {
                            Ok(tiers) => {
                                log_debug(debug, &format!("  OK: {} tiers received", tiers.len()));
                                Ok(tiers)
                            }
                            Err(e) => {
                                log_debug(debug, &format!("  PARSE ERROR: {}", e));
                                Err(format!("Failed to parse tiers: {}", e))
                            }
                        },
                        Err(e) => {
                            log_debug(debug, &format!("  ERROR: {}", e));
                            Err(format!("Failed to get tiers: {}", e))
                        }
                    };
                    let _ = response_tx.send(ApiResponse::Tiers(response));
                }
            }
        }
    });
}

fn log_debug(debug: bool, message: &str) {
    if debug {
        use std::fs::OpenOptions;
        use std::io::Write;
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open("picotui.log")
        {
            let elapsed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            let secs = elapsed.as_secs() % 86400;
            let hours = secs / 3600;
            let mins = (secs % 3600) / 60;
            let secs = secs % 60;
            let millis = elapsed.subsec_millis();
            let _ = writeln!(
                file,
                "[{:02}:{:02}:{:02}.{:03}] {}",
                hours, mins, secs, millis, message
            );
        }
    }
}
